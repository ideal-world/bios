use bios_basic::{
    rbum::serv::rbum_domain_serv::RbumDomainServ,
    spi::{api::spi_ci_bs_api, spi_initializer},
};
use tardis::{basic::result::TardisResult, db::reldb_client::TardisActiveModel, web::web_server::TardisWebServer, TardisFuns, TardisFunsInst};

use crate::{
    api::ci::{plugin_ci_api_api, plugin_exec_ci_api},
    domain::plugin_api,
    plugin_constants::DOMAIN_CODE,
};

pub async fn init(web_server: &TardisWebServer) -> TardisResult<()> {
    let mut funs = TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None);
    funs.begin().await?;
    init_db(DOMAIN_CODE.to_string(), &funs).await?;
    spi_initializer::init(DOMAIN_CODE, &funs).await?;
    funs.commit().await?;
    init_api(web_server).await
}

async fn init_db(domain_code: String, funs: &TardisFunsInst) -> TardisResult<()> {
    if RbumDomainServ::get_rbum_domain_id_by_code(&domain_code, funs).await?.is_some() {
        return Ok(());
    }
    // Initialize plugin component RBUM item table and indexs
    funs.db().init(plugin_api::ActiveModel::init(TardisFuns::reldb().backend(), None)).await?;
    Ok(())
}

async fn init_api(web_server: &TardisWebServer) -> TardisResult<()> {
    web_server.add_module(DOMAIN_CODE, (spi_ci_bs_api::SpiCiBsApi, plugin_ci_api_api::PluginApiApi, plugin_exec_ci_api::PluginExecApi)).await;
    Ok(())
}
