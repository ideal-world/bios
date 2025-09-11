use bios_basic::helper::request_helper::try_set_real_ip_from_req_to_ctx;
use bios_basic::process::task_processor::TaskProcessor;
use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::helper::rbum_scope_helper::check_without_owner_and_unsafe_fill_ctx;

use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use itertools::Itertools;
use tardis::basic::error::TardisError;
use tardis::futures::future::join_all;
use tardis::web::poem::web::Json;
use tardis::web::poem_openapi;

use tardis::web::poem_openapi::param::Path;
use tardis::web::web_resp::Void;
use tardis::web::{
    context_extractor::TardisContextExtractor,
    poem::Request,
    web_resp::{TardisApiResult, TardisResp},
};

use crate::basic::dto::iam_filer_dto::IamSubDeployFilterReq;
use crate::basic::dto::iam_sub_deploy_dto::{
    IamSubDeployDetailResp, IamSubDeployOneExportAggResp, IamSubDeployOneImportReq, IamSubDeployTowExportAggResp, IamSubDeployTowImportReq,
};
use crate::basic::serv::iam_sub_deploy_serv::IamSubDeployServ;
use crate::iam_constants;

#[derive(Clone, Default)]
pub struct IamCiSubDeployApi;

/// # Interface Console Sub Deploy API
///
/// 接口控制台二级部署API
#[poem_openapi::OpenApi(prefix_path = "/ci/sub_deploy", tag = "bios_basic::ApiTag::Tenant")]
impl IamCiSubDeployApi {
    /// get sub deploy
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamSubDeployDetailResp> {
        let funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let result = IamSubDeployServ::get_item(
            &id.0,
            &IamSubDeployFilterReq {
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

    /// One Deploy Export Data
    ///
    /// 一级部署导出数据,提供给二级部署导入数据
    #[oai(path = "/one/export/:id", method = "get")]
    async fn one_deploy_export(&self, id: Path<String>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamSubDeployOneExportAggResp> {
        let funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let result = IamSubDeployServ::one_deploy_export(&id.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// One Deploy Import Data
    ///
    /// 一级部署导入数据,从二级部署导出数据
    #[oai(path = "/one/import", method = "put")]
    async fn one_deploy_import(&self, import_req: Json<IamSubDeployOneImportReq>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Option<String>> {
        let funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        IamSubDeployServ::one_deploy_import(import_req.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        if let Some(task_id) = TaskProcessor::get_task_id_with_ctx(&ctx.0).await? {
            TardisResp::accepted(Some(task_id))
        } else {
            TardisResp::ok(None)
        }
    }

    /// Sub Deploy Export Data
    ///
    /// 二级部署导出数据,提供给一级部署导入数据
    #[oai(path = "/sub/export", method = "get")]
    async fn sub_deploy_export(
        &self,
        // start_time: Query<Option<DateTime<Utc>>>,
        // end_time: Query<Option<DateTime<Utc>>>,
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<IamSubDeployTowExportAggResp> {
        let funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let result = IamSubDeployServ::sub_deploy_export(None, None, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Sub Deploy Import Data
    ///
    /// 二级部署导入数据,从一级部署导出数据
    #[oai(path = "/sub/import", method = "put")]
    async fn sub_deploy_import(&self, import_req: Json<IamSubDeployTowImportReq>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Option<String>> {
        let mut funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        funs.begin().await?;
        IamSubDeployServ::sub_deploy_import(import_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        if let Some(task_id) = TaskProcessor::get_task_id_with_ctx(&ctx.0).await? {
            TardisResp::accepted(Some(task_id))
        } else {
            TardisResp::ok(None)
        }
    }

    /// Add Sub Deploy Rel App
    /// 添加二级部署关联项目组
    #[oai(path = "/:id/app/:app_id", method = "put")]
    async fn add_rel_app(&self, id: Path<String>, app_id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSubDeployServ::add_rel_app(&id.0, &app_id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Batch add Sub Deploy Rel App
    /// 批量添加二级部署关联项目组
    #[oai(path = "/:id/app/batch/:app_ids", method = "put")]
    async fn batch_add_rel_app(&self, id: Path<String>, app_ids: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        join_all(app_ids.0.split(',').map(|app_id| async { IamSubDeployServ::add_rel_app(&id.0, app_id, &funs, &ctx.0).await }).collect_vec())
            .await
            .into_iter()
            .collect::<Result<Vec<()>, TardisError>>()?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Delete Sub Deploy Rel App
    /// 删除二级部署关联项目组
    #[oai(path = "/:id/app/:app_id", method = "delete")]
    async fn delete_rel_app(&self, id: Path<String>, app_id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSubDeployServ::delete_rel_app(&id.0, &app_id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Batch delete Sub Deploy Rel App
    /// 批量删除二级部署关联项目组
    #[oai(path = "/:id/app/batch/:app_ids", method = "delete")]
    async fn batch_delete_rel_app(&self, id: Path<String>, app_ids: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let split = app_ids.0.split(',').collect::<Vec<_>>();
        for s in split {
            IamSubDeployServ::delete_rel_app(&id.0, s, &funs, &ctx.0).await?;
        }
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }
}
