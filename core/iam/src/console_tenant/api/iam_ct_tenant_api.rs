use crate::basic::dto::iam_filer_dto::IamTenantFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{payload::Json, OpenApi};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use crate::basic::dto::iam_tenant_dto::{IamTenantDetailResp, IamTenantModifyReq};
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::iam_constants;

pub struct IamCtTenantApi;

/// Tenant Console Tenant API
#[OpenApi(prefix_path = "/ct/tenant", tag = "crate::iam_enumeration::Tag::Tenant")]
impl IamCtTenantApi {
    /// Get Current Tenant
    #[oai(path = "/", method = "get")]
    async fn get(&self, ctx: TardisContextExtractor) -> TardisApiResult<IamTenantDetailResp> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamTenantServ::get_item(&IamTenantServ::get_id_by_ctx(&ctx.0, &funs)?, &IamTenantFilterReq::default(), &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }

    /// Modify Current Tenant
    #[oai(path = "/", method = "put")]
    async fn modify(&self, mut modify_req: Json<IamTenantModifyReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamTenantServ::modify_item(&IamTenantServ::get_id_by_ctx(&ctx.0, &funs)?, &mut modify_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }
}
