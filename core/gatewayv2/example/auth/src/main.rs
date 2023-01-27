use std::env;
use tardis::basic::result::TardisResult;
use tardis::tokio;
use tardis::TardisFuns;

#[tokio::main]
async fn main() -> TardisResult<()> {
    env::set_var("RUST_LOG", "debug,tardis=trace,sqlx=off,bios=trace,hyper::proto=off,sqlparser::parser=off");
    TardisFuns::init("config").await?;
    let web_server = TardisFuns::web_server();
    bios_auth::auth_initializer::init(web_server).await?;
    web_server.start().await
}
