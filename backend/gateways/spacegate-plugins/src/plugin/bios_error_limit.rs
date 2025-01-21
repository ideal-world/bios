use http::StatusCode;
use serde::{Deserialize, Serialize};
use spacegate_shell::ext_redis::redis::Script;
use spacegate_shell::hyper::body::Body;
use spacegate_shell::kernel::extension::OriginalIpAddr;
use spacegate_shell::plugin::{
    plugin_meta,
    schemars::{self, JsonSchema},
};
use spacegate_shell::plugin::{schema, Plugin, PluginSchemaExt};
use spacegate_shell::{BoxError, SgRequestExt, SgResponse, SgResponseExt};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::SystemTime;
use tardis::serde_json;
use tardis::web::web_resp::HEADER_X_TARDIS_ERROR;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BiosErrorLimitConfig {
    rules: HashMap<String, BiosErrorRule>,
}
const CACHE_BIOS_ERROR_LIMIT_KEY: &str = "sg:plugin:bios-error-limit";
#[derive(Debug, Clone)]
pub struct BiosErrorLimitReport {
    pub(crate) error_code: String,
    pub(crate) count: u32,
    pub(crate) time_window_ms: u32,
    pub(crate) is_rising_edge: bool,
    pub(crate) rule: BiosErrorRule,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BiosErrorRule {
    time_window_ms: u32,
    max_request_number: u32,
    path: Option<String>,
    method: Option<String>,
}
pub fn script() -> &'static Script {
    static SCRIPT: OnceLock<Script> = OnceLock::new();
    SCRIPT.get_or_init(|| Script::new(include_str!("./bios_error_limit/script.lua")))
}

#[derive(Debug, Clone)]
pub struct BiosErrorLimitPlugin {
    config: Arc<BiosErrorLimitConfig>,
    plugin_id: Arc<str>,
}
impl Deref for BiosErrorLimitPlugin {
    type Target = BiosErrorLimitConfig;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}
impl Plugin for BiosErrorLimitPlugin {
    const CODE: &'static str = "bios-error-limit";
    fn meta() -> spacegate_shell::model::PluginMetaData {
        plugin_meta! {
            description: "Filter content based on type, keywords and length."
        }
    }

    async fn call(&self, req: spacegate_shell::SgRequest, inner: spacegate_shell::plugin::Inner) -> Result<SgResponse, BoxError> {
        let ip = req.extract::<OriginalIpAddr>().to_canonical();
        let mut conn = req.get_redis_client_by_gateway_name().ok_or("missing gateway name")?.get_conn().await;
        let mut response = inner.call(req).await;
        if let Some(error_code) = response.headers().get(HEADER_X_TARDIS_ERROR).cloned() {
            use tardis::log as tracing;
            if let Ok(error_code) = error_code.to_str() {
                tracing::debug!(error_code, "catch error code");
                if let Some(rule) = self.rules.get(error_code).or(self.rules.get("*")) {
                    const EXCEEDED: i32 = 0;
                    const RISING_EDGE: i32 = 1;
                    let id = self.plugin_id.as_ref();
                    let result: i32 = script()
                        // counter key
                        .key(format!("{CACHE_BIOS_ERROR_LIMIT_KEY}:{id}:{error_code}:{ip}"))
                        // last counter reset timestamp key
                        .key(format!("{CACHE_BIOS_ERROR_LIMIT_KEY}:{id}:{error_code}:{ip}_ts"))
                        // maximum number of request
                        .arg(rule.max_request_number)
                        // time window
                        .arg(rule.time_window_ms)
                        // current timestamp
                        .arg(SystemTime::now().duration_since(std::time::UNIX_EPOCH).expect("invalid system time: before unix epoch").as_millis() as u64)
                        .invoke_async(&mut conn)
                        .await?;

                    if result == EXCEEDED || result == RISING_EDGE {
                        response.extensions_mut().insert(BiosErrorLimitReport {
                            error_code: error_code.to_string(),
                            count: rule.max_request_number,
                            time_window_ms: rule.time_window_ms,
                            is_rising_edge: result == RISING_EDGE,
                            rule: rule.clone(),
                        });
                    }
                }
            }
        }
        Ok(response)
    }

    fn create(plugin_config: spacegate_shell::model::PluginConfig) -> Result<Self, spacegate_shell::plugin::BoxError> {
        let config = serde_json::from_value(plugin_config.spec)?;
        Ok(BiosErrorLimitPlugin {
            config: Arc::new(config),
            plugin_id: plugin_config.id.to_string().into(),
        })
    }

    fn schema_opt() -> Option<schemars::schema::RootSchema> {
        Some(BiosErrorLimitPlugin::schema())
    }
}

schema!(BiosErrorLimitPlugin, BiosErrorLimitConfig);
