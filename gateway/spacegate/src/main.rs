#![warn(clippy::unwrap_used)]
use plugin::{anti_replay, anti_xss, audit_log, auth, ip_time};
use tardis::{basic::result::TardisResult, tokio, TardisFuns};

mod plugin;

pub const PACKAGE_NAME: &str = "bios_spacegate";

#[tokio::main]
async fn main() -> TardisResult<()> {
    TardisFuns::init_log()?;
    let namespaces = std::env::args().nth(1).map(Some).unwrap_or(None);
    spacegate_kernel::register_filter_def(audit_log::SgFilterAuditLogDef);
    spacegate_kernel::register_filter_def(ip_time::SgFilterIpTimeDef);
    spacegate_kernel::register_filter_def(anti_replay::SgFilterAntiReplayDef);
    spacegate_kernel::register_filter_def(anti_xss::SgFilterAntiXSSDef);
    spacegate_kernel::register_filter_def(auth::SgFilterAuthDef);
    spacegate_kernel::startup(true, namespaces, None).await?;
    spacegate_kernel::wait_graceful_shutdown().await
}
