use bios_basic::process::task_processor::TaskProcessor;
use itertools::Itertools;

use tardis::basic::error::TardisError;
use tardis::futures::future::join_all;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::dto::rbum_rel_dto::RbumRelBoneResp;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_filer_dto::IamRoleFilterReq;
use crate::basic::dto::iam_role_dto::{IamRoleAggAddReq, IamRoleAggCopyReq, IamRoleAggModifyReq, IamRoleDetailResp, IamRoleSummaryResp};
use crate::basic::serv::iam_app_serv::IamAppServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::iam_constants::RBUM_SCOPE_LEVEL_TENANT;
use crate::iam_constants::{self, RBUM_SCOPE_LEVEL_APP};

use crate::iam_enumeration::IamRoleKind;
use bios_basic::helper::request_helper::try_set_real_ip_from_req_to_ctx;
use tardis::web::poem::Request;
#[derive(Clone, Default)]
pub struct IamCtRoleApi;

/// Tenant Console Role API
/// 租户控制台角色API
#[poem_openapi::OpenApi(prefix_path = "/ct/role", tag = "bios_basic::ApiTag::Tenant")]
impl IamCtRoleApi {
    /// Add Role
    /// 添加角色
    #[oai(path = "/", method = "post")]
    async fn add(&self, is_app: Query<Option<bool>>, mut add_req: Json<IamRoleAggAddReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = if is_app.0.unwrap_or(false) {
            add_req.0.role.kind = Some(IamRoleKind::App);
            IamRoleServ::tenant_add_app_role_agg(&mut add_req.0, &funs, &ctx.0).await?
        } else {
            add_req.0.role.kind = Some(IamRoleKind::Tenant);
            IamRoleServ::add_role_agg(&mut add_req.0, &funs, &ctx.0).await?
        };
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// copy Role
    /// 复制角色
    #[oai(path = "/copy", method = "post")]
    async fn copy(&self, is_app: Query<Option<bool>>, mut copy_req: Json<IamRoleAggCopyReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = if is_app.0.unwrap_or(false) {
            copy_req.0.role.kind = Some(IamRoleKind::App);
            IamRoleServ::copy_tenant_add_app_role_agg(&mut copy_req.0, &funs, &ctx.0).await?
        } else {
            copy_req.0.role.kind = Some(IamRoleKind::Tenant);
            IamRoleServ::copy_role_agg(&mut copy_req.0, &funs, &ctx.0).await?
        };
        funs.commit().await?;
        ctx.0.execute_task().await?;
        if let Some(task_id) = TaskProcessor::get_task_id_with_ctx(&ctx.0).await? {
            TardisResp::accepted(task_id)
        } else {
            TardisResp::ok(result)
        }
    }

    /// Modify Role By Role Id
    /// 修改角色
    ///
    /// When code = 202, the return value is the asynchronous task id
    /// 当 code = 202 时，返回值为异步任务id
    #[oai(path = "/:id", method = "put")]
    async fn modify(&self, id: Path<String>, mut modify_req: Json<IamRoleAggModifyReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Option<String>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamRoleServ::modify_role_agg(&id.0, &mut modify_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        if let Some(task_id) = TaskProcessor::get_task_id_with_ctx(&ctx.0).await? {
            TardisResp::accepted(Some(task_id))
        } else {
            TardisResp::ok(None)
        }
    }

    /// Get Role By Role Id
    /// 根据角色ID获取角色
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamRoleDetailResp> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamRoleServ::get_item(&id.0, &IamRoleFilterReq::default(), &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Roles
    /// 查找角色
    #[allow(clippy::too_many_arguments)]
    #[oai(path = "/", method = "get")]
    async fn paginate(
        &self,
        id: Query<Option<String>>,
        name: Query<Option<String>>,
        app_id: Query<Option<String>>,
        in_base: Query<Option<bool>>,
        in_embed: Query<Option<bool>>,
        extend_role_id: Query<Option<String>>,
        with_sub: Query<Option<bool>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<TardisPage<IamRoleSummaryResp>> {
        let ctx = IamCertServ::try_use_app_ctx(ctx.0, app_id.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamRoleServ::paginate_items(
            &IamRoleFilterReq {
                basic: RbumBasicFilterReq {
                    ids: id.0.map(|id| vec![id]),
                    name: name.0,
                    with_sub_own_paths: with_sub.0.unwrap_or(false),
                    ..Default::default()
                },
                // kind: Some(IamRoleKind::Tenant),
                in_base: in_base.0,
                in_embed: in_embed.0,
                extend_role_id: extend_role_id.0,
                ..Default::default()
            },
            page_number.0,
            page_size.0,
            desc_by_create.0,
            desc_by_update.0,
            &funs,
            &ctx,
        )
        .await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Roles base app
    /// 聚合查询租户及基础项目角色
    #[oai(path = "/base_app", method = "get")]
    async fn find_role_base_app(
        &self,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<IamRoleSummaryResp>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let app_result = IamRoleServ::find_items(
            &IamRoleFilterReq {
                basic: RbumBasicFilterReq { ..Default::default() },
                kind: Some(IamRoleKind::App),
                in_base: Some(true),
                ..Default::default()
            },
            desc_by_create.0,
            desc_by_update.0,
            &funs,
            &ctx.0,
        )
        .await?;
        let tenant_result = IamRoleServ::find_items(
            &IamRoleFilterReq {
                basic: RbumBasicFilterReq { ..Default::default() },
                // kind: Some(IamRoleKind::Tenant),
                in_base: Some(false),
                ..Default::default()
            },
            desc_by_create.0,
            desc_by_update.0,
            &funs,
            &ctx.0,
        )
        .await?;
        ctx.0.execute_task().await?;
        let mut result = vec![];
        result.extend(tenant_result);
        result.extend(app_result);
        TardisResp::ok(result)
    }

    /// Delete Role By Role Id
    /// 删除角色
    #[oai(path = "/:id", method = "delete")]
    async fn delete(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Option<String>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamRoleServ::delete_item_with_all_rels(&id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        if let Some(task_id) = TaskProcessor::get_task_id_with_ctx(&ctx.0).await? {
            TardisResp::accepted(Some(task_id))
        } else {
            TardisResp::ok(None)
        }
    }

    /// Add Role Rel Account
    /// 添加角色关联账号
    #[oai(path = "/:id/account/:account_id", method = "put")]
    async fn add_rel_account(&self, id: Path<String>, account_id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamRoleServ::add_rel_account(&id.0, &account_id.0, Some(RBUM_SCOPE_LEVEL_TENANT), &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Batch add Role Rel Account
    /// 批量添加角色关联账号
    #[oai(path = "/:id/account/batch/:account_ids", method = "put")]
    async fn batch_add_rel_account(&self, id: Path<String>, account_ids: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        join_all(
            account_ids.0.split(',').map(|account_id| async { IamRoleServ::add_rel_account(&id.0, account_id, Some(RBUM_SCOPE_LEVEL_TENANT), &funs, &ctx.0).await }).collect_vec(),
        )
        .await
        .into_iter()
        .collect::<Result<Vec<()>, TardisError>>()?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Delete Role Rel Account
    /// 删除角色关联账号
    #[oai(path = "/:id/account/:account_id", method = "delete")]
    async fn delete_rel_account(&self, id: Path<String>, account_id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamRoleServ::delete_rel_account(&id.0, &account_id.0, Some(RBUM_SCOPE_LEVEL_TENANT), &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Batch delete Role Rel Account
    /// 批量删除角色关联账号
    #[oai(path = "/:id/account/batch/:account_ids", method = "delete")]
    async fn batch_delete_rel_account(&self, id: Path<String>, account_ids: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let split = account_ids.0.split(',').collect::<Vec<_>>();
        for s in split {
            IamRoleServ::delete_rel_account(&id.0, s, Some(RBUM_SCOPE_LEVEL_TENANT), &funs, &ctx.0).await?;
        }
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Count Rel Accounts By Role Id
    /// 根据角色ID统计关联账号
    #[oai(path = "/:id/account/total", method = "get")]
    async fn count_rel_accounts(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<u64> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamRoleServ::count_rel_accounts(&id.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Add Role Rel Res
    /// 添加角色关联资源
    #[oai(path = "/:id/res/:res_id", method = "put")]
    async fn add_rel_res(&self, id: Path<String>, res_id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamRoleServ::add_rel_res(&id.0, &res_id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Delete Role Rel Res
    /// 删除角色关联资源
    #[oai(path = "/:id/res/:res_id", method = "delete")]
    async fn delete_rel_res(&self, id: Path<String>, res_id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamRoleServ::delete_rel_res(&id.0, &res_id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Count Rel Res By Role Id
    /// 根据角色ID统计关联资源
    #[oai(path = "/:id/res/total", method = "get")]
    async fn count_rel_res(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<u64> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamRoleServ::count_rel_res(&id.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Rel Res By Role Id
    /// 根据角色ID获取关联资源
    #[oai(path = "/:id/res", method = "get")]
    async fn find_rel_res(
        &self,
        id: Path<String>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<RbumRelBoneResp>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamRoleServ::find_simple_rel_res(&id.0, desc_by_create.0, desc_by_update.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Add Role Rel Account
    /// 添加角色关联账号
    #[oai(path = "/:id/apps/account/batch", method = "put")]
    async fn batch_add_apps_rel_account(
        &self,
        id: Path<String>,
        app_ids: Query<String>,
        account_ids: Query<String>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let ctx = ctx.0;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let apps_split: Vec<&str> = app_ids.0.split(',').collect::<Vec<_>>();
        let account_split: Vec<&str> = account_ids.0.split(',').collect::<Vec<_>>();
        for app_id in apps_split {
            let mock_app_ctx = IamCertServ::try_use_app_ctx(ctx.clone(), Some(app_id.to_string()))?;
            for account_id in account_split.clone() {
                IamAppServ::add_rel_account(app_id, account_id, true, &funs, &mock_app_ctx).await?;
                IamRoleServ::add_rel_account(&id.0, account_id, Some(RBUM_SCOPE_LEVEL_APP), &funs, &mock_app_ctx).await?;
            }
        }
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Add Role Rel Account
    /// 添加角色关联账号
    #[oai(path = "/:id/apps/account/batch", method = "delete")]
    async fn batch_delete_apps_rel_account(
        &self,
        id: Path<String>,
        app_ids: Query<String>,
        account_ids: Query<String>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let ctx = ctx.0;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let apps_split: Vec<&str> = app_ids.0.split(',').collect::<Vec<_>>();
        let account_split: Vec<&str> = account_ids.0.split(',').collect::<Vec<_>>();
        for app_id in apps_split {
            let mock_app_ctx = IamCertServ::try_use_app_ctx(ctx.clone(), Some(app_id.to_string()))?;
            for account_id in account_split.clone() {
                IamRoleServ::delete_rel_account(&id.0, account_id, Some(RBUM_SCOPE_LEVEL_APP), &funs, &mock_app_ctx).await?;
            }
        }
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(Void {})
    }
}
