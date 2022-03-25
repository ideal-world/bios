use tardis::TardisFuns;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{OpenApi, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use crate::console_tenant::dto::iam_ct_tenant_dto::IamCtTenantModifyReq;
use crate::console_tenant::serv::iam_ct_tenant_serv::IamCtTenantServ;

pub struct IamCtTenantApi;

/// Tenant Console Tenant API
#[OpenApi(prefix_path = "/ct/tenant", tag = "bios_basic::Components::Iam")]
impl IamCtTenantApi {
    /// Modify Current Tenant
    #[oai(path = "/", method = "put")]
    async fn modify(&self, mut modify_req: Json<IamCtTenantModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut tx = TardisFuns::reldb().conn();
        tx.begin().await?;
        IamCtTenantServ::modify_tenant(&mut modify_req.0, &tx, &cxt.0).await?;
        tx.commit().await?;
        TardisResp::ok(Void {})
    }
}
