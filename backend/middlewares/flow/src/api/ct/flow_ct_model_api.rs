use std::collections::HashMap;

use tardis::web::{
    context_extractor::TardisContextExtractor,
    poem::{web::Json, Request},
    poem_openapi,
    web_resp::{TardisApiResult, TardisResp},
};

use crate::{
    dto::flow_model_dto::FlowModelCopyOrReferenceReq,
    flow_constants,
    serv::{
        flow_model_serv::FlowModelServ,
        flow_rel_serv::{FlowRelKind, FlowRelServ},
    },
};

#[derive(Clone)]
pub struct FlowCtModelApi;

/// Flow model process API
#[poem_openapi::OpenApi(prefix_path = "/ct/model")]
impl FlowCtModelApi {
    /// Creating or referencing models
    ///
    /// 创建或引用模型
    #[oai(path = "/copy_or_reference_model", method = "post")]
    async fn copy_or_reference_model(&self, req: Json<FlowModelCopyOrReferenceReq>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<HashMap<String, String>> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        let mut result = HashMap::new();
        for rel_model_id in req.0.rel_model_ids {
            let added_model_id = FlowModelServ::copy_or_reference_model(&rel_model_id, None, &req.0.op, Some(true), &funs, &ctx.0).await?;
            result.insert(rel_model_id.clone(), added_model_id.clone());
            if let Some(rel_template_id) = &req.0.rel_template_id {
                FlowRelServ::add_simple_rel(
                    &FlowRelKind::FlowModelTemplate,
                    &added_model_id,
                    rel_template_id,
                    None,
                    None,
                    false,
                    true,
                    None,
                    &funs,
                    &ctx.0,
                )
                .await?;
            }
        }
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }
}
