use serde::{Deserialize, Serialize};
use spacegate_shell::{
    kernel::{extension::GatewayName, SgRequest},
    plugin::{Inner, Plugin, PluginConfig},
    spacegate_ext_redis::global_repo,
    BoxError,
};
use tardis::{
    cache::Script,
    log,
    serde_json::{self, json, Value},
    tokio,
};

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
            form_response_extensions: vec!["log_content".to_string()],
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
    const CODE: &'static str = "op_redis_publisher";

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

    async fn call(&self, req: SgRequest, inner: Inner) -> Result<http::Response<spacegate_shell::SgBody>, spacegate_shell::BoxError> {
        let Some(gateway_name) = req.extensions().get::<GatewayName>() else {
            return Err("missing gateway name".into());
        };
        let Some(client) = global_repo().get(gateway_name) else {
            return Err("missing redis client".into());
        };
        let mut value = serde_json::Map::<String, Value>::new();
        if let Some(spec_id) = RedisPublisherPlugin::parse_spec_id(&req) {
            value.insert("spec_id".to_string(), json!(spec_id));
        }

        for ext_enum in &self.form_requset_extensions {
            if let Some(mut ext) = ext_enum.to_value(req.extensions())? {
                value.extend(ext.as_object_mut().unwrap().into_iter().map(|(k, v)| (k.clone(), v.clone())));
            }
        }
        let resp = inner.call(req).await;

        for ext_enum in &self.form_response_extensions {
            if let Some(mut ext) = ext_enum.to_value(resp.extensions())? {
                value.extend(ext.as_object_mut().unwrap().into_iter().map(|(k, v)| (k.clone(), v.clone())));
            }
        }

        let key = self.key.clone();
        let script = self.script.clone();
        tokio::task::spawn(async move {
            let mut conn = client.get_conn().await;
            match serde_json::to_string(&value) {
                Ok(v) => {
                    match script.key(key).arg(v).invoke_async::<_, bool>(&mut conn).await {
                        Ok(_) => {
                            log::trace!("[Plugin.OPRedisPublisher]Publish success")
                        }
                        Err(e) => {
                            log::warn!("[Plugin.OPRedisPublisher] failed to Publish:{e}")
                        }
                    };
                }
                Err(e) => {
                    log::warn!("[Plugin.OPRedisPublisher] failed to Deserialize:{e}")
                }
            }
        });

        Ok(resp)
    }
}

impl RedisPublisherPlugin {
    fn parse_spec_id(req: &SgRequest) -> Option<String> {
        let segments: Vec<_> = req.uri().path().split("/").collect();
        //找到segment为op-api的下一个segment就是spec_id
        if let Some(index) = segments.iter().position(|&seg| seg == "op-api") {
            // 确保 "op-api" 后面还有段
            if let Some(spec_id) = segments.get(index + 1) {
                return Some(spec_id.to_string());
            }
        }
        None
    }
}
