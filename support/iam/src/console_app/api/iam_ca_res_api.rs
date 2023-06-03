use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::{param::Path, param::Query};
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use bios_basic::rbum::dto::rbum_rel_dto::RbumRelBoneResp;
use bios_basic::rbum::dto::rbum_set_dto::RbumSetTreeResp;

use crate::basic::serv::iam_res_serv::IamResServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_constants;
use crate::iam_enumeration::{IamRelKind, IamSetKind};

pub struct IamCaResApi;

/// App Console Res API
///
/// Note: the current res only supports sys level.
#[poem_openapi::OpenApi(prefix_path = "/ca/res", tag = "bios_basic::ApiTag::App")]
impl IamCaResApi {
    /// Find Menu Tree
    #[oai(path = "/tree", method = "get")]
    async fn get_menu_tree(&self, exts: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<RbumSetTreeResp> {
        let funs = iam_constants::get_tardis_inst();
        let set_id = IamSetServ::get_set_id_by_code(&IamSetServ::get_default_code(&IamSetKind::Res, ""), true, &funs, &ctx.0).await?;
        let result = IamSetServ::get_menu_tree(&set_id, exts.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Rel Roles By Res Id
    #[oai(path = "/:id/role", method = "get")]
    async fn find_rel_roles(
        &self,
        id: Path<String>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<Vec<RbumRelBoneResp>> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamResServ::find_from_simple_rel_roles(&IamRelKind::IamResRole, false, &id.0, desc_by_create.0, desc_by_update.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }
}
