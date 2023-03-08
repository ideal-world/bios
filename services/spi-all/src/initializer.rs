use bios_basic::rbum::rbum_config::RbumConfig;
use bios_basic::rbum::rbum_initializer;
use bios_spi_cache::cache_initializer;
use bios_spi_kv::kv_initializer;
use bios_spi_log::log_initializer;
use bios_spi_object::object_initializer;
use bios_spi_plugin::plugin_initializer;
use bios_spi_reldb::reldb_initializer;
use bios_spi_search::search_initializer;
use bios_spi_stats::stats_initializer;
use tardis::basic::result::TardisResult;
use tardis::web::web_server::TardisWebServer;

pub async fn init(web_server: &TardisWebServer) -> TardisResult<()> {
    rbum_initializer::init("bios-spi", RbumConfig::default()).await?;
    cache_initializer::init(web_server).await?;
    kv_initializer::init(web_server).await?;
    log_initializer::init(web_server).await?;
    object_initializer::init(web_server).await?;
    plugin_initializer::init(web_server).await?;
    reldb_initializer::init(web_server).await?;
    search_initializer::init(web_server).await?;
    stats_initializer::init(web_server).await?;
    Ok(())
}
