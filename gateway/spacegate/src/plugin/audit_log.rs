use std::collections::HashMap;
use std::str::FromStr;

use async_trait::async_trait;

use bios_sdk_invoke::clients::spi_log_client;
use bios_sdk_invoke::invoke_config::InvokeConfig;
use bios_sdk_invoke::invoke_enumeration::InvokeModuleKind;
use bios_sdk_invoke::invoke_initializer;

use jsonpath_rust::{JsonPathInst, JsonPathQuery};
use serde::{Deserialize, Serialize};
use spacegate_kernel::plugins::context::SGRoleInfo;
use spacegate_kernel::plugins::{
    context::SgRoutePluginContext,
    filters::{BoxSgPluginFilter, SgPluginFilter, SgPluginFilterAccept, SgPluginFilterDef, SgPluginFilterInitDto},
};
use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::serde_json::{json, Value};

use tardis::{
    async_trait,
    basic::result::TardisResult,
    log,
    serde_json::{self},
    tokio::{self},
    TardisFuns, TardisFunsInst,
};

use super::plugin_constants;

pub const CODE: &str = "audit_log";
pub struct SgFilterAuditLogDef;

impl SgPluginFilterDef for SgFilterAuditLogDef {
    fn inst(&self, spec: serde_json::Value) -> TardisResult<BoxSgPluginFilter> {
        let filter = TardisFuns::json.json_to_obj::<SgFilterAuditLog>(spec)?;
        Ok(filter.boxed())
    }
}

#[derive(Serialize, Deserialize)]
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
}

impl SgFilterAuditLog {
    async fn get_log_content(&self, end_time: i64, ctx: &mut SgRoutePluginContext) -> TardisResult<LogParamContent> {
        let start_time = ctx.get_ext(&get_start_time_ext_code()).and_then(|time| time.parse::<i64>().ok());
        let body_string = if let Some(raw_body) = ctx.get_ext(plugin_constants::BEFORE_ENCRYPT_BODY) {
            Some(raw_body)
        } else {
            ctx.response
                .pop_body()
                .await?
                .map(|body| {
                    let body_string = String::from_utf8_lossy(&body).to_string();
                    ctx.response.set_body(body)?;
                    Ok::<_, TardisError>(body_string)
                })
                .transpose()?
        };
        let success = match serde_json::from_str::<Value>(&body_string.unwrap_or_default()) {
            Ok(json) => {
                if let Ok(matching_value) = json.path(&self.success_json_path) {
                    if let Some(matching_value) = matching_value.as_array() {
                        let matching_value = &matching_value[0];
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
        Ok(LogParamContent {
            op: ctx.request.get_method().to_string(),
            key: None,
            name: ctx.get_cert_info().and_then(|info| info.name.clone()).unwrap_or_default(),
            user_id: ctx.get_cert_info().map(|info| info.id.clone()),
            role: ctx.get_cert_info().map(|info| info.roles.clone()).unwrap_or_default(),
            ip: if let Some(real_ips) = ctx.request.get_headers().get("X-Forwarded-For") {
                real_ips.to_str().ok().and_then(|ips| ips.split(',').collect::<Vec<_>>().first().map(|ip| ip.to_string())).unwrap_or(ctx.request.get_remote_addr().ip().to_string())
            } else {
                ctx.request.get_remote_addr().ip().to_string()
            },
            path: ctx.request.get_uri_raw().path().to_string(),
            scheme: ctx.request.get_uri_raw().scheme_str().unwrap_or("http").to_string(),
            token: ctx.request.get_headers().get(&self.header_token_name).and_then(|v| v.to_str().ok().map(|v| v.to_string())),
            server_timing: start_time.map(|st| end_time - st),
            resp_status: ctx.response.get_status_code().as_u16().to_string(),
            success,
        })
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
        }
    }
}

#[async_trait]
impl SgPluginFilter for SgFilterAuditLog {
    fn accept(&self) -> SgPluginFilterAccept {
        SgPluginFilterAccept {
            accept_error_response: true,
            ..Default::default()
        }
    }

    async fn init(&mut self, _: &SgPluginFilterInitDto) -> TardisResult<()> {
        if !self.log_url.is_empty() && !self.spi_app_id.is_empty() {
            if JsonPathInst::from_str(&self.success_json_path).map_err(|e| log::error!("[Plugin.AuditLog] invalid json path:{e}")).is_err() {
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
        } else {
            log::warn!("[Plugin.AuditLog] plugin is not active, miss log_url or spi_app_id.");
            self.enabled = false;
        }
        Ok(())
    }

    async fn destroy(&self) -> TardisResult<()> {
        Ok(())
    }

    async fn req_filter(&self, _: &str, mut ctx: SgRoutePluginContext) -> TardisResult<(bool, SgRoutePluginContext)> {
        ctx.set_ext(&get_start_time_ext_code(), &tardis::chrono::Utc::now().timestamp_millis().to_string());
        return Ok((true, ctx));
    }

    async fn resp_filter(&self, _: &str, mut ctx: SgRoutePluginContext) -> TardisResult<(bool, SgRoutePluginContext)> {
        if self.enabled {
            let path = ctx.request.get_uri_raw().path().to_string();
            for exclude_path in self.exclude_log_path.clone() {
                if exclude_path == path {
                    return Ok((true, ctx));
                }
            }
            let funs = get_tardis_inst();
            let end_time = tardis::chrono::Utc::now().timestamp_millis();
            let spi_ctx = TardisContext {
                owner: ctx.get_cert_info().map(|info| info.id.clone()).unwrap_or_default(),
                roles: ctx.get_cert_info().map(|info| info.roles.clone().into_iter().map(|r| r.id).collect()).unwrap_or_default(),
                ..Default::default()
            };
            let op = ctx.request.get_method().to_string();

            let content = self.get_log_content(end_time, &mut ctx).await?;

            let log_ext = json!({
                "name":content.name,
                "id":content.user_id,
                "ip":content.ip,
                "op":op.clone(),
                "path":content.path,
                "resp_status": content.resp_status,
                "success":content.success,
            });
            let tag = self.tag.clone();
            tokio::spawn(async move {
                match spi_log_client::SpiLogClient::add(
                    &tag,
                    &TardisFuns::json.obj_to_string(&content).unwrap_or_default(),
                    Some(log_ext),
                    None,
                    None,
                    Some(op),
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

            Ok((true, ctx))
        } else {
            Ok((true, ctx))
        }
    }
}

fn get_tardis_inst() -> TardisFunsInst {
    TardisFuns::inst(CODE.to_string(), None)
}

fn get_start_time_ext_code() -> String {
    format!("{CODE}:start_time")
}

#[derive(Serialize, Deserialize)]
pub struct LogParamContent {
    pub op: String,
    pub key: Option<String>,
    pub name: String,
    pub user_id: Option<String>,
    pub role: Vec<SGRoleInfo>,
    pub ip: String,
    pub path: String,
    pub scheme: String,
    pub token: Option<String>,
    pub server_timing: Option<i64>,
    pub resp_status: String,
    //Indicates whether the business operation was successful.
    pub success: bool,
}

#[cfg(test)]
mod test {
    use spacegate_kernel::{
        http::{HeaderName, Uri},
        hyper::{Body, HeaderMap, Method, StatusCode, Version},
        plugins::context::SgRoutePluginContext,
    };
    use tardis::tokio;

    use crate::plugin::audit_log::get_start_time_ext_code;

    use super::SgFilterAuditLog;

    #[tokio::test]
    async fn test_log_content() {
        let sg_filter_audit_log = SgFilterAuditLog { ..Default::default() };
        let end_time = 20100;
        let mut header = HeaderMap::new();
        header.insert(sg_filter_audit_log.header_token_name.parse::<HeaderName>().unwrap(), "aaa".parse().unwrap());
        let mut ctx = SgRoutePluginContext::new_http(
            Method::POST,
            Uri::from_static("http://sg.idealworld.group/test1"),
            Version::HTTP_11,
            header,
            Body::from(""),
            "127.0.0.1:8080".parse().unwrap(),
            "".to_string(),
            None,
        );
        ctx.set_ext(&get_start_time_ext_code(), &20000.to_string());
        let mut ctx = ctx.resp(StatusCode::OK, HeaderMap::new(), Body::from(r##"{"code":"200","msg":"success"}"##));
        let log_content = sg_filter_audit_log.get_log_content(end_time, &mut ctx).await.unwrap();
        assert_eq!(log_content.token, Some("aaa".to_string()));
        assert_eq!(log_content.server_timing, Some(100));
        assert!(log_content.success);

        let mut header = HeaderMap::new();
        header.insert(sg_filter_audit_log.header_token_name.parse::<HeaderName>().unwrap(), "aaa".parse().unwrap());
        let ctx = SgRoutePluginContext::new_http(
            Method::POST,
            Uri::from_static("http://sg.idealworld.group/test1"),
            Version::HTTP_11,
            header,
            Body::from(""),
            "127.0.0.1:8080".parse().unwrap(),
            "".to_string(),
            None,
        );
        let mut ctx = ctx.resp(StatusCode::OK, HeaderMap::new(), Body::from(r##"{"code":200,"msg":"success"}"##));
        let log_content = sg_filter_audit_log.get_log_content(end_time, &mut ctx).await.unwrap();
        assert!(log_content.success);

        let mut header = HeaderMap::new();
        header.insert(sg_filter_audit_log.header_token_name.parse::<HeaderName>().unwrap(), "aaa".parse().unwrap());
        let ctx = SgRoutePluginContext::new_http(
            Method::POST,
            Uri::from_static("http://sg.idealworld.group/test1"),
            Version::HTTP_11,
            header,
            Body::from(""),
            "127.0.0.1:8080".parse().unwrap(),
            "".to_string(),
            None,
        );
        let mut ctx = ctx.resp(StatusCode::OK, HeaderMap::new(), Body::from(r##"{"code":"500","msg":"not success"}"##));
        let log_content = sg_filter_audit_log.get_log_content(end_time, &mut ctx).await.unwrap();
        assert!(!log_content.success);

        let mut header = HeaderMap::new();
        header.insert(sg_filter_audit_log.header_token_name.parse::<HeaderName>().unwrap(), "aaa".parse().unwrap());
        let ctx = SgRoutePluginContext::new_http(
            Method::POST,
            Uri::from_static("http://sg.idealworld.group/test1"),
            Version::HTTP_11,
            header,
            Body::from(""),
            "127.0.0.1:8080".parse().unwrap(),
            "".to_string(),
            None,
        );
        let mut ctx = ctx.resp(StatusCode::OK, HeaderMap::new(), Body::from(r##"{"code":500,"msg":"not success"}"##));
        let log_content = sg_filter_audit_log.get_log_content(end_time, &mut ctx).await.unwrap();
        assert!(!log_content.success);
    }
}
