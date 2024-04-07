use std::collections::HashMap;

use bios_basic::helper::bios_ctx_helper::unsafe_fill_ctx;
use tardis::log::debug;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::web::Path;
use tardis::web::poem::Request;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::poem_openapi::{self, param::Query};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};
use tardis::{log, tokio};

use crate::dto::flow_external_dto::FlowExternalCallbackOp;
use crate::dto::flow_inst_dto::{
    FlowInstAbortReq, FlowInstBatchBindReq, FlowInstBatchBindResp, FlowInstBindReq, FlowInstDetailResp, FlowInstFindStateAndTransitionsReq, FlowInstFindStateAndTransitionsResp,
    FlowInstModifyAssignedReq, FlowInstModifyCurrentVarsReq, FlowInstStartReq, FlowInstTransferReq, FlowInstTransferResp,
};
use crate::flow_constants;
use crate::serv::flow_inst_serv::FlowInstServ;
#[derive(Clone)]
pub struct FlowCiInstApi;

/// Flow Config process API
#[poem_openapi::OpenApi(prefix_path = "/ci/inst")]
impl FlowCiInstApi {
    /// Start Instance / 启动实例
    #[oai(path = "/", method = "post")]
    async fn start(&self, add_req: Json<FlowInstStartReq>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        let mut funs = flow_constants::get_tardis_inst();
        unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        funs.begin().await?;
        let result = FlowInstServ::start(&add_req.0, None, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(result)
    }

    /// Get Instance By Instance Id / 获取实例信息
    #[oai(path = "/:flow_inst_id", method = "get")]
    async fn get(&self, flow_inst_id: Path<String>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<FlowInstDetailResp> {
        let funs = flow_constants::get_tardis_inst();
        unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        let result = FlowInstServ::get(&flow_inst_id.0, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }

    /// Find the state and transfer information of the specified model in batch / 批量获取指定模型的状态及流转信息
    #[oai(path = "/batch/state_transitions", method = "put")]
    async fn find_state_and_next_transitions(
        &self,
        find_req: Json<Vec<FlowInstFindStateAndTransitionsReq>>,
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<FlowInstFindStateAndTransitionsResp>> {
        let funs = flow_constants::get_tardis_inst();
        unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        let result = FlowInstServ::find_state_and_next_transitions(&find_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }

    /// Abort Instance / 中止实例
    #[oai(path = "/:flow_inst_id", method = "put")]
    async fn abort(&self, flow_inst_id: Path<String>, abort_req: Json<FlowInstAbortReq>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let mut funs = flow_constants::get_tardis_inst();
        unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        funs.begin().await?;
        FlowInstServ::abort(&flow_inst_id.0, &abort_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Transfer State By State Id / 流转
    #[oai(path = "/:flow_inst_id/transition/transfer", method = "put")]
    async fn transfer(
        &self,
        flow_inst_id: Path<String>,
        transfer_req: Json<FlowInstTransferReq>,
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<FlowInstTransferResp> {
        let mut funs = flow_constants::get_tardis_inst();
        unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
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
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<FlowInstTransferResp>> {
        let mut funs = flow_constants::get_tardis_inst();
        unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        debug!("batch transfer request: {:?}", request);
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
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Void> {
        let mut funs = flow_constants::get_tardis_inst();
        unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
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
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Void> {
        let mut funs = flow_constants::get_tardis_inst();
        unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        funs.begin().await?;
        FlowInstServ::modify_current_vars(&flow_inst_id.0, &modify_req.0.vars, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Bind Single Instance / 绑定单个实例
    #[oai(path = "/bind", method = "post")]
    async fn bind(&self, add_req: Json<FlowInstBindReq>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        let mut funs = flow_constants::get_tardis_inst();
        unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        let inst_id = FlowInstServ::get_inst_ids_by_rel_business_obj_id(vec![add_req.0.rel_business_obj_id.clone()], &funs, &ctx.0).await?.pop();
        let result = if let Some(inst_id) = inst_id {
            inst_id
        } else {
            funs.begin().await?;
            let inst_id = FlowInstServ::start(
                &FlowInstStartReq {
                    rel_business_obj_id: add_req.0.rel_business_obj_id.clone(),
                    tag: add_req.0.tag.clone(),
                    create_vars: add_req.0.create_vars.clone(),
                },
                add_req.0.current_state_name.clone(),
                &funs,
                &ctx.0,
            )
            .await?;
            funs.commit().await?;
            inst_id
        };

        TardisResp::ok(result)
    }

    /// Batch Bind Instance / 批量绑定实例 （初始化）
    #[oai(path = "/batch_bind", method = "post")]
    async fn batch_bind(&self, add_req: Json<FlowInstBatchBindReq>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Vec<FlowInstBatchBindResp>> {
        let mut funs = flow_constants::get_tardis_inst();
        unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        funs.begin().await?;
        let result = FlowInstServ::batch_bind(&add_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(result)
    }

    /// Get list of instance id by rel_business_obj_id / 通过业务ID获取实例信息
    #[oai(path = "/find_detail_by_obj_ids", method = "get")]
    async fn find_detail_by_obj_ids(&self, obj_ids: Query<String>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Vec<FlowInstDetailResp>> {
        let funs = flow_constants::get_tardis_inst();
        unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        let rel_business_obj_ids: Vec<_> = obj_ids.0.split(',').map(|id| id.to_string()).collect();
        let inst_ids = FlowInstServ::get_inst_ids_by_rel_business_obj_id(rel_business_obj_ids, &funs, &ctx.0).await?;
        let mut result = vec![];
        for inst_id in inst_ids {
            if let Ok(inst_detail) = FlowInstServ::get(&inst_id, &funs, &ctx.0).await {
                result.push(inst_detail);
            }
        }
        TardisResp::ok(result)
    }

    /// trigger instance front action / 触发前置动作
    #[oai(path = "/trigger_front_action", method = "get")]
    async fn trigger_front_action(&self) -> TardisApiResult<Void> {
        let mut funs = flow_constants::get_tardis_inst();
        tokio::spawn(async move {
            funs.begin().await.unwrap();
            match FlowInstServ::trigger_front_action(&funs).await {
                Ok(_) => {
                    log::trace!("[Flow.Inst] add log success")
                }
                Err(e) => {
                    log::warn!("[Flow.Inst] failed to add log:{e}")
                }
            }
            funs.commit().await.unwrap();
        });

        TardisResp::ok(Void {})
    }
}
