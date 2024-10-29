use std::collections::HashMap;

use std::str::FromStr;
use std::time::Instant;

use bios_sdk_invoke::clients::spi_log_client::{self, LogItemAddV2Req};
use bios_sdk_invoke::invoke_config::InvokeConfig;
use bios_sdk_invoke::invoke_enumeration::InvokeModuleKind;
use bios_sdk_invoke::invoke_initializer;

use http::uri::Scheme;
use jsonpath_rust::JsonPathInst;
use serde::{Deserialize, Serialize};
use spacegate_shell::hyper::{Request, Response};
use spacegate_shell::kernel::extension::{EnterTime, PeerAddr, Reflect};

use spacegate_shell::kernel::helper_layers::function::Inner;
use spacegate_shell::plugin::{Plugin, PluginError};
use spacegate_shell::{BoxError, SgBody};
use tardis::basic::dto::TardisContext;
use tardis::log::{debug, trace, warn};
use tardis::serde_json::{json, Value};

use tardis::basic::error::TardisError;
use tardis::{
    basic::result::TardisResult,
    log,
    serde_json::{self},
    tokio::{self},
    TardisFuns, TardisFunsInst,
};

use crate::extension::audit_log_param::{AuditLogParam, LogParamContent};
use crate::extension::before_encrypt_body::BeforeEncryptBody;
use crate::extension::cert_info::CertInfo;

pub const CODE: &str = "audit-log";

use spacegate_shell::plugin::schemars;

spacegate_shell::plugin::schema!(AuditLogPlugin, AuditLogPlugin);

#[derive(Serialize, Deserialize, Clone, schemars::JsonSchema)]
#[serde(default)]
pub struct AuditLogPlugin {
    log_url: String,
    spi_app_id: String,
    tag: String,
    header_token_name: String,
    success_json_path: String,
    success_json_path_values: Vec<String>,
    /// Exclude log path exact match.
    exclude_log_path: Vec<String>,
    enabled: bool,
    #[serde(skip)]
    jsonpath_inst: Option<JsonPathInst>,
    head_key_auth_ident: String,
}

impl AuditLogPlugin {
    async fn get_log_content(&self, mut resp: Response<SgBody>) -> TardisResult<(Response<SgBody>, Option<LogParamContent>)> {
        let Some(param) = resp.extensions_mut().get::<AuditLogParam>().cloned() else {
            warn!("[Plugin.AuditLog] missing audit log param");
            return Ok((resp, None));
        };
        let path = param.request_path.clone();
        for exclude_path in self.exclude_log_path.clone() {
            if exclude_path == path {
                debug!("[Plugin.AuditLog] exclude log path matched:{}", path);
                return Ok((resp, None));
            }
        }
        trace!("[Plugin.AuditLog] exclude log path do not matched: path {}", path);

        let start_time = resp.extensions().get::<EnterTime>().map(|time| time.0);
        let end_time = Instant::now();

        let body_string = if let Some(raw_body) = resp.extensions_mut().remove::<BeforeEncryptBody>().map(|b| b.get()) {
            serde_json::from_str::<Value>(&String::from_utf8_lossy(&raw_body))
        } else {
            let body = if let Some(dumped) = resp.body().get_dumped() {
                dumped.clone()
            } else {
                let (parts, body) = resp.into_parts();
                let body = body.dump().await.map_err(|e: BoxError| TardisError::wrap(&format!("[SG.Filter.AuditLog] dump body error: {e}"), ""))?;
                resp = Response::from_parts(parts, body.dump_clone().expect(""));
                body.get_dumped().expect("not expect").clone()
            };
            serde_json::from_slice::<Value>(&body)
        };
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
        let content = LogParamContent {
            op: param.request_method,
            name: resp.extensions().get::<CertInfo>().and_then(|info| info.name.clone()).unwrap_or_default(),
            user_id: resp.extensions().get::<CertInfo>().map(|info| info.id.clone()),
            role: resp.extensions().get::<CertInfo>().map(|info| info.roles.clone()).unwrap_or_default(),
            ip: param.request_ip,
            path: param.request_path,
            scheme: param.request_scheme,
            token: param.request_headers.get(&self.header_token_name).and_then(|v| v.to_str().ok().map(|v| v.to_string())),
            server_timing: start_time.map(|st| end_time - st),
            resp_status: resp.status().as_u16().to_string(),
            success,
            own_paths: resp.extensions().get::<CertInfo>().and_then(|info| info.own_paths.clone()),
        };
        Ok((resp, Some(content)))
    }

    fn init(&mut self) -> Result<(), TardisError> {
        if let Ok(jsonpath_inst) = JsonPathInst::from_str(&self.success_json_path).map_err(|e| log::error!("[Plugin.AuditLog] invalid json path:{e}")) {
            self.jsonpath_inst = Some(jsonpath_inst);
        } else {
            self.enabled = false;
            return Err(TardisError::bad_request("[Plugin.AuditLog] plugin is not active, miss log_url or spi_app_id.", ""));
        };
        self.enabled = true;
        if self.log_url.is_empty() || self.spi_app_id.is_empty() {
            warn!("[Plugin.AuditLog] log_url or spi_app_id is empty!");
        } else {
            invoke_initializer::init(
                CODE,
                InvokeConfig {
                    spi_app_id: self.spi_app_id.clone(),
                    module_urls: HashMap::from([(InvokeModuleKind::Log.to_string(), self.log_url.clone())]),
                },
            )?;
        }

        Ok(())
    }

    fn req(&self, mut req: Request<SgBody>) -> Result<Request<SgBody>, Response<SgBody>> {
        let param = AuditLogParam {
            request_path: req.uri().path().to_string(),
            request_method: req.method().to_string(),
            request_headers: req.headers().clone(),
            request_scheme: req.uri().scheme().unwrap_or(&Scheme::HTTP).to_string(),
            request_ip: req.extensions().get::<PeerAddr>().ok_or_else(|| PluginError::internal_error::<AuditLogPlugin>("[Plugin.AuditLog] missing peer addr"))?.0.ip().to_string(),
        };

        if let Some(ident) = req.headers().get(self.head_key_auth_ident.clone()) {
            let ident = ident.to_str().unwrap_or_default().to_string();
            let reflect = req.extensions_mut().get_mut::<Reflect>().expect("missing reflect");

            if let Some(cert_info) = reflect.get_mut::<CertInfo>() {
                cert_info.id = ident;
            } else {
                reflect.insert(CertInfo {
                    id: ident,
                    own_paths: None,
                    name: None,
                    roles: vec![],
                });
            }
        };
        let reflect = req.extensions_mut().get_mut::<Reflect>().expect("missing reflect");
        reflect.insert(param);
        Ok(req)
    }

    async fn resp(&self, resp: Response<SgBody>) -> Result<Response<SgBody>, Response<SgBody>> {
        if self.enabled {
            let (resp, content) = self.get_log_content(resp).await.map_err(PluginError::internal_error::<AuditLogPlugin>)?;

            if let Some(content) = content {
                let funs = get_tardis_inst();

                let spi_ctx = TardisContext {
                    owner: resp.extensions().get::<CertInfo>().map(|info| info.id.clone()).unwrap_or_default(),
                    roles: resp.extensions().get::<CertInfo>().map(|info| info.roles.clone().into_iter().map(|r| r.id).collect()).unwrap_or_default(),
                    ..Default::default()
                };

                let tag = self.tag.clone();
                if !self.log_url.is_empty() && !self.spi_app_id.is_empty() {
                    tokio::task::spawn(async move {
                        match spi_log_client::SpiLogClient::addv2(
                            LogItemAddV2Req {
                                tag,
                                content: TardisFuns::json.obj_to_json(&content).unwrap_or_default(),
                                kind: None,
                                ext: Some(content.to_value()),
                                key: None,
                                op: Some(content.op),
                                rel_key: None,
                                idempotent_id: None,
                                ts: Some(tardis::chrono::Utc::now()),
                                owner: content.user_id,
                                own_paths: None,
                                msg: None,
                                owner_name: None,
                                push: false,
                            },
                            &funs,
                            &spi_ctx,
                        )
                        .await
                        {
                            Ok(_) => {
                                log::trace!("[Plugin.AuditLog] add log success")
                            }
                            Err(e) => {
                                log::warn!("[Plugin.AuditLog] failed to add log:{e}")
                            }
                        };
                    });
                }
            }

            Ok(resp)
        } else {
            Ok(resp)
        }
    }
}

impl Default for AuditLogPlugin {
    fn default() -> Self {
        Self {
            log_url: "".to_string(),
            spi_app_id: "".to_string(),
            tag: "gateways".to_string(),
            header_token_name: "Bios-Token".to_string(),
            success_json_path: "$.code".to_string(),
            enabled: false,
            success_json_path_values: vec!["200".to_string(), "201".to_string()],
            exclude_log_path: vec!["/starsysApi/apis".to_string()],
            jsonpath_inst: None,
            head_key_auth_ident: "Iam-Auth-Ident".to_string(),
        }
    }
}

impl Plugin for AuditLogPlugin {
    const CODE: &'static str = CODE;

    fn meta() -> spacegate_shell::plugin::PluginMetaData {
        spacegate_shell::plugin::plugin_meta!(
            description: "Audit log for spacegate, it's base on spi-log"
        )
    }

    fn create(config: spacegate_shell::plugin::PluginConfig) -> Result<Self, BoxError> {
        let mut plugin: AuditLogPlugin = serde_json::from_value(config.spec.clone()).map_err(|e| -> BoxError { format!("[Plugin.AuditLog] deserialize error:{e}").into() })?;
        plugin.init()?;
        Ok(plugin)
    }
    async fn call(&self, req: Request<SgBody>, inner: Inner) -> Result<Response<SgBody>, BoxError> {
        match self.req(req) {
            Ok(req) => {
                let resp = inner.call(req).await;
                match self.resp(resp).await {
                    Ok(resp) => Ok(resp),
                    Err(e) => Ok(e),
                }
            }
            Err(resp) => Ok(resp),
        }
    }
}

fn get_tardis_inst() -> TardisFunsInst {
    TardisFuns::inst(CODE.to_string(), None)
}

impl LogParamContent {
    fn to_value(&self) -> Value {
        json!({
            "name":self.name,
            "id":self.user_id,
            "own_paths":self.own_paths,
            "ip":self.ip,
            "op":self.op,
            "path":self.path,
            "resp_status": self.resp_status,
            "server_timing":self.server_timing,
            "success":self.success,
        })
    }
}
#[cfg(test)]
mod test {
    use http::{HeaderName, Request, Response};
    use spacegate_shell::{
        kernel::extension::{EnterTime, PeerAddr, Reflect},
        SgBody,
    };
    use tardis::tokio;

    use super::AuditLogPlugin;

    #[tokio::test]
    async fn test_log_content() {
        let ent_time = std::time::Instant::now();
        println!("test_log_content");
        let mut sg_filter_audit_log = AuditLogPlugin {
            log_url: "xxx".to_string(),
            spi_app_id: "xxx".to_string(),
            exclude_log_path: vec!["/api/test".to_string(), "/cc/api/test/file".to_string()],
            ..Default::default()
        };
        sg_filter_audit_log.init().unwrap();
        let guard = pprof::ProfilerGuardBuilder::default().frequency(100).blocklist(&["libc", "libgcc", "pthread", "vdso"]).build().unwrap();
        let mut count = 0;
        loop {
            if count == 200000 {
                break;
            }
            count += 1;

            let mut req_ref = Reflect::new();
            req_ref.insert(EnterTime::new());
            let mut req = Request::builder()
                .method("GET")
                .header(sg_filter_audit_log.header_token_name.parse::<HeaderName>().unwrap(), "aaa")
                .uri("http://idealworld.com/test1")
                .extension(req_ref)
                .extension(PeerAddr("127.0.0.1:8080".parse().unwrap()))
                .body(SgBody::full(r#"{"data":"xxxxx"}"#))
                .unwrap();
            req = sg_filter_audit_log.req(req).unwrap();
            let mut resp = Response::builder().body(SgBody::full(r#"{"code":"200","msg":"success"}"#)).unwrap();
            resp.extensions_mut().extend(req.extensions_mut().remove::<Reflect>().unwrap().into_inner());
            let log_content = sg_filter_audit_log.get_log_content(resp).await.unwrap();
            assert!(log_content.1.is_some());
            let log_content = log_content.1.unwrap();
            assert_eq!(log_content.token, Some("aaa".to_string()));
            assert!(log_content.server_timing.is_some());
            assert!(log_content.success);

            let mut req_ref = Reflect::new();
            req_ref.insert(EnterTime::new());
            let mut req = Request::builder()
                .method("GET")
                .header(sg_filter_audit_log.header_token_name.parse::<HeaderName>().unwrap(), "aaa")
                .uri("http://idealworld.com/cc/api/test/file")
                .extension(req_ref)
                .extension(PeerAddr("127.0.0.1:8080".parse().unwrap()))
                .body(SgBody::full(r#"{"data":"xxxxx"}"#))
                .unwrap();
            req = sg_filter_audit_log.req(req).unwrap();
            let mut resp = Response::builder().body(SgBody::full(r#"{"code":"200","msg":"success"}"#)).unwrap();
            resp.extensions_mut().extend(req.extensions_mut().remove::<Reflect>().unwrap().into_inner());
            let log_content = sg_filter_audit_log.get_log_content(resp).await.unwrap();
            assert!(log_content.1.is_none());
        }
        if let Ok(report) = guard.report().build() {
            let file = std::fs::File::create("flamegraph.svg").unwrap();
            report.flamegraph(file).unwrap();
        };
        let exit_time = std::time::Instant::now();
        let time = exit_time.duration_since(ent_time);
        println!("test_log_content time:{:?}", time);
    }
}
