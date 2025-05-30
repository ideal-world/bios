use bios_basic::spi::{api::spi_ci_bs_api, dto::spi_bs_dto::SpiBsCertResp, spi_constants, spi_funs::SpiBsInst, spi_initializer};
use bios_sdk_invoke::invoke_initializer;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    log::info,
    web::web_server::TardisWebServer,
    TardisFuns, TardisFunsInst,
};

use crate::{
    api::ci::{stats_ci_conf_api, stats_ci_metric_api, stats_ci_record_api, stats_ci_sync_api, stats_ci_transfer_api},
    stats_config::StatsConfig,
    stats_constants::DOMAIN_CODE,
};

pub async fn init(web_server: &TardisWebServer) -> TardisResult<()> {
    info!("[BIOS.Stats] Module initializing");
    let mut funs = crate::get_tardis_inst();
    bios_basic::rbum::rbum_initializer::init(funs.module_code(), funs.conf::<StatsConfig>().rbum.clone()).await?;
    invoke_initializer::init(funs.module_code(), funs.conf::<StatsConfig>().invoke.clone())?;
    funs.begin().await?;
    let ctx = spi_initializer::init(DOMAIN_CODE, &funs).await?;
    init_db(&funs, &ctx).await?;
    funs.commit().await?;
    crate::event::handle_events().await?;
    init_api(web_server).await?;
    info!("[BIOS.Stats] Module initialized");
    Ok(())
}

async fn init_db(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    spi_initializer::add_kind(spi_constants::SPI_PG_KIND_CODE, funs, ctx).await?;
    Ok(())
}

async fn init_api(web_server: &TardisWebServer) -> TardisResult<()> {
    web_server
        .add_module(
            DOMAIN_CODE,
            (
                spi_ci_bs_api::SpiCiBsApi,
                stats_ci_conf_api::StatsCiConfApi,
                stats_ci_record_api::StatsCiRecordApi,
                stats_ci_metric_api::StatsCiMetricApi,
                stats_ci_sync_api::StatsCiSyncApi,
                stats_ci_transfer_api::StatsCiTransferApi,
            ),
        )
        .await;
    Ok(())
}

pub async fn init_fun(bs_cert: SpiBsCertResp, ctx: &TardisContext, mgr: bool) -> TardisResult<SpiBsInst> {
    info!("[BIOS.Stats] Fun [{}]({}) initializing", bs_cert.kind_code, bs_cert.conn_uri);
    let inst = match bs_cert.kind_code.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => spi_initializer::common_pg::init(&bs_cert, ctx, mgr).await,
        _ => Err(bs_cert.bs_not_implemented())?,
    }?;
    info!("[BIOS.Stats] Fun [{}]({}) initialized", bs_cert.kind_code, bs_cert.conn_uri);
    Ok(inst)
}

#[inline]
pub(crate) fn get_tardis_inst() -> TardisFunsInst {
    TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None)
}
