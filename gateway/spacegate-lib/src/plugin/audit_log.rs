use std::collections::HashMap;

use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;

use bios_sdk_invoke::clients::spi_log_client;
use bios_sdk_invoke::invoke_config::InvokeConfig;
use bios_sdk_invoke::invoke_enumeration::InvokeModuleKind;
use bios_sdk_invoke::invoke_initializer;

use http::uri::Scheme;
use jsonpath_rust::JsonPathInst;
use serde::{Deserialize, Serialize};
use spacegate_shell::hyper::{Request, Response};
use spacegate_shell::kernel::extension::{EnterTime, PeerAddr, Reflect};

use spacegate_shell::kernel::helper_layers::bidirection_filter::{Bdf, BdfLayer, BoxRespFut};
use spacegate_shell::plugin::{JsonValue, MakeSgLayer, Plugin, PluginError};
use spacegate_shell::{BoxError, SgBody};
use tardis::basic::dto::TardisContext;
use tardis::log::warn;
use tardis::serde_json::{json, Value};

use tardis::basic::error::TardisError;
use tardis::{
    basic::result::TardisResult,
    log,
    serde_json::{self},
    tokio::{self},
    TardisFuns, TardisFunsInst,
};

use crate::extension::audit_log_param::AuditLogParam;
use crate::extension::before_encrypt_body::BeforeEncryptBody;
use crate::extension::cert_info::{CertInfo, RoleInfo};

pub const CODE: &str = "audit_log";

#[cfg(feature = "schema")]
use spacegate_plugin::schemars;
#[cfg(feature = "schema")]
spacegate_plugin::schema!(AuditLogPlugin, SgFilterAuditLog);

#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default)]
pub struct SgFilterAuditLog {
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

impl SgFilterAuditLog {
    async fn get_log_content(&self, mut resp: Response<SgBody>, param: AuditLogParam) -> TardisResult<(Response<SgBody>, LogParamContent)> {
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
            server_timing: start_time.map(|st| (end_time - st).as_millis() as i64),
            resp_status: resp.status().as_u16().to_string(),
            success,
        };
        Ok((resp, content))
    }

    fn init(&mut self) -> Result<(), TardisError> {
        if !self.log_url.is_empty() && !self.spi_app_id.is_empty() {
            if let Ok(jsonpath_inst) = JsonPathInst::from_str(&self.success_json_path).map_err(|e| log::error!("[Plugin.AuditLog] invalid json path:{e}")) {
                self.jsonpath_inst = Some(jsonpath_inst);
            } else {
                self.enabled = false;
                return Ok(());
            };
            self.enabled = true;
            invoke_initializer::init(
                CODE,
                InvokeConfig {
                    spi_app_id: self.spi_app_id.clone(),
                    module_urls: HashMap::from([(InvokeModuleKind::Log.to_string(), self.log_url.clone())]),
                },
            )?;
            Ok(())
        } else {
            self.enabled = false;
            Err(TardisError::bad_request("[Plugin.AuditLog] plugin is not active, miss log_url or spi_app_id.", ""))
        }
    }

    fn req(&self, mut req: Request<SgBody>) -> Result<Request<SgBody>, Response<SgBody>> {
        let param = AuditLogParam {
            request_path: req.uri().path().to_string(),
            request_method: req.method().to_string(),
            request_headers: req.headers().clone(),
            request_scheme: req.uri().scheme().unwrap_or(&Scheme::HTTP).to_string(),
            request_ip: req.extensions().get::<PeerAddr>().ok_or(PluginError::internal_error::<AuditLogPlugin>("[Plugin.AuditLog] missing peer addr"))?.0.ip().to_string(),
        };

        if let Some(ident) = req.headers().get(self.head_key_auth_ident.clone()) {
            let ident = ident.to_str().unwrap_or_default().to_string();
            let reflect = req.extensions_mut().get_mut::<Reflect>().expect("missing reflect");

            if let Some(cert_info) = reflect.get_mut::<CertInfo>() {
                cert_info.id = ident;
            } else {
                reflect.insert(CertInfo {
                    id: ident,
                    name: None,
                    roles: vec![],
                });
            }
        };
        let reflect = req.extensions_mut().get_mut::<Reflect>().expect("missing reflect");
        reflect.insert(param);
        Ok(req)
    }

    async fn resp(&self, mut resp: Response<SgBody>) -> Result<Response<SgBody>, Response<SgBody>> {
        let Some(audit_param) = resp.extensions_mut().remove::<AuditLogParam>() else {
            warn!("[Plugin.AuditLog] missing audit log param");
            return Ok(resp);
        };
        if self.enabled {
            let path = audit_param.request_path.clone();
            for exclude_path in self.exclude_log_path.clone() {
                if exclude_path == path {
                    return Ok(resp);
                }
            }
            let funs = get_tardis_inst();
            let _end_time = tardis::chrono::Utc::now().timestamp_millis();

            let spi_ctx = TardisContext {
                owner: resp.extensions().get::<CertInfo>().map(|info| info.id.clone()).unwrap_or_default(),
                roles: resp.extensions().get::<CertInfo>().map(|info| info.roles.clone().into_iter().map(|r| r.id).collect()).unwrap_or_default(),
                ..Default::default()
            };

            let (resp, content) = self.get_log_content(resp, audit_param).await.map_err(PluginError::internal_error::<AuditLogPlugin>)?;

            let tag = self.tag.clone();
            tokio::task::spawn(async move {
                match spi_log_client::SpiLogClient::add(
                    &tag,
                    &TardisFuns::json.obj_to_string(&content).unwrap_or_default(),
                    Some(content.to_value()),
                    None,
                    None,
                    Some(content.op),
                    None,
                    Some(tardis::chrono::Utc::now().to_rfc3339()),
                    content.user_id,
                    None,
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

            Ok(resp)
        } else {
            Ok(resp)
        }
    }
}

impl Default for SgFilterAuditLog {
    fn default() -> Self {
        Self {
            log_url: "".to_string(),
            spi_app_id: "".to_string(),
            tag: "gateway".to_string(),
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

impl Bdf for SgFilterAuditLog {
    type FutureReq = std::future::Ready<Result<Request<SgBody>, Response<SgBody>>>;

    type FutureResp = BoxRespFut;

    fn on_req(self: Arc<Self>, req: Request<SgBody>) -> Self::FutureReq {
        std::future::ready(self.req(req))
    }

    fn on_resp(self: Arc<Self>, resp: Response<SgBody>) -> Self::FutureResp {
        Box::pin(async move {
            match self.resp(resp).await {
                Ok(resp) => resp,
                Err(e) => e,
            }
        })
    }
}

impl MakeSgLayer for SgFilterAuditLog {
    fn make_layer(&self) -> Result<spacegate_shell::SgBoxLayer, spacegate_shell::BoxError> {
        let layer = BdfLayer::new(self.clone());
        Ok(spacegate_shell::SgBoxLayer::new(layer))
    }
}

pub struct AuditLogPlugin;

impl Plugin for AuditLogPlugin {
    type MakeLayer = SgFilterAuditLog;
    const CODE: &'static str = CODE;
    fn create(_: Option<String>, value: JsonValue) -> Result<Self::MakeLayer, BoxError> {
        let mut plugin: SgFilterAuditLog = serde_json::from_value(value).map_err(|e| -> BoxError { format!("[Plugin.AuditLog] deserialize error:{e}").into() })?;
        plugin.init()?;
        Ok(plugin)
    }
}

fn get_tardis_inst() -> TardisFunsInst {
    TardisFuns::inst(CODE.to_string(), None)
}

#[derive(Serialize, Deserialize)]
pub struct LogParamContent {
    pub op: String,
    pub name: String,
    pub user_id: Option<String>,
    pub role: Vec<RoleInfo>,
    pub ip: String,
    pub path: String,
    pub scheme: String,
    pub token: Option<String>,
    pub server_timing: Option<i64>,
    pub resp_status: String,
    //Indicates whether the business operation was successful.
    pub success: bool,
}

impl LogParamContent {
    fn to_value(&self) -> Value {
        json!({
            "name":self.name,
            "id":self.user_id,
            "ip":self.ip,
            "op":self.op,
            "path":self.path,
            "resp_status": self.resp_status,
            "success":self.success,
        })
    }
}

// #[cfg(test)]
// mod test {
//     use spacegate_shell::plugins::filters::{SgAttachedLevel, SgPluginFilter, SgPluginFilterInitDto};
//     use spacegate_shell::{
//         http::{HeaderName, Uri},
//         hyper::{Body, HeaderMap, Method, StatusCode, Version},
//         plugins::context::SgRoutePluginContext,
//     };
//     use tardis::tokio;

//     use crate::plugin::audit_log::get_start_time_ext_code;

//     use super::SgFilterAuditLog;

//     #[tokio::test]
//     async fn test_log_content() {
//         let ent_time = std::time::Instant::now();
//         println!("test_log_content");
//         let mut sg_filter_audit_log = SgFilterAuditLog {
//             log_url: "xxx".to_string(),
//             spi_app_id: "xxx".to_string(),
//             ..Default::default()
//         };
//         sg_filter_audit_log
//             .init(&SgPluginFilterInitDto {
//                 gateway_name: "".to_string(),
//                 gateway_parameters: Default::default(),
//                 http_route_rules: vec![],
//                 attached_level: SgAttachedLevel::Gateway,
//             })
//             .await
//             .unwrap();
//         let guard = pprof::ProfilerGuardBuilder::default().frequency(100).blocklist(&["libc", "libgcc", "pthread", "vdso"]).build().unwrap();
//         let end_time = 20100;
//         let mut count = 0;
//         loop {
//             if count == 200000 {
//                 break;
//             }
//             count += 1;
//             let mut header = HeaderMap::new();
//             header.insert(sg_filter_audit_log.header_token_name.parse::<HeaderName>().unwrap(), "aaa".parse().unwrap());
//             let mut ctx = SgRoutePluginContext::new_http(
//                 Method::POST,
//                 Uri::from_static("http://sg.idealworld.group/test1"),
//                 Version::HTTP_11,
//                 header,
//                 Body::from(""),
//                 "127.0.0.1:8080".parse().unwrap(),
//                 "".to_string(),
//                 None,
//             );
//             ctx.set_ext(&get_start_time_ext_code(), &20000.to_string());
//             let mut ctx = ctx.resp(StatusCode::OK, HeaderMap::new(), Body::from(r#"{"code":"200","msg":"success"}"#));
//             let log_content = sg_filter_audit_log.get_log_content(end_time, &mut ctx).await.unwrap();
//             assert_eq!(log_content.token, Some("aaa".to_string()));
//             assert_eq!(log_content.server_timing, Some(100));
//             assert!(log_content.success);

//             let mut header = HeaderMap::new();
//             header.insert(sg_filter_audit_log.header_token_name.parse::<HeaderName>().unwrap(), "aaa".parse().unwrap());
//             let ctx = SgRoutePluginContext::new_http(
//                 Method::POST,
//                 Uri::from_static("http://sg.idealworld.group/test1"),
//                 Version::HTTP_11,
//                 header,
//                 Body::from(""),
//                 "127.0.0.1:8080".parse().unwrap(),
//                 "".to_string(),
//                 None,
//             );
//             let mut ctx = ctx.resp(StatusCode::OK, HeaderMap::new(), Body::from(r#"{"code":200,"msg":"success"}"#));
//             let log_content = sg_filter_audit_log.get_log_content(end_time, &mut ctx).await.unwrap();
//             assert!(log_content.success);

//             let mut header = HeaderMap::new();
//             header.insert(sg_filter_audit_log.header_token_name.parse::<HeaderName>().unwrap(), "aaa".parse().unwrap());
//             let ctx = SgRoutePluginContext::new_http(
//                 Method::POST,
//                 Uri::from_static("http://sg.idealworld.group/test1"),
//                 Version::HTTP_11,
//                 header,
//                 Body::from(""),
//                 "127.0.0.1:8080".parse().unwrap(),
//                 "".to_string(),
//                 None,
//             );
//             let mut ctx = ctx.resp(StatusCode::OK, HeaderMap::new(), Body::from(r#"{"code":"500","msg":"not success"}"#));
//             let log_content = sg_filter_audit_log.get_log_content(end_time, &mut ctx).await.unwrap();
//             assert!(!log_content.success);

//             let mut header = HeaderMap::new();
//             header.insert(sg_filter_audit_log.header_token_name.parse::<HeaderName>().unwrap(), "aaa".parse().unwrap());
//             let ctx = SgRoutePluginContext::new_http(
//                 Method::POST,
//                 Uri::from_static("http://sg.idealworld.group/test1"),
//                 Version::HTTP_11,
//                 header,
//                 Body::from(""),
//                 "127.0.0.1:8080".parse().unwrap(),
//                 "".to_string(),
//                 None,
//             );
//             let mut ctx = ctx.resp(StatusCode::OK, HeaderMap::new(), Body::from(r#"{"code":500,"msg":"not success"}"#));
//             let log_content = sg_filter_audit_log.get_log_content(end_time, &mut ctx).await.unwrap();
//             assert!(!log_content.success);
//         }
//         if let Ok(report) = guard.report().build() {
//             let file = std::fs::File::create("flamegraph.svg").unwrap();
//             report.flamegraph(file).unwrap();
//         };
//         let exit_time = std::time::Instant::now();
//         let time = exit_time.duration_since(ent_time);
//         println!("test_log_content time:{:?}", time);
//     }
// }
