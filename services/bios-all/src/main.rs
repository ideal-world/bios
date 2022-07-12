use bios_iam::integration::ldap::ldap_server;
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
    TardisFuns::init("config").await?;
    let web_server = TardisFuns::web_server();
    initializer::init(web_server).await?;
    ldap_server::start().await?;
    web_server.start().await
}
