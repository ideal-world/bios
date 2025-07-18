use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::{param::Path, param::Query};
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use bios_basic::rbum::dto::rbum_rel_dto::RbumRelBoneResp;

use crate::basic::dto::iam_set_dto::IamResSetTreeResp;
use crate::basic::serv::iam_res_serv::IamResServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_constants;
use crate::iam_enumeration::{IamRelKind, IamSetKind};
use bios_basic::helper::request_helper::try_set_real_ip_from_req_to_ctx;
use tardis::web::poem::Request;
#[derive(Clone, Default)]
pub struct IamCtResApi;

/// Tenant Console Res API
/// 租户控制台资源API
///
/// Note: the current res only supports sys level.
/// 注意：当前资源仅支持系统级别。
#[poem_openapi::OpenApi(prefix_path = "/ct/res", tag = "bios_basic::ApiTag::Tenant")]
impl IamCtResApi {
    /// Find Menu Tree
    /// 查找菜单树
    #[oai(path = "/tree", method = "get")]
    async fn get_menu_tree(&self, exts: Query<Option<String>>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamResSetTreeResp> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let set_id = IamSetServ::get_set_id_by_code(&IamSetServ::get_default_code(&IamSetKind::Res, ""), true, &funs, &ctx.0).await?;
        let result = IamSetServ::get_menu_tree(&set_id, exts.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Rel Roles By Res Id
    /// 根据资源ID查找关联角色
    #[oai(path = "/:id/role", method = "get")]
    async fn find_rel_roles(
        &self,
        id: Path<String>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<RbumRelBoneResp>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamResServ::find_from_simple_rel_roles(&IamRelKind::IamResRole, false, &id.0, desc_by_create.0, desc_by_update.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }
}
