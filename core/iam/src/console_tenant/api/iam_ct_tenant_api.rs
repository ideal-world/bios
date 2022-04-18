use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{payload::Json, OpenApi};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use crate::basic::dto::iam_tenant_dto::IamTenantDetailResp;
use crate::console_tenant::dto::iam_ct_tenant_dto::IamCtTenantModifyReq;
use crate::console_tenant::serv::iam_ct_tenant_serv::IamCtTenantServ;
use crate::iam_constants;

pub struct IamCtTenantApi;

/// Tenant Console Tenant API
#[OpenApi(prefix_path = "/ct/tenant", tag = "crate::iam_enumeration::Tag::Tenant")]
impl IamCtTenantApi {
    /// Get Current Tenant
    #[oai(path = "/", method = "get")]
    async fn get(&self, cxt: TardisContextExtractor) -> TardisApiResult<IamTenantDetailResp> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamCtTenantServ::get_tenant(&funs, &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Modify Current Tenant
    #[oai(path = "/", method = "put")]
    async fn modify(&self, mut modify_req: Json<IamCtTenantModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCtTenantServ::modify_tenant(&mut modify_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }
}
