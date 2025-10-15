use std::collections::HashMap;

use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use itertools::Itertools;
use tardis::{
    basic::error::TardisError,
    web::{
        context_extractor::TardisContextExtractor,
        poem::{web::Json, Request},
        poem_openapi::{self, param::Path},
        web_resp::{TardisApiResult, TardisResp, Void},
    },
};

use crate::{
    dto::flow_model_dto::{FlowModelAggResp, FlowModelAssociativeOperationKind, FlowModelCopyOrReferenceReq, FlowModelFindRelNameByTemplateIdsReq, FlowModelKind},
    flow_constants,
    helper::task_handler_helper,
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
    async fn copy_or_reference_model(
        &self,
        req: Json<FlowModelCopyOrReferenceReq>,
        ctx: TardisContextExtractor,
        _request: &Request,
    ) -> TardisApiResult<HashMap<String, FlowModelAggResp>> {
        let mut funs = flow_constants::get_tardis_inst();
        if req.0.rel_template_id.is_none() {
            return TardisResp::err(TardisError::bad_request("rel_template_id can't be empty", ""));
        }
        funs.begin().await?;
        let orginal_models = FlowModelServ::find_rel_model_map(
            req.0.rel_template_id.clone(),
            Some(req.0.rel_model_ids.clone().keys().cloned().collect_vec()),
            true,
            &funs,
            &ctx.0,
        )
        .await?;
        let mut result = HashMap::new();
        for (tag, rel_model_id) in &req.0.rel_model_ids {
            let orginal_model_id = orginal_models.get(tag).map(|orginal_model| orginal_model.id.clone());
            if orginal_model_id.clone().unwrap_or_default() == rel_model_id.clone() {
                continue;
            }
            let update_states = req.update_states.as_ref().map(|update_states| update_states.get(tag).cloned().unwrap_or_default());
            let new_model = FlowModelServ::copy_or_reference_main_model(
                rel_model_id,
                &FlowModelAssociativeOperationKind::ReferenceOrCopy,
                FlowModelKind::AsTemplateAndAsModel,
                req.0.rel_template_id.clone(),
                &update_states,
                None,
                &funs,
                &ctx.0,
            )
            .await?;
            result.insert(rel_model_id.clone(), new_model);
        }
        funs.commit().await?;
        task_handler_helper::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// batch copy models by template_id
    ///
    /// 通过模板ID复制模型
    #[oai(path = "/copy_models_by_template_id/:from_template_id/:to_template_id", method = "post")]
    async fn copy_models_by_template_id(
        &self,
        from_template_id: Path<String>,
        to_template_id: Path<String>,
        ctx: TardisContextExtractor,
        _request: &Request,
    ) -> TardisApiResult<HashMap<String, String>> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        let result = FlowModelServ::copy_models_by_template_id(&from_template_id.0, &to_template_id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        task_handler_helper::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// batch delete models by rel_template_id
    ///
    /// 通过关联模板ID删除模型
    #[oai(path = "/delete_by_rel_template_id/:rel_template_id", method = "delete")]
    async fn delete_by_rel_template_id(&self, rel_template_id: Path<String>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<Void> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        for rel in FlowRelServ::find_to_simple_rels(&FlowRelKind::FlowModelTemplate, &rel_template_id.0, None, None, &funs, &ctx.0).await? {
            FlowModelServ::delete_item(&rel.rel_id, &funs, &ctx.0).await?;
        }
        funs.commit().await?;
        task_handler_helper::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void)
    }

    /// Get associated model names by template ID, multiple comma separated
    ///
    /// 通过模板ID获取关联的模型名，多个逗号隔开
    #[oai(path = "/find_rel_name_by_template_ids", method = "post")]
    async fn find_rel_name_by_template_ids(
        &self,
        req: Json<FlowModelFindRelNameByTemplateIdsReq>,
        ctx: TardisContextExtractor,
        _request: &Request,
    ) -> TardisApiResult<HashMap<String, Vec<String>>> {
        let funs = flow_constants::get_tardis_inst();
        let mut result = HashMap::new();
        for rel_template_id in req.0.rel_template_ids {
            result.insert(
                rel_template_id.clone(),
                FlowRelServ::find_to_simple_rels(&FlowRelKind::FlowModelTemplate, &rel_template_id, None, None, &funs, &ctx.0)
                    .await?
                    .into_iter()
                    .map(|rel| rel.rel_name)
                    .collect_vec(),
            );
        }

        TardisResp::ok(result)
    }
}
