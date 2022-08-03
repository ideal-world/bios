use bios_basic::rbum::dto::rbum_filer_dto::RbumSetTreeFilterReq;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use bios_basic::rbum::dto::rbum_set_dto::RbumSetTreeResp;

use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_constants;

pub struct IamCcOrgApi;

/// Common Console Org API
///
/// Note: the current org only supports tenant level.
#[poem_openapi::OpenApi(prefix_path = "/cc/org", tag = "crate::iam_enumeration::Tag::Common")]
impl IamCcOrgApi {
    /// Find Org Tree By Current Tenant
    #[oai(path = "/tree", method = "get")]
    async fn get_tree(&self, parent_cate_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<Vec<RbumSetTreeResp>> {
        let funs = iam_constants::get_tardis_inst();
        let set_id = IamSetServ::get_set_id_by_code(&IamSetServ::get_default_org_code_by_tenant(&funs, &ctx.0)?, true, &funs, &ctx.0).await?;
        let result = IamSetServ::get_tree(
            &set_id,
            parent_cate_id.0,
            &RbumSetTreeFilterReq {
                fetch_cate_item: true,
                ..Default::default()
            },
            &funs,
            &ctx.0,
        )
        .await?;
        TardisResp::ok(result)
    }
}
