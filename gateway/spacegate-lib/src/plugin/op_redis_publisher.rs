use std::{
    str::FromStr as _,
    time::{Duration, Instant},
};

use http::Response;
use jsonpath_rust::JsonPathInst;
use serde::{Deserialize, Serialize};
use spacegate_shell::{
    kernel::{
        extension::{EnterTime, GatewayName},
        SgRequest, SgResponse,
    },
    plugin::{Inner, Plugin, PluginConfig},
    spacegate_ext_redis::global_repo,
    BoxError,
};
use tardis::{
    cache::Script,
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
    const CODE: &'static str = "op_redis_publisher";

    fn create(config: PluginConfig) -> Result<Self, BoxError> {
        let id = config.none_mono_id();
        let layer_config = serde_json::from_value::<RedisPublisherConfig>(config.spec.clone())?;

        Ok(Self {
            key: id.redis_prefix(),
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
        let Some(gateway_name) = req.extensions().get::<GatewayName>() else {
            return Err("missing gateway name".into());
        };
        let Some(client) = global_repo().get(gateway_name) else {
            return Err("missing redis client".into());
        };
        let spec_id = RedisPublisherPlugin::parse_spec_id(&req);

        let resp = inner.call(req).await;

        let (resp, content) = self.op_log(resp).await?;

        if let Some(mut content) = content {
            content.spec_id = spec_id;
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
    fn parse_spec_id(req: &SgRequest) -> Option<String> {
        let segments: Vec<_> = req.uri().path().split('/').collect();
        //找到segment为op-api的下一个segment就是spec_id
        if let Some(index) = segments.iter().position(|&seg| seg == "op-api") {
            // 确保 "op-api" 后面还有段
            if let Some(spec_id) = segments.get(index + 1) {
                return Some(spec_id.to_string());
            }
        }
        None
    }

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
            warn!("[Plugin.OpRedisPubilsher] missing audit log param");
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
            server_timing: start_time.map(|st| end_time - st),
            resp_status: resp.status().as_u16().to_string(),
            success,
            own_paths: resp.extensions().get::<CertInfo>().and_then(|info| info.own_paths.clone()),
            spec_id: None,
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
    pub server_timing: Option<Duration>,
    pub resp_status: String,
    //Indicates whether the business operation was successful.
    pub success: bool,
    pub spec_id: Option<String>,
}
