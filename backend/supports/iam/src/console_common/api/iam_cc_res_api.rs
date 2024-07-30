use std::collections::HashMap;

use crate::basic::dto::iam_res_dto::{IamResAppReq, IamResSummaryResp};
use crate::basic::serv::iam_res_serv::IamResServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_constants;
use crate::iam_enumeration::IamSetKind;
use bios_basic::helper::request_helper::try_set_real_ip_from_req_to_ctx;
use bios_basic::rbum::dto::rbum_set_dto::{RbumSetTreeCateResp, RbumSetTreeResp};
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::web::Json;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

#[derive(Clone, Default)]
pub struct IamCcResApi;

/// Common Console Res API
/// 通用控制台资源API
#[poem_openapi::OpenApi(prefix_path = "/cc/res", tag = "bios_basic::ApiTag::Common")]
impl IamCcResApi {
    /// Find Menu Tree
    /// 查找菜单树
    #[oai(path = "/tree", method = "get")]
    async fn get_menu_tree(&self, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<RbumSetTreeResp> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let set_id = IamSetServ::get_set_id_by_code(&IamSetServ::get_default_code(&IamSetKind::Res, ""), true, &funs, &ctx.0).await?;
        let result = IamSetServ::get_menu_tree_by_roles(&set_id, &ctx.0.roles, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// build Menu Tree
    /// 构造菜单树
    #[oai(path = "/tree/build", method = "get")]
    async fn build_menu_tree(&self, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<RbumSetTreeCateResp> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let set_id = IamSetServ::get_set_id_by_code(&IamSetServ::get_default_code(&IamSetKind::Res, ""), true, &funs, &ctx.0).await?;
        let result = IamSetServ::get_menu_tree_by_roles(&set_id, &ctx.0.roles, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result.to_trees())
    }

    /// Find res by apps
    /// 根据应用查找资源
    #[oai(path = "/res", method = "get")]
    async fn get_res_by_app(&self, app_ids: Query<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<HashMap<String, Vec<IamResSummaryResp>>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let ids = app_ids.0.split(',').map(|s| s.to_string()).collect();
        let result = IamResServ::get_res_by_app_code(ids, None, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find res by apps and code
    /// 根据应用和资源编码查找资源
    #[oai(path = "/res", method = "put")]
    async fn get_res_by_app_code(&self, res_req: Json<IamResAppReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<HashMap<String, Vec<IamResSummaryResp>>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamResServ::get_res_by_app_code(res_req.0.app_ids, Some(res_req.0.res_codes), &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }
}
