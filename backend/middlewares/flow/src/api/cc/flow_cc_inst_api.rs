use std::collections::HashMap;

use serde_json::Value;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use crate::dto::flow_external_dto::FlowExternalCallbackOp;
use crate::dto::flow_inst_dto::{
    FlowInstAbortReq, FlowInstBatchCheckAuthReq, FlowInstCommentReq, FlowInstDetailResp, FlowInstFilterReq, FlowInstFindNextTransitionResp, FlowInstFindNextTransitionsReq,
    FlowInstFindRelModelReq, FlowInstFindStateAndTransitionsReq, FlowInstFindStateAndTransitionsResp, FlowInstModifyAssignedReq, FlowInstModifyCurrentVarsReq, FlowInstOperateReq,
    FlowInstStartReq, FlowInstSummaryResp, FlowInstTransferReq, FlowInstTransferResp,
};
use crate::dto::flow_model_dto::FlowModelDetailResp;
use crate::dto::flow_state_dto::FlowSysStateKind;
use crate::flow_constants;
use crate::helper::{loop_check_helper, task_handler_helper};
use crate::serv::flow_inst_serv::FlowInstServ;
#[derive(Clone)]
pub struct FlowCcInstApi;

/// Flow instance process API
#[poem_openapi::OpenApi(prefix_path = "/cc/inst")]
impl FlowCcInstApi {
    /// Start Instance(Return Instance ID)
    ///
    /// 启动实例(返回实例ID)
    #[oai(path = "/", method = "post")]
    async fn start(&self, add_req: Json<FlowInstStartReq>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<String> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        let result = FlowInstServ::start(&add_req.0, None, &funs, &ctx.0).await?;
        funs.commit().await?;
        task_handler_helper::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Start Instance(Return Instance ID)
    ///
    /// 尝试启动实例(返回实例ID)
    #[oai(path = "/try", method = "post")]
    async fn try_start(&self, add_req: Json<FlowInstStartReq>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<String> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        let result = FlowInstServ::try_start(&add_req.0, None, &funs, &ctx.0).await?;
        funs.commit().await?;
        task_handler_helper::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Start Instance(Return Instance ID)
    ///
    /// 批量尝试启动实例(返回实例ID)
    #[oai(path = "/batch/try", method = "post")]
    async fn batch_try_start(&self, add_req: Json<Vec<FlowInstStartReq>>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<HashMap<String, String>> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        let mut result = HashMap::new();
        for req in add_req.0 {
            let inst_id = FlowInstServ::try_start(&req, None, &funs, &ctx.0).await?;
            result.insert(req.rel_business_obj_id.clone(), inst_id);
        }
        funs.commit().await?;
        task_handler_helper::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Start Instance(Return Instance)
    ///
    /// 启动实例(返回实例)
    #[oai(path = "/start_and_get", method = "post")]
    async fn start_and_get(&self, add_req: Json<FlowInstStartReq>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<FlowInstDetailResp> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        let inst_id = FlowInstServ::start(&add_req.0, None, &funs, &ctx.0).await?;
        let result = FlowInstServ::get(&inst_id, &funs, &ctx.0).await?;
        funs.commit().await?;
        task_handler_helper::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Start Instance(Return Instance ID)
    ///
    /// 批量启动实例(返回实例ID)
    #[oai(path = "/batch_start", method = "post")]
    async fn batch_start(&self, add_batch_req: Json<Vec<FlowInstStartReq>>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<Void> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        for add_req in &add_batch_req.0 {
            FlowInstServ::start(add_req, None, &funs, &ctx.0).await?;
        }
        funs.commit().await?;
        task_handler_helper::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Abort Instance
    ///
    /// 终止实例
    #[oai(path = "/:flow_inst_id", method = "put")]
    async fn abort(&self, flow_inst_id: Path<String>, abort_req: Json<FlowInstAbortReq>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<Void> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;

        let flow_inst_detail = FlowInstServ::get(&flow_inst_id.0, &funs, &ctx.0).await?;
        // 若为状态流或不携带子审批流的实例，直接终止
        if flow_inst_detail.main || flow_inst_detail.artifacts.clone().unwrap_or_default().rel_child_objs.unwrap_or_default().is_empty() {
            FlowInstServ::abort(&flow_inst_id.0, &abort_req.0, &funs, &ctx.0).await?;
        } else {
            // 若为携带子审批流的数据，则直接触发状态流流转至结束
            let main_inst = FlowInstServ::find_detail_items(
                &FlowInstFilterReq {
                    rel_business_obj_ids: Some(vec![flow_inst_detail.rel_business_obj_id.clone()]),
                    main: Some(true),
                    ..Default::default()
                },
                &funs,
                &ctx.0,
            )
            .await?
            .pop()
            .ok_or_else(|| {
                funs.err().not_found(
                    "flow_inst",
                    "abort",
                    &format!("flow inst is not found by {}", flow_inst_detail.rel_business_obj_id),
                    "404-flow-inst-not-found",
                )
            })?;
            if let Some(next_finish_tran) = FlowInstServ::find_next_transitions(&main_inst, &FlowInstFindNextTransitionsReq { vars: None }, &funs, &ctx.0)
                .await?
                .into_iter()
                .find(|next_tran| next_tran.next_flow_state_sys_state == FlowSysStateKind::Finish)
            {
                FlowInstServ::transfer(
                    &main_inst,
                    &FlowInstTransferReq {
                        flow_transition_id: next_finish_tran.next_flow_transition_id,
                        message: None,
                        vars: None,
                    },
                    false,
                    FlowExternalCallbackOp::Auto,
                    loop_check_helper::InstancesTransition::default(),
                    &ctx.0,
                    &funs,
                )
                .await?;
            } else {
                FlowInstServ::abort(&flow_inst_id.0, &abort_req.0, &funs, &ctx.0).await?;
            }
        }
        funs.commit().await?;
        task_handler_helper::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Get Instance By Instance Id
    ///
    /// 获取实例信息
    #[oai(path = "/:flow_inst_id", method = "get")]
    async fn get(&self, flow_inst_id: Path<String>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<FlowInstDetailResp> {
        let funs = flow_constants::get_tardis_inst();
        let result = FlowInstServ::get(&flow_inst_id.0, &funs, &ctx.0).await?;
        task_handler_helper::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Instances
    ///
    /// 获取实例列表
    #[oai(path = "/", method = "get")]
    #[allow(clippy::too_many_arguments)]
    async fn paginate(
        &self,
        flow_model_id: Query<Option<String>>,
        rel_business_obj_id: Query<Option<String>>,
        tag: Query<Option<String>>,
        finish: Query<Option<bool>>,
        finish_abort: Query<Option<bool>>,
        main: Query<Option<bool>>,
        current_state_id: Query<Option<String>>,
        current_state_sys_kind: Query<Option<FlowSysStateKind>>,
        with_sub: Query<Option<bool>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        ctx: TardisContextExtractor,
        _request: &Request,
    ) -> TardisApiResult<TardisPage<FlowInstSummaryResp>> {
        let funs = flow_constants::get_tardis_inst();
        let result = FlowInstServ::paginate(
            &FlowInstFilterReq {
                flow_model_id: flow_model_id.0,
                tags: tag.0.map(|t| vec![t]),
                finish: finish.0,
                finish_abort: finish_abort.0,
                main: main.0,
                current_state_id: current_state_id.0,
                current_state_sys_kind: current_state_sys_kind.0,
                rel_business_obj_ids: rel_business_obj_id.0.map(|id| vec![id]),
                with_sub: with_sub.0,
                ..Default::default()
            },
            page_number.0,
            page_size.0,
            &funs,
            &ctx.0,
        )
        .await?;
        task_handler_helper::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Instances
    ///
    /// 获取实例列表
    #[oai(path = "/details", method = "get")]
    #[allow(clippy::too_many_arguments)]
    async fn paginate_detail_items(
        &self,
        flow_model_id: Query<Option<String>>,
        rel_business_obj_id: Query<Option<String>>,
        tag: Query<Option<String>>,
        finish: Query<Option<bool>>,
        main: Query<Option<bool>>,
        current_state_id: Query<Option<String>>,
        rel_inst_id: Query<Option<String>>,
        with_sub: Query<Option<bool>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
        _request: &Request,
    ) -> TardisApiResult<TardisPage<FlowInstDetailResp>> {
        let funs = flow_constants::get_tardis_inst();
        let result = FlowInstServ::paginate_detail_items(
            &FlowInstFilterReq {
                flow_version_id: flow_model_id.0,
                tags: tag.0.map(|tag| vec![tag]),
                finish: finish.0,
                main: main.0,
                current_state_id: current_state_id.0,
                rel_business_obj_ids: rel_business_obj_id.0.map(|id| vec![id]),
                rel_inst_ids: rel_inst_id.0.map(|id| vec![id]),
                with_sub: with_sub.0,
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
        task_handler_helper::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Next Transitions
    ///
    /// 获取下一个流转状态列表
    #[oai(path = "/:flow_inst_id/transition/next", method = "put")]
    async fn find_next_transitions(
        &self,
        flow_inst_id: Path<String>,
        next_req: Json<FlowInstFindNextTransitionsReq>,
        ctx: TardisContextExtractor,
        _request: &Request,
    ) -> TardisApiResult<Vec<FlowInstFindNextTransitionResp>> {
        let funs = flow_constants::get_tardis_inst();
        let inst = FlowInstServ::get(&flow_inst_id.0, &funs, &ctx.0).await?;
        let result = FlowInstServ::find_next_transitions(&inst, &next_req.0, &funs, &ctx.0).await?;
        task_handler_helper::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find the state and transfer information of the specified model in batch
    ///
    /// 批量获取指定模型的状态及流转信息
    #[oai(path = "/batch/state_transitions", method = "put")]
    async fn find_state_and_next_transitions(
        &self,
        find_req: Json<Vec<FlowInstFindStateAndTransitionsReq>>,
        ctx: TardisContextExtractor,
        _request: &Request,
    ) -> TardisApiResult<Vec<FlowInstFindStateAndTransitionsResp>> {
        let funs = flow_constants::get_tardis_inst();
        let result = FlowInstServ::find_state_and_next_transitions(&find_req.0, &funs, &ctx.0).await?;
        task_handler_helper::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Transfer State By Transaction Id
    ///
    /// 通过动作ID流转状态
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
        let inst = FlowInstServ::get(&flow_inst_id.0, &funs, &ctx.0).await?;
        FlowInstServ::check_transfer_vars(&inst, &mut transfer, &funs, &ctx.0).await?;
        funs.begin().await?;
        let result = FlowInstServ::transfer(
            &inst,
            &transfer,
            false,
            FlowExternalCallbackOp::Default,
            loop_check_helper::InstancesTransition::default(),
            &ctx.0,
            &funs,
        )
        .await?;
        funs.commit().await?;
        task_handler_helper::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Batch transfer State By Transaction Id
    ///
    /// 批量流转
    #[oai(path = "/batch/:flow_inst_ids/transition/transfer", method = "put")]
    async fn batch_transfer(
        &self,
        flow_inst_ids: Path<String>,
        transfer_req: Json<FlowInstTransferReq>,
        ctx: TardisContextExtractor,
        _request: &Request,
    ) -> TardisApiResult<Vec<FlowInstTransferResp>> {
        let funs = flow_constants::get_tardis_inst();
        let mut result = vec![];
        let flow_inst_ids: Vec<_> = flow_inst_ids.split(',').collect();
        let raw_transfer_req = transfer_req.0;
        let mut flow_inst_transfer = vec![];
        for flow_inst_id in &flow_inst_ids {
            let mut transfer_req = raw_transfer_req.clone();
            let inst = FlowInstServ::get(flow_inst_id, &funs, &ctx.0).await?;
            FlowInstServ::check_transfer_vars(&inst, &mut transfer_req, &funs, &ctx.0).await?;
            flow_inst_transfer.push((inst, transfer_req));
        }
        for (inst, transfer_req) in flow_inst_transfer {
            result.push(
                FlowInstServ::transfer(
                    &inst,
                    &transfer_req,
                    false,
                    FlowExternalCallbackOp::Default,
                    loop_check_helper::InstancesTransition::default(),
                    &ctx.0,
                    &funs,
                )
                .await?,
            );
        }
        task_handler_helper::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Modify Assigned [Deprecated]
    ///
    /// 同步执行人信息 [已废弃]
    #[oai(path = "/:flow_inst_id/transition/modify_assigned", method = "post")]
    async fn modify_assigned(
        &self,
        flow_inst_id: Path<String>,
        modify_req: Json<FlowInstModifyAssignedReq>,
        ctx: TardisContextExtractor,
        _request: &Request,
    ) -> TardisApiResult<Void> {
        let mut funs = flow_constants::get_tardis_inst();
        let vars = HashMap::from([("current_assigned".to_string(), Value::String(modify_req.0.current_assigned))]);
        let inst = FlowInstServ::get(&flow_inst_id.0, &funs, &ctx.0).await?;
        funs.begin().await?;
        FlowInstServ::modify_current_vars(&inst, &vars, loop_check_helper::InstancesTransition::default(), &funs, &ctx.0).await?;
        funs.commit().await?;
        task_handler_helper::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Modify list of variables
    ///
    /// 同步当前变量列表
    #[oai(path = "/:flow_inst_id/modify_current_vars", method = "patch")]
    async fn modify_current_vars(
        &self,
        flow_inst_id: Path<String>,
        modify_req: Json<FlowInstModifyCurrentVarsReq>,
        ctx: TardisContextExtractor,
        _request: &Request,
    ) -> TardisApiResult<Void> {
        let mut funs = flow_constants::get_tardis_inst();
        let inst = FlowInstServ::get(&flow_inst_id.0, &funs, &ctx.0).await?;
        funs.begin().await?;
        FlowInstServ::modify_current_vars(&inst, &modify_req.0.vars, loop_check_helper::InstancesTransition::default(), &funs, &ctx.0).await?;
        funs.commit().await?;
        task_handler_helper::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// operate flow instance
    ///
    /// 执行实例的操作
    #[oai(path = "/:flow_inst_id/operate", method = "post")]
    async fn operate(&self, flow_inst_id: Path<String>, operate_req: Json<FlowInstOperateReq>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<Void> {
        let mut funs = flow_constants::get_tardis_inst();
        let inst = FlowInstServ::get(&flow_inst_id.0, &funs, &ctx.0).await?;
        funs.begin().await?;
        FlowInstServ::operate(&inst, &operate_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        task_handler_helper::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// add comment
    ///
    /// 添加评论
    #[oai(path = "/:flow_inst_id/add_comment", method = "post")]
    async fn add_comment(&self, flow_inst_id: Path<String>, comment_req: Json<FlowInstCommentReq>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<String> {
        let mut funs = flow_constants::get_tardis_inst();
        let inst = FlowInstServ::get(&flow_inst_id.0, &funs, &ctx.0).await?;
        funs.begin().await?;
        let result = FlowInstServ::add_comment(&inst, &comment_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        task_handler_helper::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Batch checking of operating privileges
    ///
    /// 批量检查操作权限
    #[oai(path = "/batch/check_auth", method = "patch")]
    async fn batch_check_auth(&self, req: Json<FlowInstBatchCheckAuthReq>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<Vec<String>> {
        let funs = flow_constants::get_tardis_inst();
        let result = FlowInstServ::batch_check_auth(req.0.flow_inst_ids, &funs, &ctx.0).await?;
        task_handler_helper::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Getting the adapted related model
    ///
    /// 获取适配的关联模型
    #[oai(path = "/rel_model", method = "post")]
    async fn get_rel_model(&self, req: Json<FlowInstFindRelModelReq>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<Option<FlowModelDetailResp>> {
        let funs = flow_constants::get_tardis_inst();
        let result = FlowInstServ::find_rel_model(req.transition_id.clone(), &req.tag, &req.vars, &funs, &ctx.0).await?;
        task_handler_helper::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }
}
