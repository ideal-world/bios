use bios_basic::spi::{api::spi_ci_bs_api, dto::spi_bs_dto::SpiBsCertResp, spi_constants, spi_funs::SpiBsInst, spi_initializer};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    log::info,
    web::web_server::TardisWebServer,
    TardisFuns, TardisFunsInst,
};

use crate::{
    api::ci::{kv_ci_item_api, kv_ci_transfer_api},
    kv_config::KvConfig,
    kv_constants::DOMAIN_CODE,
};

pub async fn init(web_server: &TardisWebServer) -> TardisResult<()> {
    info!("[BIOS.KV] Module initializing");
    let mut funs = crate::get_tardis_inst();
    crate::event::handle_events().await?;
    bios_basic::rbum::rbum_initializer::init(funs.module_code(), funs.conf::<KvConfig>().rbum.clone()).await?;
    funs.begin().await?;
    let ctx = spi_initializer::init(DOMAIN_CODE, &funs).await?;
    init_db(&funs, &ctx).await?;
    funs.commit().await?;
    init_api(web_server).await?;
    info!("[BIOS.KV] Module initialized");
    Ok(())
}

async fn init_db(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    spi_initializer::add_kind(spi_constants::SPI_PG_KIND_CODE, funs, ctx).await?;
    Ok(())
}

async fn init_api(web_server: &TardisWebServer) -> TardisResult<()> {
    web_server.add_module(DOMAIN_CODE, (spi_ci_bs_api::SpiCiBsApi, kv_ci_item_api::KvCiItemApi, kv_ci_transfer_api::KvTransferApi)).await;
    Ok(())
}

pub async fn init_fun(bs_cert: SpiBsCertResp, ctx: &TardisContext, mgr: bool) -> TardisResult<SpiBsInst> {
    info!("[BIOS.KV] Fun [{}]({}) initializing", bs_cert.kind_code, bs_cert.conn_uri);
    let inst = match bs_cert.kind_code.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => spi_initializer::common_pg::init(&bs_cert, ctx, mgr).await,
        _ => Err(bs_cert.bs_not_implemented())?,
    }?;
    info!("[BIOS.KV] Fun [{}]({}) initialized", bs_cert.kind_code, bs_cert.conn_uri);
    Ok(inst)
}

#[inline]
pub(crate) fn get_tardis_inst() -> TardisFunsInst {
    TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None)
}
