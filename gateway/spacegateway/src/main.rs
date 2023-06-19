use plugin::auth;
use tardis::{basic::result::TardisResult, tokio, TardisFuns};

mod plugin;

#[tokio::main]
async fn main() -> TardisResult<()> {
    TardisFuns::init_log()?;
    let namespaces = std::env::args().nth(1).map(Some).unwrap_or(None);
    spacegate_kernel::register_filter_def(auth::CODE, Box::new(auth::SgFilterAuthDef));
    spacegate_kernel::startup(true, namespaces, None).await?;
    spacegate_kernel::wait_graceful_shutdown().await
}
