use tardis::basic::result::TardisResult;
use tardis::web::web_server::TardisWebServer;

pub async fn init(web_server: &TardisWebServer) -> TardisResult<()> {
    bios_mw_event::event_initializer::init(web_server).await?;
    Ok(())
}
