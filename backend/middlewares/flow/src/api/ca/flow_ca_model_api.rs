use std::collections::HashMap;

use tardis::web::{
    context_extractor::TardisContextExtractor,
    poem::{web::Json, Request},
    poem_openapi,
    web_resp::{TardisApiResult, TardisResp},
};

use crate::{dto::flow_model_dto::FlowModelCopyOrReferenceReq, flow_constants, serv::flow_model_serv::FlowModelServ};

#[derive(Clone)]
pub struct FlowCaModelApi;

/// Flow model process API
#[poem_openapi::OpenApi(prefix_path = "/ca/model")]
impl FlowCaModelApi {
    /// Creating or referencing models
    ///
    /// 创建或引用模型（rel_model_id：关联模型ID, op：关联模型操作类型（复制或者引用），is_create_copy：是否创建副本（当op为复制时需指定，默认不需要））
    #[oai(path = "/copy_or_reference_model", method = "post")]
    async fn copy_or_reference_model(&self, req: Json<FlowModelCopyOrReferenceReq>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<HashMap<String, String>> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        let mut result = HashMap::new();
        for rel_model_id in req.0.rel_model_ids {
            result.insert(
                rel_model_id.clone(),
                FlowModelServ::copy_or_reference_model(&rel_model_id, &req.0.op, Some(false), &funs, &ctx.0).await?,
            );
        }
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }
}
