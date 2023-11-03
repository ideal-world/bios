use spacegate_lib::register_lib_filter;
use tardis::basic::result::TardisResult;
use tardis::config::config_dto::{FrameworkConfig, LogConfig, TardisConfig};
use tardis::{tokio, TardisFuns};

#[tokio::main]
async fn main() -> TardisResult<()> {
    TardisFuns::init_log()?;
    TardisFuns::init_conf(TardisConfig::builder().fw(FrameworkConfig::builder().log(LogConfig::default()).build()).build()).await?;
    let namespaces = std::env::args().nth(1).map(Some).unwrap_or(None);
    register_lib_filter();
    spacegate_kernel::startup(true, namespaces, None).await?;
    spacegate_kernel::wait_graceful_shutdown().await
}
