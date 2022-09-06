use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use bios_basic::rbum::dto::rbum_set_dto::RbumSetTreeResp;

use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_constants;
use crate::iam_enumeration::IamSetKind;

pub struct IamCcResApi;

/// Common Console Res API
///
#[poem_openapi::OpenApi(prefix_path = "/cc/res", tag = "crate::iam_enumeration::Tag::Common")]
impl IamCcResApi {
    /// Find Menu Tree
    #[oai(path = "/tree", method = "get")]
    async fn get_menu_tree(&self, ctx: TardisContextExtractor) -> TardisApiResult<RbumSetTreeResp> {
        let funs = iam_constants::get_tardis_inst();
        let set_id = IamSetServ::get_set_id_by_code(&IamSetServ::get_default_code(&IamSetKind::Res, ""), true, &funs, &ctx.0).await?;
        let result = IamSetServ::get_menu_tree_by_roles(&set_id, &ctx.0.roles, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }
}
