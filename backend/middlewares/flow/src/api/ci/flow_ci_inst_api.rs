use std::collections::HashMap;

use bios_basic::rbum::helper::rbum_scope_helper::check_without_owner_and_unsafe_fill_ctx;
use itertools::Itertools;
use tardis::basic::dto::TardisContext;
use tardis::chrono::Utc;
use tardis::log::{debug, warn};
use tardis::serde_json::Value;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::web::Path;
use tardis::web::poem::Request;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::poem_openapi::{self, param::Query};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use crate::dto::flow_external_dto::FlowExternalCallbackOp;
use crate::dto::flow_inst_dto::{
    FlowInstAbortReq, FlowInstBatchBindReq, FlowInstBatchBindResp, FlowInstBindReq, FlowInstDetailResp, FlowInstFilterReq, FlowInstFindNextTransitionsReq,
    FlowInstFindStateAndTransitionsReq, FlowInstFindStateAndTransitionsResp, FlowInstModifyAssignedReq, FlowInstModifyCurrentVarsReq, FlowInstOperateReq, FlowInstStartReq,
    FlowInstSummaryResp, FlowInstTransferReq, FlowInstTransferResp, FlowInstTransitionInfo, FlowOperationContext,
};
use crate::dto::flow_state_dto::FlowSysStateKind;
use crate::dto::flow_transition_dto::FlowTransitionFilterReq;
use crate::flow_constants;
use crate::helper::loop_check_helper;
use crate::serv::clients::search_client::FlowSearchClient;
use crate::serv::flow_event_serv::FlowEventServ;
use crate::serv::flow_inst_serv::FlowInstServ;
use crate::serv::flow_transition_serv::FlowTransitionServ;
#[derive(Clone)]
pub struct FlowCiInstApi;

/// Flow Config process API
#[poem_openapi::OpenApi(prefix_path = "/ci/inst")]
impl FlowCiInstApi {
    /// Start Instance
    ///
    /// 启动实例
    #[oai(path = "/", method = "post")]
    async fn start(&self, add_req: Json<FlowInstStartReq>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        let funs = flow_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        let result = FlowInstServ::start(&add_req.0, None, &funs, &ctx.0).await?;
        FlowSearchClient::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Get Instance By Instance Id
    ///
    /// 获取实例信息
    #[oai(path = "/:flow_inst_id", method = "get")]
    async fn get(&self, flow_inst_id: Path<String>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<FlowInstDetailResp> {
        let funs = flow_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        let mut result = FlowInstServ::get(&flow_inst_id.0, &funs, &ctx.0).await?;
        // @TODO 临时处理方式，后续需增加接口
        result.transitions = Some(
            FlowInstServ::find_next_transitions(&result, &FlowInstFindNextTransitionsReq { vars: None }, &funs, &ctx.0)
                .await?
                .into_iter()
                .map(|tran| FlowInstTransitionInfo {
                    id: tran.next_flow_transition_id,
                    start_time: Utc::now(),
                    op_ctx: FlowOperationContext::default(),
                    output_message: Some(tran.next_flow_transition_name),
                    from_state_id: None,
                    from_state_name: None,
                    target_state_id: None,
                    target_state_name: None,
                })
                .collect_vec(),
        );
        FlowSearchClient::execute_async_task(&ctx.0).await?;
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
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<FlowInstFindStateAndTransitionsResp>> {
        let funs = flow_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        let result = FlowInstServ::find_state_and_next_transitions(&find_req.0, &funs, &ctx.0).await?;
        FlowSearchClient::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Abort Instance
    ///
    /// 中止实例
    #[oai(path = "/:flow_inst_id", method = "put")]
    async fn abort(&self, flow_inst_id: Path<String>, abort_req: Json<FlowInstAbortReq>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let mut funs = flow_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        funs.begin().await?;
        FlowInstServ::abort(&flow_inst_id.0, &abort_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        FlowSearchClient::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Transfer State By State Id
    ///
    /// 流转
    #[oai(path = "/:flow_inst_id/transition/transfer", method = "put")]
    async fn transfer(
        &self,
        flow_inst_id: Path<String>,
        transfer_req: Json<FlowInstTransferReq>,
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<FlowInstTransferResp> {
        let funs = flow_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        let mut transfer = transfer_req.0;
        let inst = FlowInstServ::get(&flow_inst_id.0, &funs, &ctx.0).await?;
        FlowInstServ::check_transfer_vars(&inst, &mut transfer, &funs, &ctx.0).await?;
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
        FlowSearchClient::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Batch transfer State By State Id
    ///
    /// 批量流转
    #[oai(path = "/batch/:flow_inst_ids/transition/transfer", method = "put")]
    async fn batch_transfer(
        &self,
        flow_inst_ids: Path<String>,
        transfer_req: Json<FlowInstTransferReq>,
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<FlowInstTransferResp>> {
        let funs = flow_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        debug!("batch transfer request: {:?}", request);
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
        FlowSearchClient::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Modify Assigned
    ///
    /// 同步执行人信息
    #[oai(path = "/:flow_inst_id/transition/modify_assigned", method = "post")]
    async fn modify_assigned(
        &self,
        flow_inst_id: Path<String>,
        modify_req: Json<FlowInstModifyAssignedReq>,
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Void> {
        let mut funs = flow_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        let vars = HashMap::from([("assigned_to".to_string(), Value::String(modify_req.0.current_assigned))]);
        let inst = FlowInstServ::get(&flow_inst_id.0, &funs, &ctx.0).await?;
        funs.begin().await?;
        FlowInstServ::modify_current_vars(&inst, &vars, loop_check_helper::InstancesTransition::default(), &funs, &ctx.0).await?;
        funs.commit().await?;
        FlowSearchClient::execute_async_task(&ctx.0).await?;
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
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Void> {
        let mut funs = flow_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        let inst = FlowInstServ::get(&flow_inst_id.0, &funs, &ctx.0).await?;
        funs.begin().await?;
        FlowInstServ::modify_current_vars(&inst, &modify_req.0.vars, loop_check_helper::InstancesTransition::default(), &funs, &ctx.0).await?;
        funs.commit().await?;
        FlowSearchClient::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Bind Single Instance
    ///
    /// 绑定单个实例
    #[oai(path = "/bind", method = "post")]
    async fn bind(&self, add_req: Json<FlowInstBindReq>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        let mut funs = flow_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        let inst_id = FlowInstServ::get_inst_ids_by_rel_business_obj_id(vec![add_req.0.rel_business_obj_id.clone()], Some(true), &funs, &ctx.0).await?.pop();
        let result = if let Some(inst_id) = inst_id {
            inst_id
        } else {
            funs.begin().await?;
            let inst_id = FlowInstServ::start(
                &FlowInstStartReq {
                    rel_business_obj_id: add_req.0.rel_business_obj_id.clone(),
                    tag: add_req.0.tag.clone(),
                    create_vars: add_req.0.create_vars.clone(),
                    check_vars: None,
                    transition_id: None,
                    vars: None,
                    log_text: None,
                    rel_transition_id: None,
                    rel_child_objs: None,
                    operator_map: None,
                    ..Default::default()
                },
                add_req.0.current_state_name.clone(),
                &funs,
                &ctx.0,
            )
            .await?;
            funs.commit().await?;
            inst_id
        };
        FlowSearchClient::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Batch Bind Instance
    ///
    /// 批量绑定实例 （初始化）
    #[oai(path = "/batch_bind", method = "post")]
    async fn batch_bind(&self, add_req: Json<FlowInstBatchBindReq>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Vec<FlowInstBatchBindResp>> {
        let mut funs = flow_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        funs.begin().await?;
        let result = FlowInstServ::batch_bind(&add_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        FlowSearchClient::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Get list of instance id by rel_business_obj_id
    ///
    /// 通过业务ID获取实例信息
    #[oai(path = "/find_detail_by_obj_ids", method = "get")]
    async fn find_detail_by_obj_ids(&self, obj_ids: Query<String>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Vec<FlowInstDetailResp>> {
        let funs = flow_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        let rel_business_obj_ids: Vec<_> = obj_ids.0.split(',').map(|id| id.to_string()).collect();
        let inst_ids = FlowInstServ::get_inst_ids_by_rel_business_obj_id(rel_business_obj_ids, Some(true), &funs, &ctx.0).await?;
        let mut result = vec![];
        for inst_id in inst_ids {
            if let Ok(inst_detail) = FlowInstServ::get(&inst_id, &funs, &ctx.0).await {
                result.push(inst_detail);
            }
        }
        FlowSearchClient::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// sync instance status to search
    ///
    /// 同步状态信息
    #[oai(path = "/status/sync", method = "get")]
    async fn sync_status(&self, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<Void> {
        let funs = flow_constants::get_tardis_inst();
        FlowInstServ::sync_status(&funs, &ctx.0).await?;
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
        FlowSearchClient::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// sync instance status to search
    ///
    /// 批量操作实例
    #[oai(path = "/:flow_inst_id/batch_operate", method = "post")]
    async fn batch_operate(
        &self,
        flow_inst_id: Path<String>,
        operate_req: Json<HashMap<String, FlowInstOperateReq>>,
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Void> {
        warn!("ci inst batch_operate flow_inst_id: {:?}, req: {:?}", flow_inst_id, operate_req);
        let mut funs = flow_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        funs.begin().await?;
        FlowInstServ::batch_operate(&flow_inst_id.0, &operate_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        FlowSearchClient::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;

        TardisResp::ok(Void {})
    }

    /// sync instance status to search
    ///
    /// 自动触发前置条件(脚本)
    #[oai(path = "/auto_front_change", method = "post")]
    async fn auto_front_change(&self, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<Void> {
        let funs = flow_constants::get_tardis_inst();
        let trans_with_front_changes = FlowTransitionServ::find_detail_items(
            &FlowTransitionFilterReq {
                is_empty_front_changes: Some(false),
                ..Default::default()
            },
            &funs,
            &ctx.0,
        )
        .await?;
        for trans_with_front_change in trans_with_front_changes {
            let insts = FlowInstServ::find_detail_items(
                &FlowInstFilterReq {
                    flow_version_id: Some(trans_with_front_change.rel_flow_model_version_id.clone()),
                    current_state_id: Some(trans_with_front_change.from_flow_state_id),
                    main: Some(true),
                    ..Default::default()
                },
                &funs,
                &ctx.0,
            )
            .await?;
            for inst in insts {
                ctx.0
                    .add_async_task(Box::new(|| {
                        Box::pin(async move {
                            let curr_inst = inst.clone();
                            let inst_ctx = TardisContext {
                                own_paths: curr_inst.own_paths.clone(),
                                ..Default::default()
                            };
                            let task_handle = tardis::tokio::spawn(async move {
                                let mut funs = flow_constants::get_tardis_inst();
                                funs.begin().await.unwrap_or_default();
                                let _ = FlowEventServ::do_front_change(&curr_inst, loop_check_helper::InstancesTransition::default(), &inst_ctx, &funs).await;
                                funs.commit().await.unwrap_or_default();
                            });
                            match task_handle.await {
                                Ok(_) => {}
                                Err(e) => tardis::log::error!("Flow Instance {} do_front_change error:{:?}", inst.id, e),
                            }
                            Ok(())
                        })
                    }))
                    .await?;
            }
        }
        FlowSearchClient::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
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
        tags: Query<Option<String>>,
        finish: Query<Option<bool>>,
        finish_abort: Query<Option<bool>>,
        main: Query<Option<bool>>,
        current_state_id: Query<Option<String>>,
        current_state_sys_kind: Query<Option<FlowSysStateKind>>,
        with_sub: Query<Option<bool>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<TardisPage<FlowInstSummaryResp>> {
        let funs = flow_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        let result = FlowInstServ::paginate(
            flow_model_id.0,
            tags.0.map(|v| v.split(',').map(|s| s.to_string()).collect_vec()),
            finish.0,
            finish_abort.0,
            main.0,
            current_state_id.0,
            current_state_sys_kind.0,
            rel_business_obj_id.0,
            with_sub.0,
            page_number.0,
            page_size.0,
            &funs,
            &ctx.0,
        )
        .await?;
        FlowSearchClient::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }
}
