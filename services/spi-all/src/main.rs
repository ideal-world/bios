use tardis::basic::result::TardisResult;
use tardis::tokio;
use tardis::TardisFuns;

mod config;
mod initializer;

///
/// Visit: http://127.0.0.1:8080/
///
#[tokio::main]
async fn main() -> TardisResult<()> {
    //     env::set_var("RUST_LOG", "debug,tardis=trace,sqlx=off,bios=trace,hyper::proto=off,sqlparser::parser=off");
    let docker = tardis::testcontainers::clients::Cli::default();
    let _x = bios_basic::test::init_rbum_test_container::init(&docker, None).await?;
    // TardisFuns::init("config").await?;
    let web_server = TardisFuns::web_server();
    initializer::init(web_server).await?;
    web_server.start().await
}
