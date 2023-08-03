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
use tardis::serde_json::{json, Value};

use tardis::{
    async_trait,
    basic::result::TardisResult,
    log,
    serde_json::{self},
    tokio::{self},
    TardisFuns, TardisFunsInst,
};

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
    enabled: bool,
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
            if JsonPathInst::from_str(&self.success_json_path).map_err(|e| log::error!("[[Plugin.AuditLog]] invalid json path:{e}")).is_err() {
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
            let funs = get_tardis_inst();
            let start_time = ctx.get_ext(&get_start_time_ext_code()).and_then(|time| time.parse::<i64>().ok());
            let end_time = tardis::chrono::Utc::now().timestamp_millis();
            let spi_ctx = TardisContext {
                owner: ctx.get_cert_info().map(|info| info.account_id.clone()).unwrap_or_default(),
                roles: ctx.get_cert_info().map(|info| info.roles.clone().into_iter().map(|r| r.id).collect()).unwrap_or_default(),
                ..Default::default()
            };
            let op = ctx.request.get_req_method().to_string();
            let resp_body = ctx.response.pop_resp_body().await?;
            let success = match resp_body {
                Some(body) => {
                    let body_string = String::from_utf8_lossy(&body).to_string();
                    let result = match serde_json::from_str::<Value>(&body_string) {
                        Ok(json) => {
                            if let Ok(matching_value) = json.path(&self.success_json_path) {
                                if matching_value.is_number() && matching_value.is_string() {
                                    let mut is_match = false;
                                    for value in self.success_json_path_values.clone() {
                                        if value == matching_value {
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
                        }
                        Err(_) => false,
                    };
                    ctx.response.set_resp_body(body)?;
                    result
                }
                None => false,
            };
            let content = LogParamContent {
                op: op.clone(),
                key: None,
                name: ctx.get_cert_info().and_then(|info| info.account_name.clone()).unwrap_or_default(),
                user_id: ctx.get_cert_info().map(|info| info.account_id.clone()),
                role: ctx.get_cert_info().map(|info| info.roles.clone()).unwrap_or_default(),
                ip: ctx.request.get_req_remote_addr().ip().to_string(),
                token: ctx.request.get_req_headers().get(&self.header_token_name).and_then(|v| v.to_str().ok().map(|v| v.to_string())),
                server_timing: start_time.map(|st| end_time - st),
                success,
            };
            let log_ext = json!({
                "name":content.name,
                "id":content.user_id,
                "ip":content.ip,
                "op":op.clone(),
                "success":content.success,
            });
            tokio::spawn(async move {
                match spi_log_client::SpiLogClient::add(
                    "tag",
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
                        log::trace!("[Plugin.AuditLog] failed to add log:{e}")
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
    pub token: Option<String>,
    pub server_timing: Option<i64>,
    //Indicates whether the business operation was successful.
    pub success: bool,
}
