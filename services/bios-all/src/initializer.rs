use tardis::basic::result::TardisResult;
use tardis::web::web_server::TardisWebServer;

pub async fn init(web_server: &TardisWebServer) -> TardisResult<()> {
    bios_auth::auth_initializer::init(web_server).await?;
    bios_iam::iam_initializer::init(web_server).await?;
    Ok(())
}
