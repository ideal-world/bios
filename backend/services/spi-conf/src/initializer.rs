use tardis::basic::result::TardisResult;
use tardis::web::web_server::TardisWebServer;

pub async fn init(web_server: &TardisWebServer) -> TardisResult<()> {
    bios_spi_conf::conf_initializer::init(web_server).await?;
    Ok(())
}
