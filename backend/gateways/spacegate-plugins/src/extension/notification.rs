use std::{borrow::Cow, collections::HashMap, sync::Arc};

use http::{header::CONTENT_TYPE, HeaderName, HeaderValue, Uri};
use serde::Serialize;
use spacegate_shell::{kernel::backend_service::http_client_service::HttpClient, SgBody, SgRequest};
use tardis::{log as tracing, serde_json};

/// Context to call notification api
///
/// Extract it from request extensions, and call [`NotificationContext::notify`] to send notification
#[derive(Debug, Clone)]
pub struct NotificationContext {
    pub(crate) api: Arc<Uri>,
    pub(crate) headers: Arc<HashMap<HeaderName, HeaderValue>>,
    pub(crate) client: HttpClient,
}

impl NotificationContext {
    fn build_notification_request(&self, req: &ReachMsgSendReq) -> SgRequest {
        let req_bytes = serde_json::to_vec(&req).expect("ReachMsgSendReq is a valid json");
        let body = SgBody::full(req_bytes);
        let mut req = SgRequest::new(body);
        req.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        *req.uri_mut() = self.api.as_ref().clone();
        for (k, v) in self.headers.iter() {
            req.headers_mut().insert(k.clone(), v.clone());
        }
        req
    }
    pub async fn notify(&self, req: &ReachMsgSendReq) {
        let notify_response = self.client.clone().request(self.build_notification_request(req)).await;
        if !notify_response.status().is_success() {
            tracing::warn!(response = ?notify_response, "send notification failed");
        }

        let Ok(response) = notify_response.into_body().dump().await.inspect_err(|e| {
            tracing::error!(error = ?e, "failed to read response body");
        }) else {
            return;
        };
        let response_str = String::from_utf8_lossy(response.get_dumped().expect("just dump body"));
        tracing::debug!(response = ?response_str, "receive notification api response");
    }
}

#[derive(Debug, Serialize)]
pub struct ReachMsgSendReq {
    pub scene_code: String,
    pub receives: Vec<ReachMsgReceive>,
    pub rel_item_id: String,
    pub replace: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct ReachMsgReceive {
    pub receive_group_code: String,
    pub receive_kind: String,
    pub receive_ids: Vec<String>,
}
