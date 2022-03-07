use tardis::basic::result::TardisResult;
use tardis::web::web_server::TardisWebServer;

pub async fn init(web_server: &mut TardisWebServer) -> TardisResult<()> {
    bios_iam::initializer::init_db().await?;
    bios_iam::initializer::init_api(web_server).await
}
