use bios_reach::reach_send_channel::SendChannelMap;
use tardis::basic::result::TardisResult;
use tardis::web::web_server::TardisWebServer;

pub async fn init(web_server: &TardisWebServer) -> TardisResult<()> {
    bios_auth::auth_initializer::init(web_server).await?;
    bios_iam::iam_initializer::init(web_server).await?;
    bios_reach::reach_initializer::init(
        web_server,
        SendChannelMap::new()
            .with_arc_channel(bios_client_hwsms::SmsClient::from_reach_config())
            .with_arc_channel(tardis::TardisFuns::mail_by_module_or_default(bios_reach::reach_consts::MODULE_CODE)),
    )
    .await?;

    bios_spi_cache::cache_initializer::init(web_server).await?;
    bios_spi_graph::graph_initializer::init(web_server).await?;
    bios_spi_kv::kv_initializer::init(web_server).await?;
    bios_spi_log::log_initializer::init(web_server).await?;
    bios_spi_object::object_initializer::init(web_server).await?;
    bios_spi_plugin::plugin_initializer::init(web_server).await?;
    bios_spi_reldb::reldb_initializer::init(web_server).await?;
    bios_spi_search::search_initializer::init(web_server).await?;
    bios_spi_stats::stats_initializer::init(web_server).await?;
    bios_spi_conf::conf_initializer::init(web_server).await?;

    bios_mw_schedule::schedule_initializer::init(web_server).await?;
    bios_mw_flow::flow_initializer::init(web_server).await?;
    Ok(())
}
