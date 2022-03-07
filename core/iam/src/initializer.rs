use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::web::web_server::TardisWebServer;
use tardis::TardisFuns;

use crate::console_system::api::iam_cs_tenant_api;
use crate::domain::iam_tenant;

pub async fn init_db() -> TardisResult<()> {
    bios_basic::rbum::initializer::init_db().await?;
    let mut tx = TardisFuns::reldb().conn();
    tx.begin().await?;
    tx.create_table(&iam_tenant::ActiveModel::create_table_statement(TardisFuns::reldb().backend())).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn init_api(web_server: &mut TardisWebServer) -> TardisResult<()> {
    web_server.add_module("iam", (iam_cs_tenant_api::IamCsTenantApi));
    Ok(())
}
