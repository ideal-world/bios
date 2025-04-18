use bios_basic::helper::request_helper::try_set_real_ip_from_req_to_ctx;
use bios_basic::process::task_processor::TaskProcessor;
use bios_basic::rbum::helper::rbum_scope_helper::check_without_owner_and_unsafe_fill_ctx;

use tardis::chrono::{DateTime, Utc};
use tardis::web::poem::web::{Json, Query};
use tardis::web::poem_openapi;

use tardis::web::poem_openapi::param::Path;
use tardis::web::{
    context_extractor::TardisContextExtractor,
    poem::Request,
    web_resp::{TardisApiResult, TardisResp},
};

use crate::basic::dto::iam_sub_deploy_dto::{IamSubDeployOneExportAggResp, IamSubDeployOneImportReq, IamSubDeployTowExportAggResp, IamSubDeployTowImportReq};
use crate::basic::serv::iam_sub_deploy_serv::IamSubDeployServ;
use crate::iam_constants;

#[derive(Clone, Default)]
pub struct IamCiSubDeployApi;

/// # Interface Console Sub Deploy API
///
/// 接口控制台二级部署API
#[poem_openapi::OpenApi(prefix_path = "/ci/sub_deploy", tag = "bios_basic::ApiTag::Tenant")]
impl IamCiSubDeployApi {
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
        start_time: Query<DateTime<Utc>>,
        end_time: Query<DateTime<Utc>>,
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<IamSubDeployTowExportAggResp> {
        let funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let result = IamSubDeployServ::sub_deploy_export(start_time.0, end_time.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Sub Deploy Import Data
    ///
    /// 二级部署导入数据,从一级部署导出数据
    #[oai(path = "/sub/import", method = "put")]
    async fn sub_deploy_import(&self, import_req: Json<IamSubDeployTowImportReq>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Option<String>> {
        let funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        IamSubDeployServ::sub_deploy_import(import_req.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        if let Some(task_id) = TaskProcessor::get_task_id_with_ctx(&ctx.0).await? {
            TardisResp::accepted(Some(task_id))
        } else {
            TardisResp::ok(None)
        }
    }
}
