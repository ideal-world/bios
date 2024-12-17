use std::{collections::HashMap, sync::Arc};

use http::{HeaderName, HeaderValue, Uri};
use serde::{Deserialize, Serialize};
use spacegate_shell::{
    kernel::backend_service::http_client_service::get_client,
    plugin::{plugin_meta, schema, schemars, Inner, Plugin, PluginSchemaExt},
    BoxError, SgRequest, SgRequestExt, SgResponse,
};
use tardis::serde_json;

use crate::extension::notification::NotificationContext;
#[derive(Serialize, Deserialize, schemars::JsonSchema)]
pub struct NotifyPluginConfig {
    api: String,
    headers: HashMap<String, String>,
}
schema!(NotifyPlugin, NotifyPluginConfig);
#[derive(Debug, Clone)]
pub struct NotifyPlugin {
    api: Arc<Uri>,
    headers: Arc<HashMap<HeaderName, HeaderValue>>,
}

impl Plugin for NotifyPlugin {
    const CODE: &'static str = "notify";
    fn meta() -> spacegate_shell::model::PluginMetaData {
        plugin_meta! {
            description: "attach a notification api calling context to the request"
        }
    }
    fn create(plugin_config: spacegate_shell::model::PluginConfig) -> Result<Self, BoxError> {
        // parse uri
        let config: NotifyPluginConfig = serde_json::from_value(plugin_config.spec)?;
        let api = config.api.parse::<Uri>()?;
        let headers = config
            .headers
            .into_iter()
            .map_while(|(k, v)| {
                if let (Ok(k), Ok(v)) = (k.parse::<HeaderName>(), v.parse::<HeaderValue>()) {
                    Some((k, v))
                } else {
                    None
                }
            })
            .collect();
        Ok(Self {
            api: Arc::new(api),
            headers: Arc::new(headers),
        })
    }
    async fn call(&self, mut req: SgRequest, inner: Inner) -> Result<SgResponse, BoxError> {
        let context = NotificationContext {
            api: self.api.clone(),
            headers: self.headers.clone(),
            client: get_client(),
        };
        req.extensions_mut().insert(context.clone());
        req.reflect_mut().insert(context);
        Ok(inner.call(req).await)
    }
    fn schema_opt() -> Option<schemars::schema::RootSchema> {
        Some(NotifyPlugin::schema())
    }
}
