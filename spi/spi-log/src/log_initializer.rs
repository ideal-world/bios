use bios_basic::spi::{api::spi_ci_bs_api, dto::spi_bs_dto::SpiBsCertResp, spi_constants, spi_funs::SpiBsInst, spi_initializer};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    log::info,
    tokio,
    web::web_server::TardisWebServer,
    TardisFuns, TardisFunsInst,
};

use crate::{api::ci::log_ci_item_api, log_config::LogConfig, log_constants::DOMAIN_CODE};

pub async fn init(web_server: &TardisWebServer) -> TardisResult<()> {
    info!("[BIOS.Log] Module initializing");
    let mut funs = crate::get_tardis_inst();
    bios_basic::rbum::rbum_initializer::init(funs.module_code(), funs.conf::<LogConfig>().rbum.clone()).await?;
    funs.begin().await?;
    let ctx = spi_initializer::init(DOMAIN_CODE, &funs).await?;
    init_db(&funs, &ctx).await?;
    funs.commit().await?;
    tokio::spawn(init_event());
    init_api(web_server).await?;
    info!("[BIOS.Log] Module initialized");
    Ok(())
}

async fn init_db(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    spi_initializer::add_kind(spi_constants::SPI_PG_KIND_CODE, funs, ctx).await?;
    Ok(())
}

async fn init_api(web_server: &TardisWebServer) -> TardisResult<()> {
    web_server.add_module(DOMAIN_CODE, (spi_ci_bs_api::SpiCiBsApi, log_ci_item_api::LogCiItemApi)).await;
    Ok(())
}

pub async fn init_fun(bs_cert: SpiBsCertResp, ctx: &TardisContext, mgr: bool) -> TardisResult<SpiBsInst> {
    info!("[BIOS.Log] Fun initializing");
    let inst = match bs_cert.kind_code.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => spi_initializer::common_pg::init(&bs_cert, ctx, mgr).await,
        _ => Err(bs_cert.bs_not_implemented())?,
    }?;
    info!("[BIOS.Log] Fun initialized");
    Ok(inst)
}

async fn init_event() -> TardisResult<()> {
    let funs = crate::get_tardis_inst();
    let conf = funs.conf::<LogConfig>();
    if let Some(event_config) = conf.event.as_ref() {
        loop {
            if TardisFuns::web_server().is_running().await {
                break;
            } else {
                tokio::task::yield_now().await
            }
        }
        crate::event::start_log_event_service(event_config).await?;
    }
    Ok(())
}

#[inline]
pub(crate) fn get_tardis_inst() -> TardisFunsInst {
    TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None)
}
