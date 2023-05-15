use bios_basic::{
    rbum::{rbum_config::RbumConfig, rbum_initializer},
    spi::{dto::spi_bs_dto::SpiBsCertResp, spi_constants, spi_funs::SpiBsInst, spi_initializer},
};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    web::web_server::TardisWebServer,
    TardisFuns, TardisFunsInst,
};

use crate::{api::ci::schedule_ci_job_api, schedule_constants::DOMAIN_CODE, serv::schedule_job_serv};

pub async fn init(web_server: &TardisWebServer) -> TardisResult<()> {
    let mut funs = TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None);
    rbum_initializer::init(funs.module_code(), RbumConfig::default()).await?;
    funs.begin().await?;
    let ctx = spi_initializer::init(DOMAIN_CODE, &funs).await?;
    schedule_job_serv::init(&funs, &ctx).await?;
    init_db(&funs, &ctx).await?;
    funs.commit().await?;
    init_api(web_server).await
}

async fn init_db(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    spi_initializer::add_kind(spi_constants::SPI_PG_KIND_CODE, funs, ctx).await?;
    Ok(())
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
