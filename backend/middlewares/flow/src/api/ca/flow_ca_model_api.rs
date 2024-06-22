use std::collections::HashMap;

use itertools::Itertools;
use tardis::web::{
    context_extractor::TardisContextExtractor,
    poem::{web::Json, Request},
    poem_openapi,
    web_resp::{TardisApiResult, TardisResp},
};

use crate::{
    dto::flow_model_dto::{FlowModelAggResp, FlowModelCopyOrReferenceReq},
    flow_constants,
    serv::flow_model_serv::FlowModelServ,
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
        let mut result = HashMap::new();
        let orginal_models = FlowModelServ::find_rel_models(
            req.0.rel_model_ids.clone().keys().map(|tag| tag.to_string()).collect_vec(),
            req.0.rel_template_id.clone(),
            true,
            &funs,
            &ctx.0,
        )
        .await?;
        for (tag, rel_model_id) in req.0.rel_model_ids {
            let orginal_model_id = orginal_models.get(&tag).map(|orginal_model| orginal_model.id.clone());
            result.insert(
                rel_model_id.clone(),
                FlowModelServ::copy_or_reference_model(orginal_model_id, &rel_model_id, None, &req.0.op, Some(false), &funs, &ctx.0).await?,
            );
        }
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }
}
