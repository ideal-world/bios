use bios_sdk_invoke::clients::spi_log_client::{self, LogItemAddV2Req};
use http::uri::Scheme;
use serde::{Deserialize, Serialize};
use spacegate_shell::{
    hyper::HeaderMap,
    kernel::{
        extension::{EnterTime, OriginalIpAddr, PeerAddr},
        Extract,
    },
    SgRequestExt, SgResponse,
};
use std::time::Duration;
use tardis::{basic::dto::TardisContext, TardisFuns, TardisFunsInst};
use tardis::{log as tracing, tokio};

use crate::{audit_log::AuditLogPlugin, plugin::PluginBiosExt};

use super::cert_info::{CertInfo, RoleInfo};

#[derive(Debug, Clone)]
pub struct AuditLogParam {
    pub request_path: String,
    pub request_method: String,
    pub request_headers: HeaderMap,
    pub request_scheme: String,
    pub request_ip: String,
}

impl Extract for AuditLogParam {
    fn extract(req: &http::Request<spacegate_shell::SgBody>) -> Self {
        AuditLogParam {
            request_path: req.uri().path().to_string(),
            request_method: req.method().to_string(),
            request_headers: req.headers().clone(),
            request_scheme: req.uri().scheme().unwrap_or(&Scheme::HTTP).to_string(),
            request_ip: req.extract::<OriginalIpAddr>().to_string(),
        }
    }
}

impl AuditLogParam {
    pub fn merge_audit_log_param_content(self, response: &SgResponse, success: bool, header_token_name: &str) -> LogParamContent {
        let cert_info = response.extensions().get::<CertInfo>();
        let start_time = response.extensions().get::<EnterTime>().map(|time| time.0);
        let end_time = std::time::Instant::now();
        let param = self;
        LogParamContent {
            op: param.request_method,
            name: cert_info.and_then(|info| info.name.clone()).unwrap_or_default(),
            user_id: cert_info.map(|info| info.id.clone()),
            role: cert_info.map(|info| info.roles.clone()).unwrap_or_default(),
            ip: param.request_ip,
            path: param.request_path,
            scheme: param.request_scheme,
            token: param.request_headers.get(header_token_name).and_then(|v| v.to_str().ok().map(|v| v.to_string())),
            server_timing: start_time.map(|st| end_time - st),
            resp_status: response.status().as_u16().to_string(),
            success,
            own_paths: cert_info.and_then(|info| info.own_paths.clone()),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LogParamContent {
    pub op: String,
    pub name: String,
    pub user_id: Option<String>,
    pub own_paths: Option<String>,
    pub role: Vec<RoleInfo>,
    pub ip: String,
    pub path: String,
    pub scheme: String,
    pub token: Option<String>,
    pub server_timing: Option<Duration>,
    pub resp_status: String,
    //Indicates whether the business operation was successful.
    pub success: bool,
}

impl LogParamContent {
    pub fn send_audit_log<P: PluginBiosExt>(self, spi_app_id: &str, log_url: &str, tag: &str) {
        send_audit_log(spi_app_id, log_url, tag, self, P::get_funs_inst_by_plugin_code());
    }
}

fn send_audit_log(spi_app_id: &str, log_url: &str, tag: &str, content: LogParamContent, funs: TardisFunsInst) {
    let spi_ctx = TardisContext {
        ak: spi_app_id.to_string(),
        own_paths: spi_app_id.to_string(),
        ..Default::default()
    };

    let tag = tag.to_string();
    if !log_url.is_empty() && !spi_app_id.is_empty() {
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
                    data_source: None,
                    ignore_push: None,
                    push: false,
                    disable: None,
                },
                &funs,
                &spi_ctx,
            )
            .await
            {
                Ok(_) => {
                    tracing::debug!("[Plugin.AuditLog] add log success")
                }
                Err(e) => {
                    tracing::warn!("[Plugin.AuditLog] failed to add log:{e}")
                }
            };
        });
    }
}
