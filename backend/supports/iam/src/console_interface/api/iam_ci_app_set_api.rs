use bios_basic::rbum::dto::rbum_set_dto::RbumSetTreeResp;
use bios_basic::rbum::helper::rbum_scope_helper::check_without_owner_and_unsafe_fill_ctx;
use bios_basic::rbum::rbum_enumeration::RbumSetCateLevelQueryKind;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_set_serv::RbumSetItemServ;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumSetItemFilterReq, RbumSetTreeFilterReq};
use bios_basic::rbum::dto::rbum_set_item_dto::RbumSetItemDetailResp;

use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_constants;
use crate::iam_enumeration::IamSetKind;
use bios_basic::helper::request_helper::try_set_real_ip_from_req_to_ctx;
use tardis::web::poem::Request;
#[derive(Clone, Default)]
pub struct IamCiAppSetApi;

/// Interface Console App Set API
/// 接口控制台应用集合API
#[poem_openapi::OpenApi(prefix_path = "/ci/apps", tag = "bios_basic::ApiTag::Interface")]
impl IamCiAppSetApi {
    /// Find App Set Items (App Or Account)
    /// 查找应用集合项（应用或账号）
    #[oai(path = "/item", method = "get")]
    async fn find_items(
        &self,
        cate_ids: Query<Option<String>>,
        item_ids: Query<Option<String>>,
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<RbumSetItemDetailResp>> {
        let funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        let ctx = IamCertServ::use_sys_or_tenant_ctx_unsafe(ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Apps, &funs, &ctx).await?;
        let result = RbumSetItemServ::find_detail_rbums(
            &RbumSetItemFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: false,
                    ..Default::default()
                },
                rel_rbum_item_disabled: Some(false),
                rel_rbum_item_can_not_exist: Some(true),
                rel_rbum_set_id: Some(set_id.clone()),
                rel_rbum_set_cate_ids: cate_ids.0.map(|ids| ids.split(',').map(|id| id.to_string()).collect::<Vec<String>>()),
                rel_rbum_item_ids: item_ids.0.map(|ids| ids.split(',').map(|id| id.to_string()).collect::<Vec<String>>()),
                ..Default::default()
            },
            None,
            None,
            &funs,
            &ctx,
        )
        .await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find App Tree By Current Tenant
    /// 查找应用树
    ///
    /// * Without parameters: Query the whole tree
    /// * ``parent_sys_code=true`` : query only the next level. This can be used to query level by level when the tree is too large
    /// * ``only_related=true`` : Invalidate the parent_sys_code parameter when this parameter is turned on, it is used to query only the tree nodes with related resources(including children nodes)
    ///  * 无参数：查询整个树
    /// * ``parent_sys_code=true``：仅查询下一级，当树太大时可以用来逐级查询
    /// * ``only_related=true``：打开此参数时失效parent_sys_code参数，用于查询只有相关资源的树节点（包括子节点）
    #[oai(path = "/tree", method = "get")]
    async fn get_tree(
        &self,
        parent_sys_code: Query<Option<String>>,
        only_related: Query<Option<bool>>,
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<RbumSetTreeResp> {
        let funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        let ctx = IamCertServ::use_sys_or_tenant_ctx_unsafe(ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Apps, &funs, &ctx).await?;
        let only_related = only_related.0.unwrap_or(false);
        let result = if only_related {
            IamSetServ::get_tree_with_auth_by_account(&set_id, &ctx.owner, &funs, &ctx).await?
        } else {
            IamSetServ::get_tree(
                &set_id,
                &mut RbumSetTreeFilterReq {
                    fetch_cate_item: true,
                    hide_item_with_disabled: true,
                    sys_codes: parent_sys_code.0.map(|parent_sys_code| vec![parent_sys_code]),
                    sys_code_query_kind: Some(RbumSetCateLevelQueryKind::Sub),
                    sys_code_query_depth: Some(1),
                    ..Default::default()
                },
                &funs,
                &ctx,
            )
            .await?
        };
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }
}
