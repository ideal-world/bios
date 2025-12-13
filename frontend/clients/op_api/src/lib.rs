//! # Op API Client
//! Webhook 客户端实现，支持通过 HTTP 方法发送 webhook 请求

use std::collections::HashMap;

use tardis::{
    basic::result::TardisResult,
    chrono::{FixedOffset, Utc},
    url::Url,
    web::reqwest::{header::{HeaderMap, HeaderValue}, Client, Method},
    TardisFuns,
};

mod api;
#[cfg(feature = "reach")]
pub mod ext;
pub use api::*;
mod model;

#[derive(Clone, Debug)]
pub struct OpApiClient {
    pub(crate) inner: Client,
    pub app_key: String,
    pub app_secret: String,
}

impl OpApiClient {
    pub fn new(app_key: impl Into<String>, app_secret: impl Into<String>) -> TardisResult<Self> {
        Ok(Self {
            inner: Default::default(),
            app_key: app_key.into(),
            app_secret: app_secret.into(),
        })
    }

    /// 添加认证头
    pub(crate) fn add_auth_headers_to(&self, headers: &mut HeaderMap, url: &str, method: &Method) -> TardisResult<()> {
        // 解析 URL
        let url = Url::parse(url)
            .map_err(|e| tardis::basic::error::TardisError::wrap(&format!("Invalid URL: {}", e), "400-invalid-url"))?;

        // 获取路径，移除开头的 "/"
        let mut path = url.path().to_string();
        if path.starts_with('/') {
            path = path[1..].to_string();
        }

        // 获取查询参数并 URL 解码（使用 query_pairs 自动解码）
        let query_params: HashMap<String, String> = url
            .query_pairs()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        // 格式化日期（GMT+8，即中国时区）
        let gmt_plus_8 = FixedOffset::east_opt(8 * 3600)
            .ok_or_else(|| tardis::basic::error::TardisError::wrap("Failed to create GMT+8 timezone", "500-timezone-error"))?;
        let req_date = (Utc::now().with_timezone(&gmt_plus_8))
            .format("%a, %d %b %Y %T GMT")
            .to_string();

        // 排序查询参数
        let sorted_query = Self::sort_hashmap_query(query_params);

        // 计算签名
        let method_str = method.as_str();
        let signature_string = format!("{}\n{}\n{}\n{}", method_str, req_date, path, sorted_query).to_lowercase();
        let signature_bytes = TardisFuns::crypto.digest.hmac_sha256(signature_string, &self.app_secret)?;
        let signature = TardisFuns::crypto.base64.encode(signature_bytes);

        // 添加认证头
        // 根据 Java 代码，使用 StarsysConstant.HEAD_KEY_DATE_FLAG 和 HEAD_KEY_AK_AUTHORIZATION
        // 这里使用常见的 header 名称，可以根据实际需要调整
        const HEAD_KEY_DATE_FLAG: &str = "Bios-Date";
        const HEAD_KEY_AK_AUTHORIZATION: &str = "Bios-Authorization";

        headers.insert(
            HEAD_KEY_DATE_FLAG,
            HeaderValue::from_str(&req_date)
                .map_err(|e| tardis::basic::error::TardisError::wrap(&format!("Failed to create date header: {}", e), "500-header-error"))?,
        );

        let auth_value = format!("{}:{}", self.app_key, signature);
        headers.insert(
            HEAD_KEY_AK_AUTHORIZATION,
            HeaderValue::from_str(&auth_value)
                .map_err(|e| tardis::basic::error::TardisError::wrap(&format!("Failed to create authorization header: {}", e), "500-header-error"))?,
        );

        Ok(())
    }


    /// 排序查询参数（按 key 的字典序，不区分大小写）
    fn sort_hashmap_query(query: HashMap<String, String>) -> String {
        if query.is_empty() {
            return String::new();
        }

        let mut entries: Vec<_> = query.iter().collect();
        entries.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

        entries
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&")
    }
}

