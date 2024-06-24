use std::collections::HashMap;

use bios_basic::rbum::{helper::rbum_scope_helper, rbum_enumeration::RbumScopeLevelKind};
use tardis::{
    basic::dto::TardisContext,
    web::{
        context_extractor::TardisContextExtractor,
        poem::{web::Json, Request},
        poem_openapi,
        web_resp::{TardisApiResult, TardisResp},
    },
};

use crate::{
    dto::flow_model_dto::{FlowModelAggResp, FlowModelAssociativeOperationKind, FlowModelCopyOrReferenceReq},
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
        let orginal_models = FlowModelServ::find_rel_models(None, true, &funs, &ctx.0).await?;
        let mock_ctx = match req.0.op {
            FlowModelAssociativeOperationKind::Copy => ctx.0.clone(),
            FlowModelAssociativeOperationKind::Reference => TardisContext {
                own_paths: rbum_scope_helper::get_path_item(RbumScopeLevelKind::L1.to_int(), &ctx.0.own_paths).unwrap_or_default(),
                ..ctx.0.clone()
            },
        };
        for (tag, rel_model_id) in req.0.rel_model_ids {
            let orginal_model_id = orginal_models.get(&tag).map(|orginal_model| orginal_model.id.clone());
            result.insert(
                rel_model_id.clone(),
                FlowModelServ::copy_or_reference_model(orginal_model_id, &rel_model_id, Some(ctx.0.own_paths.clone()), &req.0.op, Some(false), &funs, &mock_ctx).await?,
            );
        }
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }
}
