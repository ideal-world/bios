use std::collections::HashMap;

use tardis::web::{
    context_extractor::TardisContextExtractor,
    poem::{web::Json, Request},
    poem_openapi,
    web_resp::{TardisApiResult, TardisResp},
};

use crate::{
    dto::flow_model_dto::{FlowModelAggResp, FlowModelCopyOrReferenceReq, FlowModelKind, FlowModelSingleCopyOrReferenceReq},
    flow_constants,
    serv::{flow_inst_serv::FlowInstServ, flow_model_serv::FlowModelServ},
};

#[derive(Clone)]
pub struct FlowCaModelApi;

/// Flow model process API
#[poem_openapi::OpenApi(prefix_path = "/ca/model")]
impl FlowCaModelApi {
    /// Creating or referencing models
    ///
    /// 创建或引用模型
    #[oai(path = "/copy_or_reference_model", method = "post")]
    async fn copy_or_reference_model(
        &self,
        req: Json<FlowModelCopyOrReferenceReq>,
        ctx: TardisContextExtractor,
        _request: &Request,
    ) -> TardisApiResult<HashMap<String, FlowModelAggResp>> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        let _orginal_models = FlowModelServ::clean_rel_models(None, None, None, &funs, &ctx.0).await?;
        let mut result = HashMap::new();
        for (_, rel_model_id) in req.0.rel_model_ids {
            let new_model = FlowModelServ::copy_or_reference_model(&rel_model_id, &req.0.op, FlowModelKind::AsModel, &funs, &ctx.0).await?;
            FlowInstServ::batch_update_when_switch_model(&new_model, None, None, &funs, &ctx.0).await?;

            result.insert(rel_model_id.clone(), new_model);
        }

        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Creating or referencing single model
    ///
    /// 创建或引用单个模型
    #[oai(path = "/copy_or_reference_single_model", method = "post")]
    async fn copy_or_reference_single_model(
        &self,
        req: Json<FlowModelSingleCopyOrReferenceReq>,
        ctx: TardisContextExtractor,
        _request: &Request,
    ) -> TardisApiResult<FlowModelAggResp> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        let _orginal_models = FlowModelServ::clean_rel_models(None, None, Some(vec![req.0.tag.clone()]), &funs, &ctx.0).await?;
        let new_model = FlowModelServ::copy_or_reference_model(&req.0.rel_model_id, &req.0.op, FlowModelKind::AsModel, &funs, &ctx.0).await?;
        FlowInstServ::batch_update_when_switch_model(&new_model, None, None, &funs, &ctx.0).await?;

        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(new_model)
    }
}
