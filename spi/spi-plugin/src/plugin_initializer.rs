use bios_basic::{rbum::serv::rbum_domain_serv::RbumDomainServ, spi::spi_initializer};
use bios_sdk_invoke::invoke_initializer;
use tardis::{basic::result::TardisResult, db::reldb_client::TardisActiveModel, web::web_server::TardisWebServer, TardisFuns, TardisFunsInst};

use crate::{
    api::ci::{plugin_ci_api_api, plugin_ci_bs_api, plugin_ci_exec_api, plugin_ci_kind_api},
    domain::plugin_api,
    plugin_config::PluginConfig,
    plugin_constants::DOMAIN_CODE,
};

pub async fn init(web_server: &TardisWebServer) -> TardisResult<()> {
    let mut funs = crate::get_tardis_inst();
    bios_basic::rbum::rbum_initializer::init(funs.module_code(), funs.conf::<PluginConfig>().rbum.clone()).await?;
    invoke_initializer::init(funs.module_code(), funs.conf::<PluginConfig>().invoke.clone())?;
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
    funs.db().init(plugin_api::ActiveModel::init(TardisFuns::reldb().backend(), None, TardisFuns::reldb().compatible_type())).await?;
    Ok(())
}

async fn init_api(web_server: &TardisWebServer) -> TardisResult<()> {
    web_server
        .add_module(
            DOMAIN_CODE,
            (
                plugin_ci_bs_api::PluginCiBsApi,
                plugin_ci_api_api::PluginApiApi,
                plugin_ci_exec_api::PluginExecApi,
                plugin_ci_kind_api::PluginKindApi,
            ),
        )
        .await;
    Ok(())
}

#[inline]
pub(crate) fn get_tardis_inst() -> TardisFunsInst {
    TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None)
}
