//! # Huawei Cloud Platform Sms Client
//! reference: https://support.huaweicloud.com/msgsms/index.html

use std::{borrow::Cow, collections::BTreeMap, time::{SystemTime, SystemTimeError}};

use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use rand::Rng;
use tardis::{
    basic::{error::TardisError, result::TardisResult},
    chrono::{DateTime, SecondsFormat, Utc},
    crypto::crypto_digest::algorithm::{hex, Digest, Hmac, Mac, Sha256},
    rand::random,
    url::Url,
    web::reqwest::{
        header::{HeaderMap, HeaderValue, AUTHORIZATION}, Client, Method, Response
    },
};
mod api;
mod ext;
pub use api::*;
mod model;
pub use model::*;
#[derive(Clone, Debug)]
pub struct SmsClient {
    pub(crate) inner: Client,
    pub app_key: String,
    pub app_secret: String,
    pub base_url: String,
}

impl SmsClient {
    pub fn new(base_url: String, app_key: impl Into<String>, app_secret: impl Into<String>) -> Self {
        let app_key: String = app_key.into();
        let app_secret: String = app_secret.into();

        SmsClient {
            inner: Default::default(),
            base_url,
            app_key,
            app_secret,
        }
    }
    fn signature(
        &self,
        action: &str,
        method: Method,
        body_content: &str,
        canonical_uri: &str,
        canonical_query_string: &str,
        version: &str,
        query_params: &[(&str, &str)],
        headers: &mut HeaderMap
    ) -> TardisResult<()> {
        fn sha256_hex(message: &str) -> String {
            let mut hasher = Sha256::new();
            hasher.update(message);
            format!("{:x}", hasher.finalize()).to_lowercase()
        }
        fn hmac256(key: &[u8], message: &str) -> TardisResult<Vec<u8>> {
            let mut mac = Hmac::<Sha256>::new_from_slice(key)
            .map_err(|e| TardisError::internal_error(&format!("use data key on sha256 fail:{}", e), "500-reach-send-failed"))?;
            mac.update(message.as_bytes());
            let signature = mac.finalize();
            Ok(signature.into_bytes().to_vec())
        }
        let hashed_request_payload = if body_content.is_empty() {
            sha256_hex("") 
        } else {
            sha256_hex(&body_content) 
        };

        // x-acs-date
        let now_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|e| TardisError::internal_error(&format!("Get current timestamp failed: {}", e), "500-reach-send-failed"))?
            .as_secs();
        let datetime = DateTime::from_timestamp(now_time as i64, 0).ok_or_else(|| TardisError::internal_error(&format!("Get datetime from timestamp failed: {}", now_time), "500-reach-send-failed"))?;
        let datetime_str = datetime.format("%Y-%m-%dT%H:%M:%SZ").to_string();

        // x-acs-signature-nonce
        let signature_nonce = generate_random_string(32);
        // 待签名请求头
        let sign_header_arr = &[
            "host",
            "x-acs-action",
            "x-acs-content-sha256",
            "x-acs-date",
            "x-acs-signature-nonce",
            "x-acs-version",
        ];
        let sign_headers = sign_header_arr.join(";");
        // 1.构造规范化请求头
        headers.insert("Host", HeaderValue::from_str(self.base_url.as_str()).unwrap());
        headers.insert("x-acs-action", HeaderValue::from_str(action).unwrap());
        headers.insert("x-acs-version", HeaderValue::from_str(version).unwrap());
        headers.insert("x-acs-date", HeaderValue::from_str(&datetime_str).unwrap());
        headers.insert("x-acs-signature-nonce", HeaderValue::from_str(&signature_nonce).unwrap());
        headers.insert("x-acs-content-sha256", HeaderValue::from_str(&hashed_request_payload).unwrap());

        // 2.构造待签名请求头
        let canonical_request = format!(
            "{}\n{}\n{}\n{}\n\n{}\n{}",
            method.as_str(),
            canonical_uri,
            canonical_query_string,
            sign_header_arr.iter().map(|&header| format!("{}:{}", header, headers[header].to_str().unwrap())).collect::<Vec<_>>().join("\n"),
            sign_headers,
            hashed_request_payload
        );
        // 3.计算待签名请求头的 SHA-256 哈希;
        let result = sha256_hex(&canonical_request);
        // 4.构建待签名字符串
        let string_to_sign = format!("ACS3-HMAC-SHA256\n{}", result);
        // 5.计算签名
        let signature = hmac256(self.app_secret.as_bytes(), &string_to_sign).map_err(|e| TardisError::internal_error(&format!("Calculate signature failed: {}", e), "500-reach-send-failed"))?;
        let data_sign = hex::encode(&signature);
        let auth_data = format!(
            "ACS3-HMAC-SHA256 Credential={},SignedHeaders={},Signature={}",
            self.app_key, sign_headers, data_sign
        );
        // 6.构建 Authorization
        headers.insert("Authorization", HeaderValue::from_str(&auth_data).unwrap());
        Ok(())
    }

    async fn send_request(
        &self,
        method: Method,
        url: &str,
        headers: HeaderMap,
        query_params: &[(&str, &str)],     // 接收 query 参数
        body: &RequestBody,                // 用此判断 body 数据类型
        body_content: &str,                // body 不为空时 接收 body 请求参数 FormData/Json/Binary
    ) -> TardisResult<Response> {
        let mut request_builder = self.inner.request(method.clone(), url);
        // 添加请求头 headers
        for (k, v) in headers.iter() {
            request_builder = request_builder.header(k, v.clone());
        }
         // 添加请求体 body
         match body {
            RequestBody::Binary(_) => {
                request_builder = request_builder.header("Content-Type", "application/octet-stream");
                request_builder = request_builder.body(body_content.to_string()); // 移动这里的值
            }
            RequestBody::Json(_) => {
                // 如果body为map，且不为空，转化为Json后存储在 body_content 变量中，设置  application/json; charset=utf-8
                if !body_content.is_empty() { 
                    request_builder = request_builder.body(body_content.to_string());
                    request_builder = request_builder.header("Content-Type", "application/json; charset=utf-8");
                }
            }
            RequestBody::FormData(_) => {
                // 处理 form-data 类型，设置 content-type
                if !body_content.is_empty() { 
                request_builder = request_builder.header("Content-Type", "application/x-www-form-urlencoded");
                request_builder = request_builder.body(body_content.to_string());
                }
            }
            RequestBody::None => {
                request_builder = request_builder.body(String::new());
            }
        }
        // 构建请求
        let request = request_builder
            .build()
            .map_err(|e| TardisError::internal_error(&format!("build request fail: {}", e), "500-reach-send-failed"))?;
        // 发起请求
        let response = self.inner
            .execute(request)
            .await
            .map_err(|e| TardisError::internal_error(&format!("execute request fail: {}", e), "500-reach-send-failed"))?;
        // 返回结果
        Ok(response)
    }

    // 规范化请求
    pub async fn call_api(
        &self,
        method: Method,
        canonical_uri: &str,
        query_params: &[(&str, &str)], 
        action: &str,
        version: &str,
        body: RequestBody,
    ) -> TardisResult<Response> {
        fn build_sored_encoded_query_string(query_params: &[(&str, &str)]) -> String {
            let sorted_query_params: BTreeMap<_, _> = query_params.iter().copied().collect();
            let encoded_params: Vec<String> = sorted_query_params
                .into_iter()
                .map(|(k, v)| {
                    let encoded_key = percent_code(k);
                    let encoded_value = percent_code(v);
                    format!("{}={}", encoded_key, encoded_value)
                })
                .collect();
            encoded_params.join("&")
        }
        let body_content = body.to_string();

        let mut headers = HeaderMap::new();

        let canonical_query_string = build_sored_encoded_query_string(query_params); // 参数编码拼接处理
        // 签名
        self.signature(action, method.clone(), &body_content, canonical_uri, &canonical_query_string, version, query_params, &mut headers);
        // 构造 url 拼接请求参数
        let url: String;
        if !query_params.is_empty() {
            url = format!("https://{}{}?{}", self.base_url.as_str(), canonical_uri, canonical_query_string);
        } else {
            url = format!("https://{}{}", self.base_url.as_str(), canonical_uri);
        }
        let response = self.send_request(
            method,
            &url,
            headers,
            query_params,                
            &body,                      
            &body_content,                
        )
        .await?;

        Ok(response)
    }
}

// 生成签名唯一随机数
pub fn generate_random_string(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| CHARSET[rng.gen_range(0..CHARSET.len())] as char)
        .collect()
}

pub fn percent_code(encode_str: &str) -> Cow<'_, str> {
    let encoded = utf8_percent_encode(encode_str, NON_ALPHANUMERIC)
        .to_string()
        .replace("+", "20%")
        .replace("%5F", "_")
        .replace("%2D", "-")
        .replace("%2E", ".")
        .replace("%7E", "~");
        
    Cow::Owned(encoded) // 返回一个 Cow<str> 可以持有 String 或 &str
}