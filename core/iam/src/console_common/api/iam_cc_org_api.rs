use bios_basic::rbum::dto::rbum_filer_dto::RbumSetTreeFilterReq;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use bios_basic::rbum::dto::rbum_set_dto::RbumSetTreeResp;
use bios_basic::rbum::rbum_enumeration::RbumSetCateLevelQueryKind;

use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_constants;

pub struct IamCcOrgApi;

/// Common Console Org API
///
/// Note: the current org only supports tenant level.
#[poem_openapi::OpenApi(prefix_path = "/cc/org", tag = "crate::iam_enumeration::Tag::Common")]
impl IamCcOrgApi {
    /// Find Org Tree By Current Tenant
    ///
    /// * Without parameters: Query the whole tree
    /// * ``parent_sys_code=true`` : query only the next level. This can be used to query level by level when the tree is too large
    #[oai(path = "/tree", method = "get")]
    async fn get_tree(&self, parent_sys_code: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<RbumSetTreeResp> {
        let funs = iam_constants::get_tardis_inst();
        let set_id = IamSetServ::get_set_id_by_code(&IamSetServ::get_default_org_code_by_tenant(&funs, &ctx.0)?, true, &funs, &ctx.0).await?;
        let result = IamSetServ::get_tree(
            &set_id,
            &mut RbumSetTreeFilterReq {
                fetch_cate_item: true,
                sys_codes: parent_sys_code.0.map(|parent_sys_code| vec![parent_sys_code]),
                sys_code_query_kind: Some(RbumSetCateLevelQueryKind::Sub),
                sys_code_query_depth: Some(1),
                ..Default::default()
            },
            &funs,
            &ctx.0,
        )
        .await?;
        TardisResp::ok(result)
    }
}
