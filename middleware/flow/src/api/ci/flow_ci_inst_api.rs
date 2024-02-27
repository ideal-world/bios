use tardis::basic::dto::TardisContext;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::poem_openapi::{self, param::Query};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};
use tardis::{log, tokio};

use crate::dto::flow_inst_dto::{FlowInstBatchBindReq, FlowInstBatchBindResp, FlowInstBindReq, FlowInstDetailResp, FlowInstStartReq};
use crate::flow_constants;
use crate::serv::flow_inst_serv::FlowInstServ;
#[derive(Clone)]
pub struct FlowCiInstApi;

/// Flow Config process API
#[poem_openapi::OpenApi(prefix_path = "/ci/inst")]
impl FlowCiInstApi {
    /// Bind Single Instance / 绑定单个实例
    #[oai(path = "/bind", method = "post")]
    async fn bind(&self, add_req: Json<FlowInstBindReq>, ctx: TardisContextExtractor) -> TardisApiResult<String> {
        let mut funs = flow_constants::get_tardis_inst();
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
    async fn batch_bind(&self, add_req: Json<FlowInstBatchBindReq>, ctx: TardisContextExtractor) -> TardisApiResult<Vec<FlowInstBatchBindResp>> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        let result = FlowInstServ::batch_bind(&add_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(result)
    }

    /// Get list of instance id by rel_business_obj_id / 通过业务ID获取实例信息
    #[oai(path = "/find_detail_by_obj_ids", method = "get")]
    async fn find_detail_by_obj_ids(&self, obj_ids: Query<String>, ctx: TardisContextExtractor) -> TardisApiResult<Vec<FlowInstDetailResp>> {
        let funs = flow_constants::get_tardis_inst();
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

    /// refresh var_collect / 刷新var_collect
    #[oai(path = "/refresh_var_collect", method = "get")]
    async fn refresh_var_collect(&self, inst_id: Query<String>, assigned_id: Query<String>) -> TardisApiResult<Void> {
        let funs = flow_constants::get_tardis_inst();
        let global_ctx = TardisContext::default();
        FlowInstServ::modify_assigned(&inst_id.0, &assigned_id.0, &funs, &global_ctx).await?;

        TardisResp::ok(Void {})
    }
}
