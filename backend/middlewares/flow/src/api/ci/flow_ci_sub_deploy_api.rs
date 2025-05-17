use bios_basic::helper::request_helper::try_set_real_ip_from_req_to_ctx;
use bios_basic::rbum::helper::rbum_scope_helper::check_without_owner_and_unsafe_fill_ctx;

use tardis::web::poem::web::Json;
use tardis::web::poem_openapi;

use tardis::web::poem_openapi::param::Path;
use tardis::web::web_resp::Void;
use tardis::web::{
    context_extractor::TardisContextExtractor,
    poem::Request,
    web_resp::{TardisApiResult, TardisResp},
};

use crate::dto::flow_sub_deploy_dto::{FlowSubDeployOneExportAggResp, FlowSubDeployOneImportReq, FlowSubDeployTowExportAggResp, FlowSubDeployTowImportReq};
use crate::flow_constants;
use crate::serv::flow_sub_deploy_serv::FlowSubDeployServ;

#[derive(Clone, Default)]
pub struct FlowCiSubDeployApi;

/// # Interface Console Sub Deploy API
///
/// 接口控制台二级部署API
#[poem_openapi::OpenApi(prefix_path = "/ci/sub_deploy")]
impl FlowCiSubDeployApi {
    /// One Deploy Export Data
    ///
    /// 一级部署导出数据,提供给二级部署导入数据
    #[oai(path = "/one/export/:id", method = "get")]
    async fn one_deploy_export(&self, id: Path<String>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<FlowSubDeployOneExportAggResp> {
        let funs = flow_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let result = FlowSubDeployServ::one_deploy_export(&id.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// One Deploy Import Data
    ///
    /// 一级部署导入数据,从二级部署导出数据
    #[oai(path = "/one/import", method = "put")]
    async fn one_deploy_import(&self, import_req: Json<FlowSubDeployOneImportReq>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let funs = flow_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        FlowSubDeployServ::one_deploy_import(import_req.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void)
    }

    /// Sub Deploy Export Data
    ///
    /// 二级部署导出数据,提供给一级部署导入数据
    #[oai(path = "/sub/export", method = "get")]
    async fn sub_deploy_export(
        &self,
        // start_time: Query<String>,
        // end_time: Query<String>,
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<FlowSubDeployTowExportAggResp> {
        let funs = flow_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let result = FlowSubDeployServ::sub_deploy_export(&funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Sub Deploy Import Data
    ///
    /// 二级部署导入数据,从一级部署导出数据
    #[oai(path = "/sub/import", method = "put")]
    async fn sub_deploy_import(&self, import_req: Json<FlowSubDeployTowImportReq>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let mut funs = flow_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        funs.begin().await?;
        FlowSubDeployServ::sub_deploy_import(import_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void)
    }
}
