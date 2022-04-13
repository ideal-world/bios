use std::env;

use tardis::basic::result::TardisResult;
use tardis::tokio;
use tardis::TardisFuns;

use crate::config::BiosConfig;

mod config;
mod initializer;

///
/// Visit: http://127.0.0.1:8081/
///
#[tokio::main]
async fn main() -> TardisResult<()> {
    env::set_var("RUST_LOG", "info");
    TardisFuns::init("config").await?;
    let mut web_server = TardisFuns::web_server();
    initializer::init(web_server).await?;
    web_server.start().await
}
