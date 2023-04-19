use bios_basic::process::task_processor::TaskProcessor;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_filer_dto::IamTenantFilterReq;
use crate::basic::dto::iam_platform_dto::{IamPlatformAggDetailResp, IamPlatformAggModifyReq};
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_platform_serv::IamPlatformServ;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::iam_constants;

pub struct IamCsPlatformApi;

/// System Console Platform API
#[poem_openapi::OpenApi(prefix_path = "/cs/platform", tag = "bios_basic::ApiTag::System")]
impl IamCsPlatformApi {
    
    /// modify Platform config
    #[oai(path = "/", method = "put")]
    async fn modify(&self, modify_req: Json<IamPlatformAggModifyReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamPlatformServ::modify_platform_agg(&modify_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Get Platform config
    #[oai(path = "/", method = "get")]
    async fn get(&self, ctx: TardisContextExtractor) -> TardisApiResult<IamPlatformAggDetailResp> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamPlatformServ::get_platform_agg(&funs, &ctx.0).await?;
        TardisResp::ok(result)
    }
}
