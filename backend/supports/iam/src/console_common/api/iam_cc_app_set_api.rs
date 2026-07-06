use std::collections::HashMap;

use bios_basic::rbum::dto::rbum_set_dto::RbumSetTreeResp;
use itertools::Itertools;
use tardis::basic::error::TardisError;
use tardis::futures::future::join_all;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumSetTreeFilterReq};
use bios_basic::rbum::dto::rbum_set_item_dto::RbumSetItemDetailResp;
use bios_basic::rbum::rbum_enumeration::RbumSetCateLevelQueryKind;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_filer_dto::IamTenantFilterReq;
use crate::basic::dto::iam_set_dto::IamSetItemWithDefaultSetAddReq;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::iam_constants;
use crate::iam_config::IamConfig;
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
    /// * Platform context (``own_paths`` is empty): returns merged tree with platform root -> tenants -> app cates; use ``tenant_id`` when ``parent_sys_code`` is an app cate sys_code
    ///  * 无参数：查询整个树
    /// * ``parent_sys_code=true``：仅查询下一级，当树太大时可以用来逐级查询
    /// * ``only_related=true``：打开此参数时失效parent_sys_code参数，用于查询只有相关资源的树节点（包括子节点）
    /// * 平台上下文（``own_paths`` 为空）：返回平台根 -> 租户 -> 产品组的合并树；当 ``parent_sys_code`` 为产品组 sys_code 时需同时传入 ``tenant_id``
    #[oai(path = "/tree", method = "get")]
    async fn get_tree(
        &self,
        parent_sys_code: Query<Option<String>>,
        only_related: Query<Option<bool>>,
        tenant_id: Query<Option<String>>,
        tenant_ids: Query<Option<String>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<RbumSetTreeResp> {
        let funs = iam_constants::get_tardis_inst();
        let ctx = IamCertServ::use_sys_or_tenant_ctx_unsafe(ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let only_related = only_related.0.unwrap_or(false);
        let result = if ctx.own_paths.is_empty() {
            let specific_tenant_ids = tenant_ids.0.map(|s| s.split(',').map(String::from).collect::<Vec<_>>());
            let tenants = if let Some(ids) = specific_tenant_ids {
                IamTenantServ::find_items(
                    &IamTenantFilterReq {
                        basic: RbumBasicFilterReq {
                            ignore_scope: true,
                            ids: Some(ids),
                            enabled: Some(true),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    Some(true),
                    None,
                    &funs,
                    &ctx,
                )
                .await?
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
                    Some(true),
                    None,
                    &funs,
                    &ctx,
                )
                .await?
            };
            IamSetServ::get_platform_apps_tree(&tenants, parent_sys_code.0, tenant_id.0, only_related, &funs, &ctx).await?
        } else {
            let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Apps, &funs, &ctx).await?;
            if only_related {
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
            }
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
        tenant_id: Query<Option<String>>,
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

        let tenants = if let Some(ids) = specific_tenant_ids {
            IamTenantServ::find_items(
                &IamTenantFilterReq {
                    basic: RbumBasicFilterReq {
                        ignore_scope: true,
                        ids: Some(ids),
                        enabled: Some(true),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                Some(true),
                None,
                &funs,
                &sys_ctx,
            )
            .await?
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
                Some(true),
                None,
                &funs,
                &sys_ctx,
            )
            .await?
        };
        let platform_apps_tree = IamSetServ::get_platform_apps_tree(&tenants, parent_sys_code.0, tenant_id.0, only_related, &funs, &sys_ctx).await?;
        result.insert(funs.conf::<IamConfig>().platform_apps_tree_root_id.clone(), platform_apps_tree);

        ext_ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find App Set Items (App Or Account)
    /// 查找应用集合项（应用或账户）
    ///
    /// * Platform context: ``cate_id=__iam_platform_apps_root__`` queries platform root bindings; tenant virtual root uses ``cate_id={tenant_id}`` with optional ``tenant_id``
    /// * 平台上下文：``cate_id=__iam_platform_apps_root__`` 查询平台根绑定；租户虚拟根使用 ``cate_id={tenant_id}`` 并可选 ``tenant_id``
    #[oai(path = "/item", method = "get")]
    async fn find_items(
        &self,
        cate_id: Query<Option<String>>,
        item_id: Query<Option<String>>,
        tenant_id: Query<Option<String>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<RbumSetItemDetailResp>> {
        let funs = iam_constants::get_tardis_inst();
        let ctx = IamCertServ::use_sys_or_tenant_ctx_unsafe(ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let result = IamSetServ::find_apps_set_items(cate_id.0, item_id.0, tenant_id.0, &funs, &ctx).await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Add App Set Item (App Or Account)
    /// 添加应用集合项（应用或账号）
    ///
    /// * Platform root: ``set_cate_id=__iam_platform_apps_root__`` or empty
    /// * Tenant cate on platform context: ``set_cate_id={cate_id}`` with ``tenant_id={tenant_id}``
    /// * 平台根：``set_cate_id=__iam_platform_apps_root__`` 或留空；平台上下文操作租户节点时需传 ``tenant_id``
    #[oai(path = "/item", method = "put")]
    async fn add_set_item(&self, add_req: Json<IamSetItemWithDefaultSetAddReq>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let ctx = IamCertServ::use_sys_or_tenant_ctx_unsafe(ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let result = IamSetServ::add_apps_set_item(add_req.0.set_cate_id, tenant_id.0, add_req.0.sort, add_req.0.rel_rbum_item_id, &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Batch Add App Set Item (App Or Account)
    /// 批量添加应用集项（应用或账号）
    #[oai(path = "/item/batch", method = "put")]
    async fn batch_add_set_item(
        &self,
        add_req: Json<IamSetItemWithDefaultSetAddReq>,
        tenant_id: Query<Option<String>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<String>> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let ctx = IamCertServ::use_sys_or_tenant_ctx_unsafe(ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let tenant_id = tenant_id.0;
        let set_cate_id = add_req.set_cate_id.clone();
        let sort = add_req.sort;
        let result = join_all(
            add_req
                .rel_rbum_item_id
                .split(',')
                .map(|item_id| async {
                    IamSetServ::add_apps_set_item(set_cate_id.clone(), tenant_id.clone(), sort, item_id.to_string(), &funs, &ctx).await
                })
                .collect_vec(),
        )
        .await
        .into_iter()
        .collect::<Result<Vec<String>, TardisError>>()?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Delete App Set Item (App Or Account) By App Set Item Id
    /// 根据应用集项ID删除应用集项（应用或账号）
    #[oai(path = "/item/:id", method = "delete")]
    async fn delete_item(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let ctx = IamCertServ::use_sys_or_tenant_ctx_unsafe(ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        IamSetServ::delete_set_item(&id.0, &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Batch Delete App Set Item (App Or Account) By App Set Item Id
    /// 根据应用集项ID批量删除应用集项（应用或账号）
    #[oai(path = "/item/batch/:ids", method = "delete")]
    async fn batch_delete_item(&self, ids: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let ctx = IamCertServ::use_sys_or_tenant_ctx_unsafe(ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        for id in ids.0.split(',') {
            IamSetServ::delete_set_item(id, &funs, &ctx).await?;
        }
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(Void {})
    }
}
