use std::collections::HashMap;

use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use crate::dto::flow_external_dto::FlowExternalCallbackOp;
use crate::dto::flow_inst_dto::{
    FlowInstAbortReq, FlowInstDetailResp, FlowInstFindNextTransitionResp, FlowInstFindNextTransitionsReq, FlowInstFindStateAndTransitionsReq, FlowInstFindStateAndTransitionsResp,
    FlowInstModifyAssignedReq, FlowInstModifyCurrentVarsReq, FlowInstStartReq, FlowInstSummaryResp, FlowInstTransferReq, FlowInstTransferResp,
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
    async fn start(&self, add_req: Json<FlowInstStartReq>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<String> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        let result = FlowInstServ::start(&add_req.0, None, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(result)
    }

    /// Abort Instance / 中止实例
    #[oai(path = "/:flow_inst_id", method = "put")]
    async fn abort(&self, flow_inst_id: Path<String>, abort_req: Json<FlowInstAbortReq>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<Void> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        FlowInstServ::abort(&flow_inst_id.0, &abort_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Get Instance By Instance Id / 获取实例信息
    #[oai(path = "/:flow_inst_id", method = "get")]
    async fn get(&self, flow_inst_id: Path<String>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<FlowInstDetailResp> {
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
        _request: &Request,
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
        _request: &Request,
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
        _request: &Request,
    ) -> TardisApiResult<Vec<FlowInstFindStateAndTransitionsResp>> {
        let funs = flow_constants::get_tardis_inst();
        let result = FlowInstServ::find_state_and_next_transitions(&find_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }

    /// Transfer State By State Id / 流转
    #[oai(path = "/:flow_inst_id/transition/transfer", method = "put")]
    async fn transfer(
        &self,
        flow_inst_id: Path<String>,
        transfer_req: Json<FlowInstTransferReq>,
        ctx: TardisContextExtractor,
        _request: &Request,
    ) -> TardisApiResult<FlowInstTransferResp> {
        let mut funs = flow_constants::get_tardis_inst();
        let mut transfer = transfer_req.0;
        FlowInstServ::check_transfer_vars(&flow_inst_id.0, &mut transfer, &funs, &ctx.0).await?;
        funs.begin().await?;
        let result = FlowInstServ::transfer(&flow_inst_id.0, &transfer, false, FlowExternalCallbackOp::Default, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(result)
    }

    /// Batch transfer State By State Id / 批量流转
    #[oai(path = "/batch/:flow_inst_ids/transition/transfer", method = "put")]
    async fn batch_transfer(
        &self,
        flow_inst_ids: Path<String>,
        transfer_req: Json<FlowInstTransferReq>,
        ctx: TardisContextExtractor,
        _request: &Request,
    ) -> TardisApiResult<Vec<FlowInstTransferResp>> {
        let mut funs = flow_constants::get_tardis_inst();
        let mut result = vec![];
        let flow_inst_ids: Vec<_> = flow_inst_ids.split(',').collect();
        let raw_transfer_req = transfer_req.0;
        let mut flow_inst_id_transfer_map = HashMap::new();
        funs.begin().await?;
        for flow_inst_id in &flow_inst_ids {
            let mut transfer_req = raw_transfer_req.clone();
            FlowInstServ::check_transfer_vars(flow_inst_id, &mut transfer_req, &funs, &ctx.0).await?;
            flow_inst_id_transfer_map.insert(flow_inst_id, transfer_req);
        }
        for (flow_inst_id, transfer_req) in flow_inst_id_transfer_map {
            result.push(FlowInstServ::transfer(flow_inst_id, &transfer_req, false, FlowExternalCallbackOp::Default, &funs, &ctx.0).await?);
        }
        funs.commit().await?;
        TardisResp::ok(result)
    }

    /// Modify Assigned / 同步执行人信息
    #[oai(path = "/:flow_inst_id/transition/modify_assigned", method = "post")]
    async fn modify_assigned(
        &self,
        flow_inst_id: Path<String>,
        modify_req: Json<FlowInstModifyAssignedReq>,
        ctx: TardisContextExtractor,
        _request: &Request,
    ) -> TardisApiResult<Void> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        FlowInstServ::modify_assigned(&flow_inst_id.0, &modify_req.0.current_assigned, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Modify list of variables / 同步当前变量列表
    #[oai(path = "/:flow_inst_id/modify_current_vars", method = "patch")]
    async fn modify_current_vars(
        &self,
        flow_inst_id: Path<String>,
        modify_req: Json<FlowInstModifyCurrentVarsReq>,
        ctx: TardisContextExtractor,
        _request: &Request,
    ) -> TardisApiResult<Void> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        FlowInstServ::modify_current_vars(&flow_inst_id.0, &modify_req.0.vars, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }
}
