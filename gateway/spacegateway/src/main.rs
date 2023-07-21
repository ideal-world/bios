#![warn(clippy::unwrap_used)]
use plugin::{auth, ip_time};
use tardis::{basic::result::TardisResult, tokio, TardisFuns};

mod plugin;

#[tokio::main]
async fn main() -> TardisResult<()> {
    TardisFuns::init_log()?;
    let namespaces = std::env::args().nth(1).map(Some).unwrap_or(None);
    spacegate_kernel::register_filter_def(auth::CODE, Box::new(auth::SgFilterAuthDef));
    spacegate_kernel::register_filter_def(ip_time::CODE, Box::new(ip_time::SgFilterIpTimeDef));
    spacegate_kernel::startup(true, namespaces, None).await?;
    spacegate_kernel::wait_graceful_shutdown().await
}
