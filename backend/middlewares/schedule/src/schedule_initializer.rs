use crate::{api::ci::schedule_ci_job_api, schedule_config::ScheduleConfig, schedule_constants::DOMAIN_CODE, serv::schedule_job_serv_v2};
use bios_basic::spi::{dto::spi_bs_dto::SpiBsCertResp, spi_constants, spi_funs::SpiBsInst, spi_initializer};
use bios_sdk_invoke::invoke_initializer;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    log::error,
    web::{web_server::TardisWebServer, ws_client::TardisWSClient},
    TardisFuns,
};

pub async fn init(web_server: &TardisWebServer) -> TardisResult<()> {
    let mut funs = TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None);
    invoke_initializer::init(funs.module_code(), funs.conf::<ScheduleConfig>().invoke.clone())?;
    funs.begin().await?;
    schedule_job_serv_v2::init();
    funs.commit().await?;
    init_api(web_server).await
}

async fn init_api(web_server: &TardisWebServer) -> TardisResult<()> {
    web_server.add_module(DOMAIN_CODE, schedule_ci_job_api::ScheduleCiJobApi).await;
    Ok(())
}

pub async fn init_fun(bs_cert: SpiBsCertResp, ctx: &TardisContext, mgr: bool) -> TardisResult<SpiBsInst> {
    match bs_cert.kind_code.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => spi_initializer::common_pg::init(&bs_cert, ctx, mgr).await,
        _ => Err(bs_cert.bs_not_implemented())?,
    }
}