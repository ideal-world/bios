use tardis::basic::result::TardisResult;
use tardis::tokio;
use tardis::TardisFuns;

mod initializer;

///
/// Visit: http://127.0.0.1:8080/
///
#[tokio::main]
async fn main() -> TardisResult<()> {
    TardisFuns::init(Some("config")).await?;
    let web_server = TardisFuns::web_server();
    initializer::init(&web_server).await?;
    web_server.start().await?;
    web_server.await;
    Ok(())
}
