use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use crate::dto::flow_inst_dto::{
    FlowInstAbortReq, FlowInstBindReq, FlowInstBindResp, FlowInstDetailResp, FlowInstFindNextTransitionResp, FlowInstFindNextTransitionsReq, FlowInstFindStateAndTransitionsReq,
    FlowInstFindStateAndTransitionsResp, FlowInstModifyAssignedReq, FlowInstStartReq, FlowInstSummaryResp, FlowInstTransferReq, FlowInstTransferResp,
};
use crate::flow_constants;
use crate::serv::flow_inst_serv::FlowInstServ;
#[derive(Clone)]
pub struct FlowCcInstApi;

/// Flow instance process API
#[poem_openapi::OpenApi(prefix_path = "/cc/inst")]
impl FlowCcInstApi {
    /// Start Instance / 启动实例
    #[oai(path = "/", method = "post")]
    async fn start(&self, add_req: Json<FlowInstStartReq>, ctx: TardisContextExtractor) -> TardisApiResult<String> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        let result = FlowInstServ::start(&add_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(result)
    }

    /// Batch Bind Instance / 批量绑定实例 （初始化）
    #[oai(path = "/batch_bind", method = "post")]
    async fn batch_bind(&self, add_req: Json<FlowInstBindReq>, ctx: TardisContextExtractor) -> TardisApiResult<Vec<FlowInstBindResp>> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        let result = FlowInstServ::batch_bind(&add_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(result)
    }

    /// Abort Instance / 中止实例
    #[oai(path = "/:flow_inst_id", method = "delete")]
    async fn abort(&self, flow_inst_id: Path<String>, abort_req: Json<FlowInstAbortReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        FlowInstServ::abort(&flow_inst_id.0, &abort_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Get Instance By Instance Id / 获取实例信息
    #[oai(path = "/:flow_inst_id", method = "get")]
    async fn get(&self, flow_inst_id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<FlowInstDetailResp> {
        let funs = flow_constants::get_tardis_inst();
        let result = FlowInstServ::get(&flow_inst_id.0, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }

    /// Find Instances / 获取实例列表
    #[oai(path = "/", method = "get")]
    async fn paginate(
        &self,
        flow_model_id: Query<Option<String>>,
        tag: Query<Option<String>>,
        finish: Query<Option<bool>>,
        with_sub: Query<Option<bool>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<FlowInstSummaryResp>> {
        let funs = flow_constants::get_tardis_inst();
        let result = FlowInstServ::paginate(flow_model_id.0, tag.0, finish.0, with_sub.0, page_number.0, page_size.0, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }

    /// Find Next Transitions / 获取下一个流转状态列表
    #[oai(path = "/:flow_inst_id/transition/next", method = "put")]
    async fn find_next_transitions(
        &self,
        flow_inst_id: Path<String>,
        next_req: Json<FlowInstFindNextTransitionsReq>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<Vec<FlowInstFindNextTransitionResp>> {
        let funs = flow_constants::get_tardis_inst();
        let result = FlowInstServ::find_next_transitions(&flow_inst_id.0, &next_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }

    /// Find the state and transfer information of the specified model in batch / 批量获取指定模型的状态及流转信息
    #[oai(path = "/batch/state_transitions", method = "put")]
    async fn find_state_and_next_transitions(
        &self,
        find_req: Json<Vec<FlowInstFindStateAndTransitionsReq>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<Vec<FlowInstFindStateAndTransitionsResp>> {
        let funs = flow_constants::get_tardis_inst();
        let result = FlowInstServ::find_state_and_next_transitions(&find_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }

    /// Transfer State By State Id / 流转
    #[oai(path = "/:flow_inst_id/transition/transfer", method = "put")]
    async fn transfer(&self, flow_inst_id: Path<String>, transfer_req: Json<FlowInstTransferReq>, ctx: TardisContextExtractor) -> TardisApiResult<FlowInstTransferResp> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        FlowInstServ::check_transfer_vars(&flow_inst_id.0, &transfer_req.0, &funs, &ctx.0).await?;
        let result = FlowInstServ::transfer(&flow_inst_id.0, &transfer_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(result)
    }

    /// Modify Assigned / 同步执行人信息
    #[oai(path = "/:flow_inst_id/transition/modify_assigned", method = "post")]
    async fn modify_assigned(&self, flow_inst_id: Path<String>, modify_req: Json<FlowInstModifyAssignedReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        FlowInstServ::modify_assigned(&flow_inst_id.0, &modify_req.0.current_assigned, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }
}
