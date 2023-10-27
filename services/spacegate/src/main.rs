use spacegate_lib::register_lib_filter;
use tardis::basic::result::TardisResult;
use tardis::{tokio, TardisFuns};
use tardis::config::config_dto::{FrameworkConfig, LogConfig, TardisConfig};
use tracing_subscriber::filter::Directive;

#[tokio::main]
async fn main() -> TardisResult<()> {
    TardisFuns::init_log()?;
    TardisFuns::init_conf(TardisConfig::builder().fw(FrameworkConfig::builder().log(LogConfig::builder().level("info".parse::<Directive>().expect("parse info level error!")).build()).build()).build()).await?;
    let namespaces = std::env::args().nth(1).map(Some).unwrap_or(None);
    register_lib_filter();
    spacegate_kernel::startup(true, namespaces, None).await?;
    spacegate_kernel::wait_graceful_shutdown().await
}
