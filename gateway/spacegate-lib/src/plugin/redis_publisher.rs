use serde::{Deserialize, Serialize};
use spacegate_shell::{
    kernel::extension::{ExtensionPack, GatewayName},
    plugin::{Inner, Plugin, PluginConfig},
    spacegate_ext_redis::global_repo,
    BoxError,
};
use tardis::{cache::Script, serde_json};

use crate::extension::ExtensionPackEnum;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct RedisPublisherConfig {
    pub form_requset_extensions: Vec<String>,
    pub form_response_extensions: Vec<String>,
}

impl Default for RedisPublisherConfig {
    fn default() -> Self {
        Self {
            form_requset_extensions: Default::default(),
            form_response_extensions: Default::default(),
        }
    }
}
pub struct RedisPublisherPlugin {
    pub key: String,
    pub form_requset_extensions: Vec<ExtensionPackEnum>,
    pub form_response_extensions: Vec<ExtensionPackEnum>,
    pub script: Script,
}

impl Plugin for RedisPublisherPlugin {
    const CODE: &'static str = "redis_publisher";

    fn create(config: PluginConfig) -> Result<Self, BoxError> {
        let id = config.none_mono_id();
        let layer_config = serde_json::from_value::<RedisPublisherConfig>(config.spec.clone())?;
        Ok(Self {
            key: id.redis_prefix(),
            form_requset_extensions: layer_config.form_requset_extensions.into_iter().map(Into::into).collect(),
            form_response_extensions: layer_config.form_response_extensions.into_iter().map(Into::into).collect(),
            script: Script::new(
                r##"
          local channel = KEYS[1];
          local message = ARGV[1];

          PUBLISH channel message
          "##,
            ),
        })
    }

    async fn call(&self, req: http::Request<spacegate_shell::SgBody>, inner: Inner) -> Result<http::Response<spacegate_shell::SgBody>, spacegate_shell::BoxError> {
        let Some(gateway_name) = req.extensions().get::<GatewayName>() else {
            return Err("missing gateway name".into());
        };
        let Some(client) = global_repo().get(gateway_name) else {
            return Err("missing redis client".into());
        };
        let mut conn = client.get_conn().await;
        for ext_enum in &self.form_requset_extensions {
            match ext_enum {
                ExtensionPackEnum::AuditLogParam(ext) => {
                    if let Some(ext) = ext.get(req.extensions()) {
                        self.script.key(&self.key).arg(serde_json::to_string(ext)?).invoke_async(&mut conn).await?;
                    }
                }
                ExtensionPackEnum::LogParamContent(ext) => {
                    if let Some(ext) = ext.get(req.extensions()) {
                        self.script.key(&self.key).arg(serde_json::to_string(ext)?).invoke_async(&mut conn).await?;
                    }
                }
                ExtensionPackEnum::None => (),
            }
        }
        let reslt = inner.call(req).await;

        Ok(reslt)
    }
}
