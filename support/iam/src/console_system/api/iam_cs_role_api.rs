use bios_basic::process::task_processor::TaskProcessor;
use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::dto::rbum_rel_dto::RbumRelBoneResp;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use crate::basic::dto::iam_filer_dto::IamRoleFilterReq;
use crate::basic::dto::iam_role_dto::{IamRoleAggAddReq, IamRoleAggModifyReq, IamRoleDetailResp, IamRoleSummaryResp};
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::iam_constants;
use crate::iam_enumeration::IamRoleKind;

#[derive(Clone, Default)]
pub struct IamCsRoleApi;

/// System Console Role API
#[poem_openapi::OpenApi(prefix_path = "/cs/role", tag = "bios_basic::ApiTag::System")]
impl IamCsRoleApi {
    /// Add Role
    #[oai(path = "/", method = "post")]
    async fn add(&self, tenant_id: Query<Option<String>>, mut add_req: Json<IamRoleAggAddReq>, ctx: TardisContextExtractor) -> TardisApiResult<String> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        add_req.0.role.kind = Some(IamRoleKind::System);
        let result = IamRoleServ::add_role_agg(&mut add_req.0, &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Modify Role By Role Id
    ///
    /// When code = 202, the return value is the asynchronous task id
    #[oai(path = "/:id", method = "put")]
    async fn modify(
        &self,
        id: Path<String>,
        tenant_id: Query<Option<String>>,
        mut modify_req: Json<IamRoleAggModifyReq>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<Option<String>> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamRoleServ::modify_role_agg(&id.0, &mut modify_req.0, &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        if let Some(task_id) = TaskProcessor::get_task_id_with_ctx(&ctx).await? {
            TardisResp::accepted(Some(task_id))
        } else {
            TardisResp::ok(None)
        }
    }

    /// Get Role By Role Id
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<IamRoleDetailResp> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamRoleServ::get_item(&id.0, &IamRoleFilterReq::default(), &funs, &ctx).await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Roles
    #[oai(path = "/", method = "get")]
    async fn paginate(
        &self,
        id: Query<Option<String>>,
        name: Query<Option<String>>,
        tenant_id: Query<Option<String>>,
        with_sub: Query<Option<bool>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<IamRoleSummaryResp>> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamRoleServ::paginate_items(
            &IamRoleFilterReq {
                basic: RbumBasicFilterReq {
                    ids: id.0.map(|id| vec![id]),
                    name: name.0,
                    with_sub_own_paths: with_sub.0.unwrap_or(false),
                    ..Default::default()
                },
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

    /// Delete Role By Role Id
    #[oai(path = "/:id", method = "delete")]
    async fn delete(&self, id: Path<String>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamRoleServ::delete_item_with_all_rels(&id.0, &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Add Role Rel Account
    #[oai(path = "/:id/account/:account_id", method = "put")]
    async fn add_rel_account(&self, id: Path<String>, account_id: Path<String>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamRoleServ::add_rel_account(&id.0, &account_id.0, None, &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Batch Add Role Rel Account
    #[oai(path = "/:id/account/batch/:account_id", method = "put")]
    async fn batch_add_rel_account(&self, id: Path<String>, account_ids: Path<String>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let split = account_ids.0.split(',').collect::<Vec<_>>();
        for s in split {
            IamRoleServ::add_rel_account(&id.0, s, None, &funs, &ctx).await?;
        }
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Delete Role Rel Account
    #[oai(path = "/:id/account/:account_id", method = "delete")]
    async fn delete_rel_account(&self, id: Path<String>, account_id: Path<String>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamRoleServ::delete_rel_account(&id.0, &account_id.0, None, &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Batch delete Role Rel Account
    #[oai(path = "/:id/account/batch/:account_ids", method = "delete")]
    async fn batch_delete_rel_account(&self, id: Path<String>, account_ids: Path<String>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let split = account_ids.0.split(',').collect::<Vec<_>>();
        for s in split {
            IamRoleServ::delete_rel_account(&id.0, s, None, &funs, &ctx).await?;
        }
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Count Rel Accounts By Role Id
    #[oai(path = "/:id/account/total", method = "get")]
    async fn count_rel_accounts(&self, id: Path<String>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<u64> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamRoleServ::count_rel_accounts(&id.0, &funs, &ctx).await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Add Role Rel Res
    #[oai(path = "/:id/res/:res_id", method = "put")]
    async fn add_rel_res(&self, id: Path<String>, res_id: Path<String>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamRoleServ::add_rel_res(&id.0, &res_id.0, &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Delete Role Rel Res
    #[oai(path = "/:id/res/:res_id", method = "delete")]
    async fn delete_rel_res(&self, id: Path<String>, res_id: Path<String>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamRoleServ::delete_rel_res(&id.0, &res_id.0, &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Count Rel Res By Role Id
    #[oai(path = "/:id/res/total", method = "get")]
    async fn count_rel_res(&self, id: Path<String>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<u64> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamRoleServ::count_rel_res(&id.0, &funs, &ctx).await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Rel Res By Role Id
    #[oai(path = "/:id/res", method = "get")]
    async fn find_rel_res(
        &self,
        id: Path<String>,
        tenant_id: Query<Option<String>>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<Vec<RbumRelBoneResp>> {
        let ctx = if let Some(tenant_id) = tenant_id.0 {
            IamCertServ::try_use_tenant_ctx(ctx.0, Some(tenant_id))?
        } else {
            IamCertServ::use_sys_ctx_unsafe(ctx.0)?
        };
        let funs = iam_constants::get_tardis_inst();
        let result = IamRoleServ::find_simple_rel_res(&id.0, desc_by_create.0, desc_by_update.0, &funs, &ctx).await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }
}
