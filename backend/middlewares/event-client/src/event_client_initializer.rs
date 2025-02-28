use crate::event_client_config::EventClientConfig;
use bios_sdk_invoke::clients::event_client::init_ws_client_node;
use bios_sdk_invoke::invoke_initializer;
use std::time::Duration;
use tardis::tracing::{self, instrument};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    TardisFuns,
};
const CODE: &str = "event-client";
#[instrument(name = "event-client-init")]
pub async fn init() -> TardisResult<()> {
    let funs = TardisFuns::inst(CODE, None);
    let config = funs.conf::<EventClientConfig>();
    let invoke_config = &config.invoke;
    invoke_initializer::init(funs.module_code(), invoke_config.clone())?;

    if config.enable {
        tracing::info!(?config, "initialize event client");
        let context = TardisContext {
            ak: invoke_config.spi_app_id.to_string(),
            own_paths: invoke_config.spi_app_id.to_string(),
            ..Default::default()
        };
        #[cfg(feature = "local")]
        if config.local {
            bios_sdk_invoke::clients::event_client::init_local_client_node().await?;
            return Ok(())
        } 
        init_ws_client_node(config.max_retry_times, Duration::from_millis(config.retry_duration_ms as u64), &context, &funs).await;
    } else {
        tardis::tracing::info!("event client no enabled");
    }
    Ok(())
}
