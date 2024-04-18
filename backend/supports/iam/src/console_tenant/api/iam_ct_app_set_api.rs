use bios_basic::rbum::dto::rbum_set_dto::RbumSetTreeResp;
use itertools::Itertools;
use tardis::basic::error::TardisError;

use tardis::futures::future::join_all;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use bios_basic::rbum::dto::rbum_filer_dto::RbumSetTreeFilterReq;
use bios_basic::rbum::dto::rbum_set_item_dto::RbumSetItemDetailResp;
use bios_basic::rbum::rbum_enumeration::RbumSetCateLevelQueryKind;

use crate::basic::dto::iam_set_dto::{IamSetCateAddReq, IamSetCateModifyReq, IamSetItemAddReq, IamSetItemWithDefaultSetAddReq};
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_constants;
use crate::iam_enumeration::IamSetKind;
use bios_basic::helper::request_helper::try_set_real_ip_from_req_to_ctx;
use tardis::web::poem::Request;
#[derive(Clone, Default)]
pub struct IamCtAppSetApi;

/// Tenant Console App Set API
/// 租户控制台应用集API
#[poem_openapi::OpenApi(prefix_path = "/ct/apps", tag = "bios_basic::ApiTag::Tenant")]
impl IamCtAppSetApi {
    /// Add App Set Cate
    /// 添加应用集分类
    #[oai(path = "/cate", method = "post")]
    async fn add_cate(&self, add_req: Json<IamSetCateAddReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Apps, &funs, &ctx.0).await?;
        let result = IamSetServ::add_set_cate(&set_id, &add_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Modify App Set Cate By App Cate Id
    /// 修改应用集分类
    #[oai(path = "/cate/:id", method = "put")]
    async fn modify_set_cate(&self, id: Path<String>, modify_req: Json<IamSetCateModifyReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSetServ::modify_set_cate(&id.0, &modify_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Find App Tree By Current Tenant
    ///
    /// * Without parameters: Query the whole tree
    /// * ``parent_sys_code=true`` : query only the next level. This can be used to query level by level when the tree is too large
    /// * ``only_related=true`` : Invalidate the parent_sys_code parameter when this parameter is turned on, it is used to query only the tree nodes with related resources(including children nodes)
    ///
    ///  根据当前租户查询应用树
    ///
    /// * 无参数：查询整个树
    /// * ``parent_sys_code=true``：查询下一级，当树太大时可以用来逐级查询
    /// * ``only_related=true``：打开此参数时失效parent_sys_code参数，用来查询只有相关资源的树节点（包括子节点）
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

    /// Delete App Set Cate By Org Cate Id
    /// 删除应用集分类
    #[oai(path = "/cate/:id", method = "delete")]
    async fn delete_cate(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSetServ::delete_set_cate(&id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Add App Set Item (App Or Account)
    /// 添加应用集项（应用或账号）
    #[oai(path = "/item", method = "put")]
    async fn add_set_item(&self, add_req: Json<IamSetItemWithDefaultSetAddReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let ctx = IamCertServ::use_sys_or_tenant_ctx_unsafe(ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Apps, &funs, &ctx).await?;
        let result = IamSetServ::add_set_item(
            &IamSetItemAddReq {
                set_id,
                set_cate_id: add_req.set_cate_id.clone().unwrap_or_default(),
                sort: add_req.sort,
                rel_rbum_item_id: add_req.rel_rbum_item_id.to_string(),
            },
            &funs,
            &ctx,
        )
        .await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Batch Add App Set Item (App Or Account)
    /// 批量添加应用集项（应用或账号）
    #[oai(path = "/item/batch", method = "put")]
    async fn batch_add_set_item(&self, add_req: Json<IamSetItemWithDefaultSetAddReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Vec<String>> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let ctx = IamCertServ::use_sys_or_tenant_ctx_unsafe(ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Apps, &funs, &ctx).await?;
        let result = join_all(
            add_req
                .rel_rbum_item_id
                .split(',')
                .map(|item_id| async {
                    IamSetServ::add_set_item(
                        &IamSetItemAddReq {
                            set_id: set_id.clone(),
                            set_cate_id: add_req.set_cate_id.clone().unwrap_or_default(),
                            sort: add_req.sort,
                            rel_rbum_item_id: item_id.to_string(),
                        },
                        &funs,
                        &ctx,
                    )
                    .await
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

    /// Find App Set Items (App Or Account)
    /// 查找应用集项（应用或账号）
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

    /// batch Delete App Set Item (App Or Account) By App Set Item Id
    /// 根据应用集项ID批量删除应用集项（应用或账号）
    #[oai(path = "/item/batch/:ids", method = "delete")]
    async fn batch_delete_item(&self, ids: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let ctx = IamCertServ::use_sys_or_tenant_ctx_unsafe(ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        for id in ids.split(',').collect::<Vec<_>>() {
            IamSetServ::delete_set_item(id, &funs, &ctx).await?;
        }
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Check App Scope with Account
    /// 检查应用范围
    #[oai(path = "/scope", method = "get")]
    async fn check_scope(&self, app_id: Query<String>, account_id: Query<Option<String>>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<bool> {
        let funs = iam_constants::get_tardis_inst();
        let ctx = IamCertServ::use_sys_or_tenant_ctx_unsafe(ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Apps, &funs, &ctx).await?;
        let result = IamSetServ::check_scope(&app_id.0, &account_id.0.unwrap_or_else(|| ctx.owner.clone()), &set_id, &funs, &ctx).await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }
}
