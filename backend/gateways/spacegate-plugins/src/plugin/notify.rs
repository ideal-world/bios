use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    net::IpAddr,
    sync::{Arc, OnceLock},
};

use bios_sdk_invoke::{invoke_config::InvokeConfig, invoke_enumeration::InvokeModuleKind, invoke_initializer};
use http::HeaderValue;
use serde::{Deserialize, Serialize};
use spacegate_shell::{
    ext_redis::RedisClient,
    kernel::extension::OriginalIpAddr,
    plugin::{plugin_meta, plugins::limit::RateLimitReport, schema, schemars, Inner, Plugin, PluginSchemaExt},
    BoxError, SgRequest, SgRequestExt, SgResponse,
};
use tardis::{
    log as tracing,
    regex::{self, Regex},
    serde_json,
};

use crate::extension::{
    audit_log_param::AuditLogParam,
    cert_info::CertInfo,
    notification::{CertLockReport, ContentFilterForbiddenReport, NotificationContext, ReachMsgSendReq, TamperReport, UnauthorizedOperationReport},
};
#[derive(Serialize, Deserialize, schemars::JsonSchema)]
#[serde(default)]
pub struct NotifyPluginConfig {
    /// templates for different notification types
    /// - rate_limit: rate limit notification
    ///   - count: number of requests
    ///   - time_window: time window
    ///
    /// - tamper: tamper notification
    ///
    /// - unauthorized_operation: unauthorized operation notification
    ///
    /// - cert_lock: cert lock notification
    ///
    /// - content_filter_forbidden: content filter forbidden notification
    ///   - reason: forbidden reason
    templates: HashMap<String, NotifyPluginConfigTemplatesItem>,
    log_api: String,
    reach_api: String,
    spi_app_id: String,
    audit_log_tag: String,
    audit_log_token_header_name: String,
    dedup_cache_cool_down: String,
}

impl Default for NotifyPluginConfig {
    fn default() -> Self {
        Self {
            templates: Default::default(),
            log_api: "http://bios.devops:8080".into(),
            reach_api: "http://bios.devops:8080".into(),
            spi_app_id: "".into(),
            audit_log_tag: "iam_abnormal".into(),
            audit_log_token_header_name: "Bios-Token".into(),
            dedup_cache_cool_down: "10m".into(),
        }
    }
}

pub trait Report {
    fn get_replacement(&self) -> HashMap<&'static str, String>;
    fn key(&self) -> &'static str;
    fn dedup_hash(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.key().hash(&mut hasher);
        hasher.finish()
    }
}

pub struct WithUserAndIp<'a, T> {
    pub(crate) user: Option<&'a str>,
    pub(crate) ip: IpAddr,
    pub(crate) raw: &'a T,
}

pub trait ReportExt {
    fn with_user_and_ip<'a>(&'a self, user: Option<&'a str>, ip: IpAddr) -> WithUserAndIp<'a, Self>
    where
        Self: Sized;
}

impl<T: Report> ReportExt for T {
    fn with_user_and_ip<'a>(&'a self, user: Option<&'a str>, ip: IpAddr) -> WithUserAndIp<'a, Self> {
        WithUserAndIp { user, ip, raw: self }
    }
}

impl<T> Report for WithUserAndIp<'_, T>
where
    T: Report,
{
    fn get_replacement(&self) -> HashMap<&'static str, String> {
        let mut replace = self.raw.get_replacement();
        let mut formatted_user = self.ip.to_string();
        // replace . with full-width dot
        if let Some(tamper_user) = self.user {
            formatted_user.push('(');
            formatted_user.push_str(tamper_user);
            formatted_user.push(')');
        }
        replace.insert("user", formatted_user);
        replace
    }
    fn key(&self) -> &'static str {
        self.raw.key()
    }
    fn dedup_hash(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.raw.dedup_hash().hash(&mut hasher);
        self.ip.hash(&mut hasher);
        hasher.finish()
    }
}

impl Report for RateLimitReport {
    fn get_replacement(&self) -> HashMap<&'static str, String> {
        let mut replace = HashMap::new();
        let count = format!("{}", self.plugin.max_request_number);
        let time_window = format!("{}ms", self.plugin.time_window_ms);
        replace.insert("count", count);
        replace.insert("time_window", time_window);
        replace
    }
    fn key(&self) -> &'static str {
        "rate_limit"
    }
}

impl Report for CertLockReport {
    fn get_replacement(&self) -> HashMap<&'static str, String> {
        HashMap::new()
    }
    fn key(&self) -> &'static str {
        "cert_lock"
    }
}

impl Report for TamperReport {
    fn get_replacement(&self) -> HashMap<&'static str, String> {
        HashMap::new()
    }
    fn key(&self) -> &'static str {
        "tamper"
    }
}

impl Report for UnauthorizedOperationReport {
    fn get_replacement(&self) -> HashMap<&'static str, String> {
        HashMap::new()
    }
    fn key(&self) -> &'static str {
        "unauthorized_operation"
    }
}

impl Report for ContentFilterForbiddenReport {
    fn get_replacement(&self) -> HashMap<&'static str, String> {
        let mut replace = HashMap::new();
        replace.insert("reason", self.forbidden_reason.to_string());
        replace
    }
    fn key(&self) -> &'static str {
        "content_filter_forbidden"
    }
}
#[derive(Serialize, Deserialize, schemars::JsonSchema, Default, Debug)]
pub struct NotifyPluginConfigTemplates {
    pub rate_limit: NotifyPluginConfigTemplatesItem,
    pub tamper: NotifyPluginConfigTemplatesItem,
    pub unauthorized_operation: NotifyPluginConfigTemplatesItem,
}
#[derive(Serialize, Deserialize, schemars::JsonSchema, Default, Debug)]
#[serde(default)]
pub struct NotifyPluginConfigTemplatesItem {
    pub reach: Option<ReachRequest>,
    pub audit_log: Option<String>,
}

#[derive(Serialize, Deserialize, schemars::JsonSchema, Debug, Clone)]
#[serde(tag = "method", content = "request")]
pub enum ReachRequest {
    ByScene(ReachMsgSendReq),
    ByTemplate {
        contact: String,
        template_id: String,
        replace: HashMap<String, String>,
    },
}

impl ReachRequest {
    pub fn merge_replace<K: Into<String>>(&mut self, replace: impl IntoIterator<Item = (K, String)>) {
        match self {
            Self::ByScene(req) => {
                req.merge_replace(replace);
            }
            Self::ByTemplate { replace: old_replace, .. } => {
                old_replace.extend(replace.into_iter().map(|(k, v)| (k.into(), v)));
            }
        }
    }
}

schema!(NotifyPlugin, NotifyPluginConfig);
#[derive(Debug, Clone)]
pub struct NotifyPlugin {
    log_api: Arc<str>,
    reach_api: Arc<str>,
    spi_app_id: Arc<str>,
    audit_log_tag: Arc<str>,
    audit_log_token_header_name: Arc<str>,
    templates: Arc<HashMap<String, NotifyPluginConfigTemplatesItem>>,
    dedup_cache_cool_down: std::time::Duration,
}

pub fn format_template(template: &str, replace: &HashMap<&'static str, String>) -> String {
    static PLACEHOLDER_RE: OnceLock<Regex> = OnceLock::new();
    const NULL_STR: &str = "<null>";
    fn re() -> &'static Regex {
        PLACEHOLDER_RE.get_or_init(|| Regex::new(r"\$\{(\w+)\}").expect("invalid regex"))
    }
    let formatted = re().replace_all(template, |caps: &regex::Captures| {
        let replaced = replace.get(caps.get(1).expect("regex should have a capture").as_str());
        if let Some(replaced) = replaced {
            replaced.as_str()
        } else {
            NULL_STR
        }
    });
    formatted.to_string()
}

impl Plugin for NotifyPlugin {
    const CODE: &'static str = "notify";
    fn meta() -> spacegate_shell::model::PluginMetaData {
        plugin_meta! {
            description: "attach a notification api calling context to the request"
        }
    }
    fn create(plugin_config: spacegate_shell::model::PluginConfig) -> Result<Self, BoxError> {
        let config: NotifyPluginConfig = serde_json::from_value(plugin_config.spec)?;
        if config.spi_app_id.is_empty() {
            tardis::log::error!("[Plugin.AuditLog] log_url or spi_app_id is empty!");
        } else {
            invoke_initializer::init(
                Self::CODE,
                InvokeConfig {
                    spi_app_id: config.spi_app_id.clone(),
                    module_urls: HashMap::from([
                        (InvokeModuleKind::Log.to_string(), config.log_api.clone()),
                        (InvokeModuleKind::Reach.to_string(), config.reach_api.clone()),
                    ]),
                    ..Default::default()
                },
            )?;
        }
        let dedup_cache_cool_down = crate::utils::parse_duration(&config.dedup_cache_cool_down)?;
        Ok(Self {
            log_api: config.log_api.into(),
            reach_api: config.reach_api.into(),
            spi_app_id: config.spi_app_id.into(),
            audit_log_tag: config.audit_log_tag.into(),
            audit_log_token_header_name: config.audit_log_token_header_name.into(),
            templates: Arc::new(config.templates),
            dedup_cache_cool_down,
        })
    }
    async fn call(&self, mut req: SgRequest, inner: Inner) -> Result<SgResponse, BoxError> {
        let audit_param = req.extract::<AuditLogParam>();
        let redis_client = req.extract::<Option<RedisClient>>();
        let Some(redis_client) = redis_client else {
            // skip report since redis client is not available
            return Ok(inner.call(req).await);
        };
        let context = NotificationContext {
            templates: self.templates.clone(),
            log_api: self.log_api.clone(),
            reach_api: self.reach_api.clone(),
            spi_app_id: self.spi_app_id.clone(),
            audit_log_tag: self.audit_log_tag.clone(),
            audit_log_token_header_name: self.audit_log_token_header_name.clone(),
            audit_param: Arc::new(audit_param),
            cache_client: redis_client,
            dedup_cache_cool_down: self.dedup_cache_cool_down,
        };
        req.extensions_mut().insert(context.clone());
        let req_cert_info = req.extensions().get::<CertInfo>().cloned();
        let ip = req.extract::<OriginalIpAddr>().0;
        let response = inner.call(req).await;
        let user = req_cert_info.as_ref().or_else(|| response.extensions().get::<CertInfo>()).and_then(|c| c.name.clone());
        if let Some(rate_limit_report) = response.extensions().get::<RateLimitReport>().filter(|r| r.rising_edge) {
            tracing::debug!(report = ?rate_limit_report, "catch rate limit report");
            context.report(&response, rate_limit_report.with_user_and_ip(user.as_deref(), ip));
        }
        if let Some(report) = response.extensions().get::<ContentFilterForbiddenReport>() {
            tracing::debug!(?report, "catch content filter forbidden report");

            context.report(&response, report.with_user_and_ip(user.as_deref(), ip));
        }
        if let Some(error_code) = response.headers().get(tardis::web::web_resp::HEADER_X_TARDIS_ERROR) {
            tracing::debug!(?error_code, "catch error code");
            if let Some(error_kind) = KnownError::try_from_header_value(error_code) {
                match error_kind {
                    KnownError::Tamper => {
                        context.report(&response, TamperReport {}.with_user_and_ip(user.as_deref(), ip));
                    }
                    KnownError::UnauthorizedOperation => {
                        context.report(&response, UnauthorizedOperationReport {}.with_user_and_ip(user.as_deref(), ip));
                    }
                    KnownError::CertLock => {
                        context.report(&response, CertLockReport {}.with_user_and_ip(user.as_deref(), ip));
                    }
                }
            }
        }
        Ok(response)
    }
    fn schema_opt() -> Option<schemars::schema::RootSchema> {
        Some(NotifyPlugin::schema())
    }
}

pub enum KnownError {
    Tamper,
    UnauthorizedOperation,
    CertLock,
}

impl KnownError {
    pub fn try_from_header_value(header_value: &HeaderValue) -> Option<Self> {
        match header_value.to_str().ok()? {
            code if code.starts_with("401-signature-error") => Some(Self::Tamper),
            code if code.starts_with("403-req-permission-denied") || code.starts_with("401-auth-req-unauthorized") => Some(Self::UnauthorizedOperation),
            code if code.starts_with("400-rbum-cert-lock") => Some(Self::CertLock),
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_template() {
        use super::format_template;
        let template = "hello ${obj}";
        let mut replace = std::collections::HashMap::new();
        replace.insert("obj", "world".to_string());
        assert_eq!(format_template(template, &replace), "hello world");
        let replace = std::collections::HashMap::new();
        assert_eq!(format_template(template, &replace), "hello <null>");
    }
}
