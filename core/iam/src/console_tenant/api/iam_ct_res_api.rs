use bios_basic::rbum::dto::rbum_rel_agg_dto::RbumRelAggResp;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{param::Path, param::Query, OpenApi};
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use bios_basic::rbum::dto::rbum_set_cate_dto::RbumSetTreeResp;

use crate::basic::serv::iam_res_serv::IamResServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_constants;

pub struct IamCtResApi;

/// Tenant Console Res API
///
/// Note: the current res only supports sys level.
#[OpenApi(prefix_path = "/ct/res", tag = "crate::iam_enumeration::Tag::Tenant")]
impl IamCtResApi {
    /// Find Res Cates
    #[oai(path = "/cate", method = "get")]
    async fn find_cates(&self, sys_res: Query<Option<bool>>, cxt: TardisContextExtractor) -> TardisApiResult<Vec<RbumSetTreeResp>> {
        let funs = iam_constants::get_tardis_inst();
        let set_id = if sys_res.0.unwrap_or(false) {
            IamSetServ::get_set_id_by_code(&IamSetServ::get_default_res_code_by_own_paths(""), &funs, &cxt.0).await?
        } else {
            IamSetServ::get_default_set_id_by_cxt(false, &funs, &cxt.0).await?
        };
        let result = IamSetServ::find_set_cates(&set_id, &funs, &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Find Rel Roles By Res Id
    #[oai(path = "/:id/role", method = "get")]
    async fn find_rel_roles(
        &self,
        id: Path<String>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        cxt: TardisContextExtractor,
    ) -> TardisApiResult<Vec<RbumRelAggResp>> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamResServ::find_rel_roles(&id.0, false, desc_by_create.0, desc_by_update.0, &funs, &cxt.0).await?;
        TardisResp::ok(result)
    }
}
