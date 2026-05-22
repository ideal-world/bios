use std::collections::HashMap;

use bios_basic::rbum::dto::rbum_set_dto::RbumSetTreeResp;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumSetTreeFilterReq};
use bios_basic::rbum::dto::rbum_set_item_dto::RbumSetItemDetailResp;
use bios_basic::rbum::rbum_enumeration::RbumSetCateLevelQueryKind;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_filer_dto::IamTenantFilterReq;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::iam_constants;
use crate::iam_enumeration::IamSetKind;
use bios_basic::helper::request_helper::try_set_real_ip_from_req_to_ctx;
use tardis::web::poem::Request;
#[derive(Clone, Default)]
pub struct IamCcAppSetApi;

/// Tenant Console App Set API
/// 租户控制台应用集合API
#[poem_openapi::OpenApi(prefix_path = "/cc/apps", tag = "bios_basic::ApiTag::Common")]
impl IamCcAppSetApi {
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
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<RbumSetTreeResp> {
        let funs = iam_constants::get_tardis_inst();
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

    /// Find App Tree For All Tenants (or selected tenants if ``tenant_ids`` is set)
    /// 返回各租户的应用树；若传入 ``tenant_ids``（逗号分隔的租户 ID）则仅返回这些租户，否则返回全部启用租户
    ///
    /// Other query parameters behave the same as ``GET /cs/apps/tree`` (except ``tenant_id`` replaced by ``tenant_ids``).
    /// 其余查询参数与 ``GET /cs/apps/tree`` 一致（``tenant_id`` 在此处为 ``tenant_ids``）。
    #[oai(path = "/tree/all", method = "get")]
    async fn get_tree_all(
        &self,
        parent_sys_code: Query<Option<String>>,
        only_related: Query<Option<bool>>,
        tenant_ids: Query<Option<String>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<HashMap<String, RbumSetTreeResp>> {
        let funs = iam_constants::get_tardis_inst();
        let only_related = only_related.0.unwrap_or(false);
        let mut result = HashMap::new();
        let ext_ctx = ctx.0;

        let specific_tenant_ids = tenant_ids.0.map(|s| s.split(',').map(String::from).collect::<Vec<_>>());

        let sys_ctx = IamCertServ::use_sys_ctx_unsafe(ext_ctx.clone())?;
        try_set_real_ip_from_req_to_ctx(request, &sys_ctx).await?;

        let tenant_id_list: Vec<String> = if let Some(ids) = specific_tenant_ids {
            ids
        } else {
            IamTenantServ::find_items(
                &IamTenantFilterReq {
                    basic: RbumBasicFilterReq {
                        ignore_scope: true,
                        own_paths: Some("".to_string()),
                        with_sub_own_paths: true,
                        enabled: Some(true),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                None,
                None,
                &funs,
                &sys_ctx,
            )
            .await?
            .into_iter()
            .map(|t| t.id)
            .collect()
        };

        let parent_sys_code_q = parent_sys_code.0.clone();
        for tid in tenant_id_list {
            let t_ctx = IamCertServ::use_tenant_ctx(sys_ctx.clone(), &tid)?;
            let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Apps, &funs, &t_ctx).await?;
            let tree = if only_related {
                IamSetServ::get_tree_with_auth_by_account_opt(&set_id, &t_ctx.owner, &funs, &t_ctx).await?
            } else {
                Some(
                    IamSetServ::get_tree(
                        &set_id,
                        &mut RbumSetTreeFilterReq {
                            fetch_cate_item: true,
                            hide_item_with_disabled: true,
                            sys_codes: parent_sys_code_q.clone().map(|parent_sys_code| vec![parent_sys_code]),
                            sys_code_query_kind: Some(RbumSetCateLevelQueryKind::Sub),
                            sys_code_query_depth: Some(1),
                            ..Default::default()
                        },
                        &funs,
                        &t_ctx,
                    )
                    .await?
                )
            };
            if let Some(tree) = tree {
                result.insert(tid, tree);
            }
        }
        ext_ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find App Set Items (App Or Account)
    /// 查找应用集合项（应用或账户）
    #[oai(path = "/item", method = "get")]
    async fn find_items(
        &self,
        cate_id: Query<Option<String>>,
        item_id: Query<Option<String>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<RbumSetItemDetailResp>> {
        let funs = iam_constants::get_tardis_inst();
        let ctx = IamCertServ::use_sys_or_tenant_ctx_unsafe(ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Apps, &funs, &ctx).await?;
        let result = IamSetServ::find_set_items(Some(set_id), cate_id.0, item_id.0, None, false, None, &funs, &ctx).await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }
}
