use tardis::{basic::result::TardisResult, web::web_server::TardisWebServer, TardisFunsInst};

use crate::{console_app::api::reldb_ca_config, console_interface::api::reldb_ci_process, reldb_config::RelDbConfig, reldb_constants};

pub async fn init(web_server: &TardisWebServer) -> TardisResult<()> {
    let funs = reldb_constants::get_tardis_inst();
    init_db(funs).await?;
    init_api(web_server).await
}

async fn init_api(web_server: &TardisWebServer) -> TardisResult<()> {
    web_server.add_module(reldb_constants::COMPONENT_CODE, (reldb_ca_config::RelDbConfigApi, reldb_ci_process::RelDbProcessApi)).await;
    Ok(())
}

pub async fn init_db(mut funs: TardisFunsInst) -> TardisResult<()> {
    bios_basic::rbum::rbum_initializer::init(funs.module_code(), funs.conf::<RelDbConfig>().rbum.clone()).await?;
    funs.begin().await?;
    // TODO
    funs.commit().await?;
    Ok(())
}
