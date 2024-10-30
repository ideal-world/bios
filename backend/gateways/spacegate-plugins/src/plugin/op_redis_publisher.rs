use std::{str::FromStr as _, time::Instant};

use http::Response;
use jsonpath_rust::JsonPathInst;
use serde::{Deserialize, Serialize};
use spacegate_shell::{
    ext_redis::redis::Script,
    kernel::{extension::EnterTime, SgRequest, SgResponse},
    plugin::{Inner, Plugin, PluginConfig},
    BoxError, SgRequestExt as _,
};
use tardis::{
    log::{self, warn},
    serde_json::{self, Value},
    tokio,
};

use crate::extension::{audit_log_param::AuditLogParam, before_encrypt_body::BeforeEncryptBody, cert_info::CertInfo};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct RedisPublisherConfig {
    success_json_path: String,
    success_json_path_values: Vec<String>,
}

impl Default for RedisPublisherConfig {
    fn default() -> Self {
        Self {
            success_json_path: "$.code".to_string(),
            success_json_path_values: vec!["200".to_string(), "201".to_string()],
        }
    }
}
pub struct RedisPublisherPlugin {
    pub key: String,
    pub script: Script,
    jsonpath_inst: Option<JsonPathInst>,
    success_json_path_values: Vec<String>,
}

impl Plugin for RedisPublisherPlugin {
    const CODE: &'static str = "op-redis-publisher";

    fn meta() -> spacegate_shell::model::PluginMetaData {
        spacegate_shell::model::plugin_meta!(
            description: "Build for open platform, and it depend on plugin audit-log"
        )
    }

    fn create(config: PluginConfig) -> Result<Self, BoxError> {
        let layer_config = serde_json::from_value::<RedisPublisherConfig>(config.spec.clone())?;

        Ok(Self {
            key: config.id.redis_prefix(),
            jsonpath_inst: if let Ok(jsonpath_inst) = JsonPathInst::from_str(&layer_config.success_json_path).map_err(|e| log::error!("[Plugin.AuditLog] invalid json path:{e}")) {
                Some(jsonpath_inst)
            } else {
                None
            },
            success_json_path_values: layer_config.success_json_path_values,
            script: Script::new(
                r##"
          local channel = KEYS[1];
          local message = ARGV[1];

          return redis.call('PUBLISH',channel,message);
          "##,
            ),
        })
    }

    async fn call(&self, req: SgRequest, inner: Inner) -> Result<http::Response<spacegate_shell::SgBody>, spacegate_shell::BoxError> {
        let Some(client) = req.get_redis_client_by_gateway_name() else {
            return Err("missing redis client".into());
        };

        let resp = inner.call(req).await;

        let (resp, content) = self.op_log(resp).await?;

        if let Some(content) = content {
            let key = self.key.clone();
            let script = self.script.clone();
            tokio::task::spawn(async move {
                let mut conn = client.get_conn().await;
                match serde_json::to_string(&content) {
                    Ok(v) => {
                        match script.key(&key).arg(v).invoke_async::<_, bool>(&mut conn).await {
                            Ok(_) => {
                                log::trace!("[Plugin.OPRedisPublisher]Publish channel:{key} success")
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
        }

        Ok(resp)
    }
}

impl RedisPublisherPlugin {
    async fn op_log(&self, mut resp: SgResponse) -> Result<(SgResponse, Option<OpLogContent>), BoxError> {
        let body_string = if let Some(raw_body) = resp.extensions().get::<BeforeEncryptBody>().map(|b| b.clone().get()) {
            serde_json::from_str::<Value>(&String::from_utf8_lossy(&raw_body))
        } else {
            let body = if let Some(dumped) = resp.body().get_dumped() {
                dumped.clone()
            } else {
                let (parts, body) = resp.into_parts();
                let body = body.dump().await.map_err(|e: BoxError| format!("[SG.Filter.AuditLog] dump body error: {e}"))?;
                resp = Response::from_parts(parts, body.dump_clone().expect(""));
                body.get_dumped().expect("not expect").clone()
            };
            serde_json::from_slice::<Value>(&body)
        };

        let Some(param) = resp.extensions().get::<AuditLogParam>() else {
            warn!("[Plugin.OpRedisPublisher] missing audit log param");
            return Ok((resp, None));
        };

        let start_time = resp.extensions().get::<EnterTime>().map(|time| time.0);
        let end_time = Instant::now();

        let success = match body_string {
            Ok(json) => {
                if let Some(jsonpath_inst) = &self.jsonpath_inst {
                    if let Some(matching_value) = jsonpath_inst.find_slice(&json).first() {
                        if matching_value.is_string() {
                            let mut is_match = false;
                            for value in self.success_json_path_values.clone() {
                                if Some(value.as_str()) == matching_value.as_str() {
                                    is_match = true;
                                    break;
                                }
                            }
                            is_match
                        } else if matching_value.is_number() {
                            let mut is_match = false;
                            for value in self.success_json_path_values.clone() {
                                let value = value.parse::<i64>();
                                if value.is_ok() && value.ok() == matching_value.as_i64() {
                                    is_match = true;
                                    break;
                                }
                            }
                            is_match
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Err(_) => false,
        };

        let content = OpLogContent {
            op: param.request_method.clone(),
            name: resp.extensions().get::<CertInfo>().and_then(|info| info.name.clone()).unwrap_or_default(),
            user_id: resp.extensions().get::<CertInfo>().map(|info| info.id.clone()),
            ip: param.request_ip.clone(),
            path: param.request_path.clone(),
            scheme: param.request_scheme.clone(),
            server_timing: start_time.map(|st| (end_time - st).as_nanos()),
            resp_status: resp.status().as_u16().to_string(),
            success,
            own_paths: resp.extensions().get::<CertInfo>().and_then(|info| info.own_paths.clone()),
        };
        Ok((resp, Some(content)))
    }
}

#[derive(Deserialize, Serialize)]
struct OpLogContent {
    pub op: String,
    pub name: String,
    pub user_id: Option<String>,
    pub own_paths: Option<String>,
    pub ip: String,
    pub path: String,
    pub scheme: String,
    pub server_timing: Option<u128>,
    pub resp_status: String,
    //Indicates whether the business operation was successful.
    pub success: bool,
}
