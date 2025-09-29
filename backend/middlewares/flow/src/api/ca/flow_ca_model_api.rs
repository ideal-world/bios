use std::collections::HashMap;

use bios_basic::rbum::{rbum_enumeration::RbumRelFromKind, serv::rbum_item_serv::RbumItemCrudOperation};
use tardis::web::{
    context_extractor::TardisContextExtractor,
    poem::{web::Json, Request},
    poem_openapi,
    web_resp::{TardisApiResult, TardisResp},
};

use crate::{
    dto::flow_model_dto::{FlowModelAggResp, FlowModelAssociativeOperationKind, FlowModelCopyOrReferenceReq, FlowModelKind, FlowModelSingleCopyOrReferenceReq},
    flow_constants,
    helper::task_handler_helper,
    serv::{flow_model_serv::FlowModelServ, flow_rel_serv::{FlowRelKind, FlowRelServ}},
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
        let orginal_models = FlowModelServ::find_rel_model_map(req.0.rel_template_id.clone(), None, true, &funs, &ctx.0).await?;
        let mut result = HashMap::new();
        for (tag, orginal_model) in orginal_models {
            // 若不存在对应tag的模型，则直接删除
            if let Some(rel_model_id) = req.0.rel_model_ids.get(&tag) {
                let update_states = req.update_states.as_ref().map(|update_states| update_states.get(&tag).cloned().unwrap_or_default());
                let new_model = FlowModelServ::copy_or_reference_main_model(rel_model_id, &req.0.op, FlowModelKind::AsModel, req.0.rel_template_id.clone(), &update_states, None, &funs, &ctx.0).await?;
                result.insert(rel_model_id.clone(), new_model);
            } else {
                FlowModelServ::delete_item(&orginal_model.id, &funs, &ctx.0).await?;
            }
        }

        if req.0.op == FlowModelAssociativeOperationKind::Reference || req.0.op == FlowModelAssociativeOperationKind::ReferenceOrCopy {
            if let (Some(app_id), Some(rel_template_id)) = (FlowModelServ::get_app_id_by_ctx(&ctx.0), &req.0.rel_template_id) {
                // 若存在引用操作，且当前处于应用层，则需要更新应用的关联模型
                if let Some(old_template_id) = FlowModelServ::find_rel_template_id(&funs, &ctx.0).await? {
                    FlowRelServ::delete_simple_rel(&FlowRelKind::FlowAppTemplate, &app_id, &old_template_id, &funs, &ctx.0).await?;
                }
                FlowRelServ::add_simple_rel(&FlowRelKind::FlowAppTemplate, &app_id, RbumRelFromKind::Item, rel_template_id, None, None, true, true, None, &funs, &ctx.0).await?;
            }
        }
        funs.commit().await?;
        task_handler_helper::execute_async_task(&ctx.0).await?;
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
        // let _orginal_models = FlowModelServ::clean_rel_models(None, None, Some(vec![req.0.tag.clone()]), &funs, &ctx.0).await?;
        let update_states = req.update_states.as_ref().map(|update_states| update_states.get(&req.0.tag).cloned().unwrap_or_default());
        let new_model = FlowModelServ::copy_or_reference_main_model(&req.0.rel_model_id, &req.0.op, FlowModelKind::AsModel, None, &update_states, None, &funs, &ctx.0).await?;

        funs.commit().await?;
        task_handler_helper::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(new_model)
    }
}
