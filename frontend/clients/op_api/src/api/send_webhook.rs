use tardis::{
    basic::{error::TardisError, result::TardisResult},
    web::reqwest::{header::HeaderMap, Method},
};

use crate::OpApiClient;

/// Webhook 请求参数
#[derive(Debug)]
pub struct WebhookRequest<'r> {
    pub url: &'r str,
    pub method: Method,
    pub content: Option<&'r str>,
}

impl<'r> WebhookRequest<'r> {
    pub fn new(url: &'r str, method: Method, content: Option<&'r str>) -> Self {
        Self { url, method, content }
    }
}

impl OpApiClient {
    /// 发送 webhook 请求
    pub async fn send_webhook(&self, request: WebhookRequest<'_>) -> TardisResult<()> {
        tardis::log::trace!("send webhook request: {:?}", request);

        let mut request_builder = self.inner.request(request.method.clone(), request.url);

        // 添加认证头
        let mut headers = HeaderMap::new();
        self.add_auth_headers_to(&mut headers, request.url, &request.method)?;
        request_builder = request_builder.headers(headers);

        // 对于支持请求体的方法（POST, PUT, PATCH），添加请求体
        match request.method {
            Method::GET | Method::DELETE | Method::HEAD | Method::OPTIONS | Method::TRACE => {}
            _ => {
                // 从 content 中获取 webhook_content 作为请求体
                if let Some(webhook_content) = request.content {
                    let json_value = tardis::TardisFuns::json.str_to_json(webhook_content)?;
                    request_builder = request_builder.json(&json_value);
                }
            }
        }

        let response = request_builder
            .send()
            .await
            .map_err(|e| TardisError::wrap(&format!("Failed to send webhook request: {}", e), "500-webhook-send-error"))?;

        // 检查响应状态
        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            return Err(TardisError::wrap(
                &format!("Webhook request failed with status {}: {}", status, error_body),
                "500-webhook-response-error",
            ));
        }

        Ok(())
    }
}

