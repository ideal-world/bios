use bios_basic::spi::{dto::spi_bs_dto::SpiBsCertResp, spi_constants, spi_funs::SpiBsInst, spi_initializer};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    log::{self, info},
    web::web_server::TardisWebServer,
    TardisFuns, TardisFunsInst,
};

use crate::{
    api::{init_api, init_nacos_servers},
    conf_config::ConfConfig,
    conf_constants::DOMAIN_CODE,
};

pub async fn init(web_server: &TardisWebServer) -> TardisResult<()> {
    info!("[BIOS.Conf] Module initializing");
    let mut funs = crate::get_tardis_inst();
    let cfg = funs.conf::<ConfConfig>();
    bios_basic::rbum::rbum_initializer::init(funs.module_code(), cfg.rbum.clone()).await?;
    funs.begin().await?;
    let ctx = spi_initializer::init(DOMAIN_CODE, &funs).await?;
    init_db(&funs, &ctx).await?;
    funs.commit().await?;
    init_api(web_server).await;
    let funs = crate::get_tardis_inst();
    let cfg = funs.conf::<ConfConfig>();
    init_nacos_servers(&cfg).await?;
    info!("[BIOS.Conf] Module initialized");
    Ok(())
}

async fn init_db(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    spi_initializer::add_kind(spi_constants::SPI_PG_KIND_CODE, funs, ctx).await?;
    Ok(())
}

pub async fn init_fun(bs_cert: SpiBsCertResp, ctx: &TardisContext, mgr: bool) -> TardisResult<SpiBsInst> {
    info!("[BIOS.Conf] Fun [{}]({}) initializing", bs_cert.kind_code, bs_cert.conn_uri);
    let inst = match bs_cert.kind_code.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => spi_initializer::common_pg::init(&bs_cert, ctx, mgr).await,
        _ => Err(bs_cert.bs_not_implemented())?,
    }?;
    info!("[BIOS.Conf] Fun [{}]({}) initialized", bs_cert.kind_code, bs_cert.conn_uri);
    Ok(inst)
}

#[allow(dead_code)]
/// init spi-conf's admin cert
pub async fn init_admin_cert(funs: &TardisFunsInst, ctx: &TardisContext) {
    let cfg = funs.conf::<ConfConfig>();
    let req = cfg.get_admin_account();
    match crate::serv::register(req, funs, ctx).await {
        Ok(_) => {
            log::info!("[spi-conf] admin account registered");
        }
        Err(e) => {
            log::error!("[spi-conf] encounter an error when trying to register admin account: {e}");
        }
    }
}

#[inline]
pub(crate) fn get_tardis_inst() -> TardisFunsInst {
    TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None)
}

#[inline]
pub(crate) fn get_tardis_inst_ref() -> &'static TardisFunsInst {
    use std::sync::OnceLock;
    static INST: OnceLock<TardisFunsInst> = OnceLock::new();
    INST.get_or_init(|| TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None))
}
