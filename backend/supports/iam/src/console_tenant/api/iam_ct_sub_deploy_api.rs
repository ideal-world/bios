use bios_basic::process::task_processor::TaskProcessor;

use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use itertools::Itertools;
use tardis::basic::error::TardisError;
use tardis::futures::future::join_all;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::payload::Attachment;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};
use tardis::web::{poem, poem_openapi};

use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_filer_dto::{IamSubDeployFilterReq, IamSubDeployHostFilterReq, IamSubDeployLicenseFilterReq};
use crate::basic::dto::iam_sub_deploy_dto::{IamSubDeployAddReq, IamSubDeployDetailResp, IamSubDeployModifyReq, IamSubDeploySummaryResp};
use crate::basic::dto::iam_sub_deploy_host_dto::{IamSubDeployHostAddReq, IamSubDeployHostDetailResp, IamSubDeployHostModifyReq};
use crate::basic::dto::iam_sub_deploy_license_dto::{IamSubDeployLicenseAddReq, IamSubDeployLicenseDetailResp, IamSubDeployLicenseModifyReq};
use crate::basic::serv::iam_sub_deploy_serv::{IamSubDeployHostServ, IamSubDeployLicenseServ, IamSubDeployServ};
use crate::iam_constants::{self, RBUM_SCOPE_LEVEL_TENANT};
use crate::iam_enumeration::{IamRelKind, IamSubDeployHostKind};

use bios_basic::helper::request_helper::try_set_real_ip_from_req_to_ctx;
use tardis::web::poem::Request;
#[derive(Clone, Default)]
pub struct IamCtSubDeployApi;
#[derive(Clone, Default)]
pub struct IamCtSubDeployHostApi;
#[derive(Clone, Default)]
pub struct IamCtSubDeployLicenseApi;

/// Tenant Console Sub Deploy API
/// 租户控制台 二级部署 API
#[poem_openapi::OpenApi(prefix_path = "/ct/sub_deploy", tag = "bios_basic::ApiTag::Tenant")]
impl IamCtSubDeployApi {
    #[oai(path = "/", method = "post")]
    async fn add(&self, mut add_req: Json<IamSubDeployAddReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        add_req.0.scope_level = Some(RBUM_SCOPE_LEVEL_TENANT);
        let result = IamSubDeployServ::add_item(&mut add_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    #[oai(path = "/:id", method = "put")]
    async fn modify(&self, id: Path<String>, mut modify_req: Json<IamSubDeployModifyReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSubDeployServ::modify_item(&id.0, &mut modify_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Get Sub Deploy By Sub Deploy Id
    /// 根据二级部署ID获取二级部署
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamSubDeployDetailResp> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamSubDeployServ::get_item(&id.0, &IamSubDeployFilterReq::default(), &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Sub Deploy
    /// 查找二级部署
    #[allow(clippy::too_many_arguments)]
    #[oai(path = "/", method = "get")]
    async fn paginate(
        &self,
        ids: Query<Option<String>>,
        name: Query<Option<String>>,
        code: Query<Option<String>>,
        province: Query<Option<String>>,
        access_url: Query<Option<String>>,
        enabled: Query<Option<bool>>,
        with_sub: Query<Option<bool>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<TardisPage<IamSubDeploySummaryResp>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamSubDeployServ::paginate_items(
            &IamSubDeployFilterReq {
                basic: RbumBasicFilterReq {
                    ids: ids.0.map(|ids| ids.split(',').map(|id| id.to_string()).collect::<Vec<String>>()),
                    name: name.0,
                    code: code.0,
                    with_sub_own_paths: with_sub.0.unwrap_or(false),
                    enabled: enabled.0,
                    ..Default::default()
                },
                province: province.0,
                access_url: access_url.0,
                ..Default::default()
            },
            page_number.0,
            page_size.0,
            desc_by_create.0,
            desc_by_update.0,
            &funs,
            &ctx.0,
        )
        .await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Delete Sub Deploy By Sub Deploy Id
    /// 删除二级部署
    #[oai(path = "/:id", method = "delete")]
    async fn delete(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Option<String>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSubDeployServ::delete_item_with_ext_rel(&id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        if let Some(task_id) = TaskProcessor::get_task_id_with_ctx(&ctx.0).await? {
            TardisResp::accepted(Some(task_id))
        } else {
            TardisResp::ok(None)
        }
    }

    /// Add Sub Deploy Rel Account
    /// 添加二级部署关联账号
    #[oai(path = "/:id/account/:account_id", method = "put")]
    async fn add_rel_account(&self, id: Path<String>, account_id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSubDeployServ::add_rel_account(&id.0, &account_id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Batch add Sub Deploy Rel Account
    /// 批量添加二级部署关联账号
    #[oai(path = "/:id/account/batch/:account_ids", method = "put")]
    async fn batch_add_rel_account(&self, id: Path<String>, account_ids: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        join_all(account_ids.0.split(',').map(|account_id| async { IamSubDeployServ::add_rel_account(&id.0, account_id, &funs, &ctx.0).await }).collect_vec())
            .await
            .into_iter()
            .collect::<Result<Vec<()>, TardisError>>()?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Delete Sub Deploy Rel Account
    /// 删除二级部署关联账号
    #[oai(path = "/:id/account/:account_id", method = "delete")]
    async fn delete_rel_account(&self, id: Path<String>, account_id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSubDeployServ::delete_rel_account(&id.0, &account_id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Batch delete Sub Deploy Rel Account
    /// 批量删除二级部署关联账号
    #[oai(path = "/:id/account/batch/:account_ids", method = "delete")]
    async fn batch_delete_rel_account(&self, id: Path<String>, account_ids: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let split = account_ids.0.split(',').collect::<Vec<_>>();
        for s in split {
            IamSubDeployServ::delete_rel_account(&id.0, s, &funs, &ctx.0).await?;
        }
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Get Sub Deploy Rel Account Ids
    /// 获取二级部署关联账号
    #[oai(path = "/:id/account", method = "get")]
    async fn find_rel_account_ids(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Vec<String>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamSubDeployServ::find_rel_id_by_sub_deploy_id(&IamRelKind::IamSubDeployAccount, &id.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Exist Sub Deploy Rel Account
    /// 二级部署关联账号是否存在
    #[oai(path = "/:id/account/:account_id/exist", method = "get")]
    async fn exist_rel_account(&self, id: Path<String>, account_id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<bool> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamSubDeployServ::exist_sub_deploy_rels(&IamRelKind::IamSubDeployAccount, &id.0, &account_id.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Add Sub Deploy Rel Auth Account
    /// 添加二级部署关联认证账号
    #[oai(path = "/:id/auth/account/:account_id", method = "put")]
    async fn add_rel_auth_account(&self, id: Path<String>, account_id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSubDeployServ::add_rel_auth_account(&id.0, &account_id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Batch add Sub Deploy Rel Auth Account
    /// 批量添加二级部署关联认证账号
    #[oai(path = "/:id/auth/account/batch/:account_ids", method = "put")]
    async fn batch_add_rel_auth_account(&self, id: Path<String>, account_ids: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        join_all(account_ids.0.split(',').map(|account_id| async { IamSubDeployServ::add_rel_auth_account(&id.0, account_id, &funs, &ctx.0).await }).collect_vec())
            .await
            .into_iter()
            .collect::<Result<Vec<()>, TardisError>>()?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Delete Sub Deploy Rel Auth Account
    /// 删除二级部署关联认证账号
    #[oai(path = "/:id/auth/account/:account_id", method = "delete")]
    async fn delete_rel_auth_account(&self, id: Path<String>, account_id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSubDeployServ::delete_rel_auth_account(&id.0, &account_id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Batch delete Sub Deploy Rel Auth Account
    /// 批量删除二级部署关联认证账号
    #[oai(path = "/:id/auth/account/batch/:account_ids", method = "delete")]
    async fn batch_delete_rel_auth_account(&self, id: Path<String>, account_ids: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let split = account_ids.0.split(',').collect::<Vec<_>>();
        for s in split {
            IamSubDeployServ::delete_rel_auth_account(&id.0, s, &funs, &ctx.0).await?;
        }
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Get Sub Deploy Rel Auth Account Ids
    /// 获取二级部署关联认证账号
    #[oai(path = "/:id/auth/account", method = "get")]
    async fn find_rel_auth_account_ids(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Vec<String>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamSubDeployServ::find_rel_id_by_sub_deploy_id(&IamRelKind::IamSubDeployAuthAccount, &id.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Exist Sub Deploy Rel Auth Account
    /// 二级部署关联认证账号是否存在
    #[oai(path = "/:id/auth/account/:account_id/exist", method = "get")]
    async fn exist_rel_auth_account(&self, id: Path<String>, account_id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<bool> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamSubDeployServ::exist_sub_deploy_rels(&IamRelKind::IamSubDeployAuthAccount, &id.0, &account_id.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Add Sub Deploy Rel Org
    /// 添加二级部署关联组织
    #[oai(path = "/:id/org/:org_id", method = "put")]
    async fn add_rel_org(&self, id: Path<String>, org_id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSubDeployServ::add_rel_org(&id.0, &org_id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Batch add Sub Deploy Rel Org
    /// 批量添加二级部署关联组织
    #[oai(path = "/:id/org/batch/:org_ids", method = "put")]
    async fn batch_add_rel_org(&self, id: Path<String>, org_ids: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        join_all(org_ids.0.split(',').map(|org_id| async { IamSubDeployServ::add_rel_org(&id.0, org_id, &funs, &ctx.0).await }).collect_vec())
            .await
            .into_iter()
            .collect::<Result<Vec<()>, TardisError>>()?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Delete Sub Deploy Rel Org
    /// 删除二级部署关联组织
    #[oai(path = "/:id/org/:org_id", method = "delete")]
    async fn delete_rel_org(&self, id: Path<String>, org_id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSubDeployServ::delete_rel_org(&id.0, &org_id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Batch delete Sub Deploy Rel Org
    /// 批量删除二级部署关联组织
    #[oai(path = "/:id/org/batch/:org_ids", method = "delete")]
    async fn batch_delete_rel_org(&self, id: Path<String>, org_ids: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let split = org_ids.0.split(',').collect::<Vec<_>>();
        for s in split {
            IamSubDeployServ::delete_rel_org(&id.0, s, &funs, &ctx.0).await?;
        }
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Get Sub Deploy Rel Org Ids
    /// 获取二级部署关联组织
    #[oai(path = "/:id/org", method = "get")]
    async fn find_rel_org_ids(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Vec<String>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamSubDeployServ::find_rel_id_by_sub_deploy_id(&IamRelKind::IamSubDeployOrg, &id.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Exist Sub Deploy Rel Org
    /// 二级部署关联组织是否存在
    #[oai(path = "/:id/org/:org_id/exist", method = "get")]
    async fn exist_rel_org(&self, id: Path<String>, org_id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<bool> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamSubDeployServ::exist_sub_deploy_rels(&IamRelKind::IamSubDeployOrg, &id.0, &org_id.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Add Sub Deploy Rel Apps
    /// 添加二级部署关联项目组
    #[oai(path = "/:id/apps/:apps_id", method = "put")]
    async fn add_rel_apps(&self, id: Path<String>, apps_id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSubDeployServ::add_rel_apps(&id.0, &apps_id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Batch add Sub Deploy Rel Apps
    /// 批量添加二级部署关联项目组
    #[oai(path = "/:id/apps/batch/:apps_ids", method = "put")]
    async fn batch_add_rel_apps(&self, id: Path<String>, apps_ids: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        join_all(apps_ids.0.split(',').map(|apps_id| async { IamSubDeployServ::add_rel_apps(&id.0, apps_id, &funs, &ctx.0).await }).collect_vec())
            .await
            .into_iter()
            .collect::<Result<Vec<()>, TardisError>>()?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Delete Sub Deploy Rel Apps
    /// 删除二级部署关联项目组
    #[oai(path = "/:id/apps/:apps_id", method = "delete")]
    async fn delete_rel_apps(&self, id: Path<String>, apps_id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSubDeployServ::delete_rel_apps(&id.0, &apps_id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Batch delete Sub Deploy Rel Apps
    /// 批量删除二级部署关联项目组
    #[oai(path = "/:id/apps/batch/:apps_ids", method = "delete")]
    async fn batch_delete_rel_apps(&self, id: Path<String>, apps_ids: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let split = apps_ids.0.split(',').collect::<Vec<_>>();
        for s in split {
            IamSubDeployServ::delete_rel_apps(&id.0, s, &funs, &ctx.0).await?;
        }
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Get Sub Deploy Rel Apps Ids
    /// 获取二级部署关联项目组
    #[oai(path = "/:id/apps", method = "get")]
    async fn find_rel_apps_ids(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Vec<String>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamSubDeployServ::find_rel_id_by_sub_deploy_id(&IamRelKind::IamSubDeployApps, &id.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Exist Sub Deploy Rel Apps
    /// 二级部署关联项目组是否存在
    #[oai(path = "/:id/apps/:apps_id/exist", method = "get")]
    async fn exist_rel_apps(&self, id: Path<String>, apps_id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<bool> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamSubDeployServ::exist_sub_deploy_rels(&IamRelKind::IamSubDeployApps, &id.0, &apps_id.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Add Sub Deploy Rel Other
    /// 添加二级部署关联其他
    #[oai(path = "/:id/other/:other_id", method = "put")]
    async fn add_rel_other(&self, id: Path<String>, other_id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSubDeployServ::add_rel_other(&id.0, &other_id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Batch add Sub Deploy Rel Other
    /// 批量添加二级部署关联其他
    #[oai(path = "/:id/other/batch/:other_ids", method = "put")]
    async fn batch_add_rel_other(&self, id: Path<String>, other_ids: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        join_all(other_ids.0.split(',').map(|other_id| async { IamSubDeployServ::add_rel_other(&id.0, other_id, &funs, &ctx.0).await }).collect_vec())
            .await
            .into_iter()
            .collect::<Result<Vec<()>, TardisError>>()?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Delete Sub Deploy Rel Other
    /// 删除二级部署关联其他
    #[oai(path = "/:id/other/:other_id", method = "delete")]
    async fn delete_rel_other(&self, id: Path<String>, other_id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSubDeployServ::delete_rel_other(&id.0, &other_id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Batch delete Sub Deploy Rel Other
    /// 批量删除二级部署关联其他
    #[oai(path = "/:id/other/batch/:other_ids", method = "delete")]
    async fn batch_delete_rel_other(&self, id: Path<String>, other_ids: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let split = other_ids.0.split(',').collect::<Vec<_>>();
        for s in split {
            IamSubDeployServ::delete_rel_other(&id.0, s, &funs, &ctx.0).await?;
        }
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Get Sub Deploy Rel Other Ids
    /// 获取二级部署关联其他
    #[oai(path = "/:id/other", method = "get")]
    async fn find_rel_other_ids(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Vec<String>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamSubDeployServ::find_rel_id_by_sub_deploy_id(&IamRelKind::IamSubDeployRel, &id.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Exist Sub Deploy Rel Other
    /// 二级部署关联其他是否存在
    #[oai(path = "/:id/other/:other_id/exist", method = "get")]
    async fn exist_rel_other(&self, id: Path<String>, other_id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<bool> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamSubDeployServ::exist_sub_deploy_rels(&IamRelKind::IamSubDeployRel, &id.0, &other_id.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }
}

/// Tenant Console Sub Deploy Host API
///
/// 租户控制台 二级部署 地址 API
#[poem_openapi::OpenApi(prefix_path = "/ct/sub_deploy/host", tag = "bios_basic::ApiTag::Tenant")]
impl IamCtSubDeployHostApi {
    /// Get Sub Deploy Host By Sub Deploy Host Id
    ///
    /// 根据二级部署地址ID获取二级部署地址
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamSubDeployHostDetailResp> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamSubDeployHostServ::get_rbum(
            &id.0,
            &IamSubDeployHostFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            &funs,
            &ctx.0,
        )
        .await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Sub Deploy Host
    ///
    /// 查找二级部署地址
    #[oai(path = "/", method = "get")]
    async fn paginate(
        &self,
        ids: Query<Option<String>>,
        sub_deploy_id: Query<Option<String>>,
        host: Query<Option<String>>,
        host_type: Query<Option<IamSubDeployHostKind>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<TardisPage<IamSubDeployHostDetailResp>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamSubDeployHostServ::paginate_rbums(
            &IamSubDeployHostFilterReq {
                basic: RbumBasicFilterReq {
                    ids: ids.0.map(|ids| ids.split(',').map(|id| id.to_string()).collect::<Vec<String>>()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                sub_deploy_id: sub_deploy_id.0,
                host: host.0,
                host_type: host_type.0,
                ..Default::default()
            },
            page_number.0,
            page_size.0,
            desc_by_create.0,
            desc_by_update.0,
            &funs,
            &ctx.0,
        )
        .await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Add Sub Deploy Host
    ///
    /// 添加二级部署地址
    #[oai(path = "/", method = "post")]
    async fn add(&self, mut add_req: Json<IamSubDeployHostAddReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamSubDeployHostServ::add_rbum(&mut add_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Modify Sub Deploy Host By Sub Deploy Host Id
    ///
    /// 修改二级部署地址
    #[oai(path = "/:id", method = "put")]
    async fn modify(&self, id: Path<String>, mut modify_req: Json<IamSubDeployHostModifyReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSubDeployHostServ::modify_rbum(&id.0, &mut modify_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Delete Sub Deploy Host By Sub Deploy Host Id
    ///
    /// 删除二级部署地址
    #[oai(path = "/:id", method = "delete")]
    async fn delete(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Option<String>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSubDeployHostServ::delete_rbum(&id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        if let Some(task_id) = TaskProcessor::get_task_id_with_ctx(&ctx.0).await? {
            TardisResp::accepted(Some(task_id))
        } else {
            TardisResp::ok(None)
        }
    }
}

/// Tenant Console Sub Deploy License API
///
/// 租户控制台 二级部署 许可证 API
#[poem_openapi::OpenApi(prefix_path = "/ct/sub_deploy/license", tag = "bios_basic::ApiTag::Tenant")]
impl IamCtSubDeployLicenseApi {
    /// Get Sub Deploy License By Sub Deploy Id
    ///
    /// 根据二级部署ID获取二级部署许可证
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamSubDeployLicenseDetailResp> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamSubDeployLicenseServ::get_rbum(
            &id.0,
            &IamSubDeployLicenseFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            &funs,
            &ctx.0,
        )
        .await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Sub Deploy License
    ///
    /// 查找二级部署许可证
    #[oai(path = "/", method = "get")]
    async fn paginate(
        &self,
        ids: Query<Option<String>>,
        sub_deploy_id: Query<String>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<TardisPage<IamSubDeployLicenseDetailResp>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamSubDeployLicenseServ::paginate_rbums(
            &IamSubDeployLicenseFilterReq {
                basic: RbumBasicFilterReq {
                    ids: ids.0.map(|ids| ids.split(',').map(|id| id.to_string()).collect::<Vec<String>>()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                sub_deploy_id: Some(sub_deploy_id.0),
                ..Default::default()
            },
            page_number.0,
            page_size.0,
            desc_by_create.0,
            desc_by_update.0,
            &funs,
            &ctx.0,
        )
        .await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Show License
    ///
    /// 显示许可证
    #[oai(path = "/show/license/:id", method = "get")]
    async fn show_license(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamSubDeployLicenseServ::show_license(
            &id.0,
            &IamSubDeployLicenseFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            &funs,
            &ctx.0,
        )
        .await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Download License
    ///
    /// 下载许可证
    #[oai(path = "/download/license/:id", method = "get")]
    async fn download_license(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> Result<Attachment<Vec<u8>>, poem::Error> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamSubDeployLicenseServ::generate_license(&id.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        let attachment = Attachment::new(result.into_bytes()).filename("license.cert");
        Ok(attachment.into())
    }

    /// Add Sub Deploy License
    ///
    /// 添加二级部署许可证
    #[oai(path = "/", method = "post")]
    async fn add(&self, mut add_req: Json<IamSubDeployLicenseAddReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamSubDeployLicenseServ::add_rbum(&mut add_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Modify Sub Deploy License By Sub Deploy Id
    ///
    /// 修改二级部署许可证
    #[oai(path = "/:id", method = "put")]
    async fn modify(&self, id: Path<String>, mut modify_req: Json<IamSubDeployLicenseModifyReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSubDeployLicenseServ::modify_rbum(&id.0, &mut modify_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Delete Sub Deploy License By Sub Deploy Id
    ///
    /// 删除二级部署许可证
    #[oai(path = "/:id", method = "delete")]
    async fn delete(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Option<String>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSubDeployLicenseServ::delete_rbum(&id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        if let Some(task_id) = TaskProcessor::get_task_id_with_ctx(&ctx.0).await? {
            TardisResp::accepted(Some(task_id))
        } else {
            TardisResp::ok(None)
        }
    }
}
