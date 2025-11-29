use std::{
    collections::HashSet,
    sync::{Arc, OnceLock},
};

use bios_reach::{
    dto::{ContentReplace, ReachChannelKind},
    reach_config::ReachConfig,
    reach_constants::MODULE_CODE,
    reach_send_channel::{GenericTemplate, SendChannel},
};
use tardis::{
    async_trait::async_trait,
    basic::{error::TardisError, result::TardisResult},
    TardisFuns,
    web::reqwest::Method,
};

use crate::{OpApiClient, WebhookRequest};

#[async_trait]
impl SendChannel for OpApiClient {
    fn kind(&self) -> ReachChannelKind {
        ReachChannelKind::WebHook
    }

    async fn send(&self, template: GenericTemplate<'_>, content: &ContentReplace, _to: &HashSet<&str>) -> TardisResult<()> {
        // 从模板的 topic 字段获取 webhook URL
        let webhook_url = template
            .topic
            .ok_or_else(|| TardisError::conflict("template missing webhook URL (topic field)", "409-reach-bad-template"))?;

        // 从 content 中获取 webhook_method，默认为 POST
        let method_str = content.get("webhook_method").map(|s| s.as_str()).unwrap_or("POST");
        let method = Method::from_bytes(method_str.as_bytes())
            .map_err(|e| TardisError::wrap(&format!("Invalid HTTP method '{}': {}", method_str, e), "400-invalid-webhook-method"))?;

        // 从 content 中获取 webhook_content 作为请求体
        let webhook_content = content.get("webhook_content").map(|s| s.as_str());

        let request = WebhookRequest::new(webhook_url, method, webhook_content);
        self.send_webhook(request).await
    }
}

impl OpApiClient {
    pub fn from_reach_config() -> Arc<Self> {
        static OP_API_CLIENT: OnceLock<Arc<OpApiClient>> = OnceLock::new();
        OP_API_CLIENT
            .get_or_init(|| {
                // this would block thread but it's ok
                let config = TardisFuns::cs_config::<ReachConfig>(MODULE_CODE);
                let opapi_config = &config.opapi;
                OpApiClient::new(&opapi_config.app_key, &opapi_config.app_secret)
                    .expect("Failed to create OpApiClient")
                    .into()
            })
            .clone()
    }
}

