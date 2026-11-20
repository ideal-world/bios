use std::collections::HashMap;

use serde::Serialize;
use tardis::{
    basic::{error::TardisError, result::TardisResult},
    serde_json,
    url::Url,
    web::reqwest::{header::HeaderMap, Method},
};

use crate::{model::*, SmsClient};

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SendSmsRequest<'r> {
    pub phone_numbers: &'r str,
    pub template_code: &'r str,
    pub template_param: Option<String>,
    pub sign_name: &'r str,
}

impl<'r> SendSmsRequest<'r> {
    pub fn new(phone_numbers: &'r str, sms_content: SmsContent<'r>) -> Self {
        let template_param = if !sms_content.template_param.is_empty() {
            Some(serde_json::to_string(&sms_content.template_param).expect("string[] to rust String shouldn't fail"))
        } else {
            None
        };
        Self {
            phone_numbers,
            template_code: sms_content.template_code,
            sign_name: sms_content.sign_name,
            template_param,
        }
    }
}

impl SmsClient {
    pub async fn send_sms(&self, request: SendSmsRequest<'_>) -> TardisResult<SmsResponse> {
        tardis::log::debug!("[SMS] request: phone_numbers={}, template_code={}, sign_name={}, template_param={:?}", request.phone_numbers, request.template_code, request.sign_name, request.template_param);
        let action = "SendSms"; // API名称
        let version = "2017-05-25"; // API版本号
        let mut query: Vec<(&str, &str)> = Vec::new();
        query.push(("PhoneNumbers", request.phone_numbers));
        query.push(("TemplateCode", request.template_code));
        query.push(("SignName", request.sign_name));
        if let Some(template_param) = request.template_param.as_ref() {
            query.push(("TemplateParam", template_param));
        }
        // query 参数
        let query_params: &[(&str, &str)] = &query;
        // 请求体 body 为空时
        let body = RequestBody::Json(HashMap::new());
        tardis::log::debug!("[SMS] call_api: base_url={}, real_url={}, action={}, version={}, query_params={:?}", self.base_url, self.real_url, action, version, query_params);
        // 发起请求
        let resp = self.call_api(Method::POST, "/", query_params, action, version, body).await?;
        let resp_status = resp.status();
        let resp_headers = resp.headers().clone();
        tardis::log::debug!("[SMS] response status: {}", resp_status);
        tardis::log::debug!("[SMS] response headers: {:?}", resp_headers);
        let body_bytes = resp.bytes().await.map_err(|e| TardisError::internal_error(&format!("read response body failed: {}", e), "500-reach-send-failed"))?;
        tardis::log::debug!("[SMS] response body: {}", String::from_utf8_lossy(&body_bytes));
        if body_bytes.is_empty() {
            tardis::log::debug!("[SMS] ERROR: empty response body, status={}", resp_status);
            return Err(TardisError::internal_error("sms gateway returned empty response body", "500-reach-send-failed"));
        }
        serde_json::from_slice(&body_bytes).map_err(|e| TardisError::internal_error(&format!("parse sms response failed: {}", e), "500-reach-send-failed"))
    }
}
