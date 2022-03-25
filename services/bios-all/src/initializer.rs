use tardis::basic::result::TardisResult;
use tardis::web::web_server::TardisWebServer;

pub async fn init(web_server: &mut TardisWebServer) -> TardisResult<()> {
    bios_iam::iam_initializer::init_db().await?;
    bios_iam::iam_initializer::init_api(web_server).await
}
