use std::{collections::HashMap, sync::Arc, time::Duration};

use serde::{Deserialize, Serialize};
use tardis::{basic::dto::TardisContext, log as tracing, tokio};

/// Context to call notification api
///
/// Extract it from request extensions, and call [`NotificationContext::notify`] to send notification
#[derive(Debug, Clone)]
pub struct NotificationContext {
    pub(crate) reach_api: Arc<str>,
    pub(crate) log_api: Arc<str>,
    pub(crate) spi_app_id: Arc<str>,
    pub(crate) audit_log_tag: Arc<str>,
    pub(crate) audit_log_token_header_name: Arc<str>,
    pub(crate) templates: Arc<HashMap<String, NotifyPluginConfigTemplatesItem>>,
    pub(crate) audit_param: Arc<AuditLogParam>,
    pub(crate) cache_client: RedisClient,
    pub(crate) dedup_cache_cool_down: Duration,
}

impl NotificationContext {
    pub fn submit_notify(&self, req: ReachMsgSendReq, dedup_hash: u64) {
        if self.reach_api.is_empty() {
            tracing::debug!("reach api is empty, skip sending notification");
            return;
        }
        let cache_client = self.cache_client.clone();
        let ctx = TardisContext {
            ak: self.spi_app_id.to_string(),
            own_paths: self.spi_app_id.to_string(),
            ..Default::default()
        };
        let cool_down = (self.dedup_cache_cool_down.as_secs() as u64).min(1);
        tokio::spawn(async move {
            let key = format!("sg:plugin:{}:{}", NotifyPlugin::CODE, dedup_hash);
            let mut conn = cache_client.get_conn().await;
            // check if the key exists
            if let Ok(Some(_)) = conn.get::<'_, _, Option<String>>(&key).await {
                tracing::debug!("dedup cache hit, skip sending notification");
                return;
            }

            // set the dedup key
            if let Err(e) = conn.set_ex::<'_, _, _,Option<String>>(&key, "1", cool_down).await {
                tracing::error!(error = ?e, "set dedup cache failed");
                return;
            }

            let funs = NotifyPlugin::get_funs_inst_by_plugin_code();
            let response = bios_sdk_invoke::clients::reach_client::ReachClient::send_message(&req.into(), &funs, &ctx).await;
            if let Err(e) = response {
                tracing::error!(error = ?e, "send notification failed");
            }
        });
    }
    pub fn report<R: Report>(&self, response: &SgResponse, report: R) {
        let replace = report.get_replacement();
        let key = report.key();

        if let Some(template) = self.templates.get(key) {
            if let Some(notify_req) = template.reach.as_ref() {
                let mut req = notify_req.clone();
                req.merge_replace(replace.clone());
                let context = self.clone();
                context.submit_notify(req, report.dedup_hash());
            }
            if let Some(log_template) = template.audit_log.as_ref() {
                let formatted = format_template(log_template, &replace);
                self.submit_audit_log(response, Some(formatted));
            }
        }
    }
    pub fn submit_audit_log(&self, response: &SgResponse, extra_info: Option<String>) {
        let mut log_param_content = self.audit_param.as_ref().clone().merge_audit_log_param_content(response, true, &self.audit_log_token_header_name);
        if let Some(extra_info) = extra_info {
            log_param_content.op = extra_info;
        }
        log_param_content.send_audit_log(&self.spi_app_id, &self.log_api, &self.audit_log_tag);
    }
}

pub struct TamperReport {}

pub struct UnauthorizedOperationReport {}

pub struct CertLockReport {}

#[derive(Debug, Clone)]
pub struct ContentFilterForbiddenReport {
    pub(crate) forbidden_reason: String,
}

use spacegate_shell::{
    ext_redis::{redis::AsyncCommands, RedisClient},
    plugin::{
        schemars::{self, JsonSchema},
        Plugin,
    },
    SgResponse,
};

use crate::plugin::{
    notify::{format_template, NotifyPlugin, NotifyPluginConfigTemplates, NotifyPluginConfigTemplatesItem, Report},
    PluginBiosExt,
};

use super::audit_log_param::AuditLogParam;
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct ReachMsgSendReq {
    pub scene_code: String,
    pub receives: Vec<ReachMsgReceive>,
    pub rel_item_id: String,
    pub replace: HashMap<String, String>,
}

impl ReachMsgSendReq {
    pub fn merge_replace<K: Into<String>>(&mut self, replace: impl IntoIterator<Item = (K, String)>) {
        self.replace.extend(replace.into_iter().map(|(k, v)| (k.into(), v)));
    }
}

impl From<ReachMsgSendReq> for bios_sdk_invoke::clients::reach_client::ReachMsgSendReq {
    fn from(val: ReachMsgSendReq) -> Self {
        bios_sdk_invoke::clients::reach_client::ReachMsgSendReq {
            scene_code: val.scene_code,
            receives: val.receives.into_iter().map(Into::into).collect(),
            rel_item_id: val.rel_item_id,
            replace: val.replace,
        }
    }
}
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct ReachMsgReceive {
    pub receive_group_code: String,
    pub receive_kind: String,
    pub receive_ids: Vec<String>,
}

impl From<ReachMsgReceive> for bios_sdk_invoke::clients::reach_client::ReachMsgReceive {
    fn from(val: ReachMsgReceive) -> Self {
        bios_sdk_invoke::clients::reach_client::ReachMsgReceive {
            receive_group_code: val.receive_group_code,
            receive_kind: val.receive_kind,
            receive_ids: val.receive_ids,
        }
    }
}
