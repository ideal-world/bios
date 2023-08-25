use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use crate::dto::flow_inst_dto::{FlowInstBatchBindReq, FlowInstBatchBindResp, FlowInstBindReq, FlowInstStartReq};
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
        funs.begin().await?;
        let result = FlowInstServ::start(
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
}
