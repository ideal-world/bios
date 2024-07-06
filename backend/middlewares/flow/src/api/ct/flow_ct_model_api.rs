use std::collections::HashMap;

use bios_basic::rbum::{dto::rbum_filer_dto::RbumBasicFilterReq, serv::rbum_item_serv::RbumItemCrudOperation};
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
    dto::flow_model_dto::{
        FlowModelAggResp, FlowModelAssociativeOperationKind, FlowModelCopyOrReferenceReq, FlowModelFilterReq, FlowModelFindRelNameByTemplateIdsReq, FlowModelModifyReq,
    },
    flow_constants,
    serv::{
        flow_inst_serv::FlowInstServ,
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
        let orginal_models = FlowModelServ::clean_rel_models(
            req.0.rel_template_id.clone(),
            Some(req.0.rel_model_ids.clone().values().cloned().collect_vec()),
            None,
            &funs,
            &ctx.0,
        )
        .await?;
        let mut result = HashMap::new();
        for (tag, rel_model_id) in req.0.rel_model_ids {
            let orginal_model_id = orginal_models.get(&tag).map(|orginal_model| orginal_model.id.clone());
            if orginal_model_id.clone().unwrap_or_default() == rel_model_id {
                continue;
            }
            let new_model = FlowModelServ::copy_or_reference_model(&rel_model_id, None, &req.0.op, Some(true), &funs, &ctx.0).await?;
            if let Some(rel_template_id) = &req.0.rel_template_id {
                FlowRelServ::add_simple_rel(
                    &FlowRelKind::FlowModelTemplate,
                    &new_model.id,
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
            FlowInstServ::batch_update_when_switch_model(
                orginal_model_id,
                &new_model.tag,
                &new_model.id,
                new_model.states.clone(),
                &new_model.init_state_id,
                &funs,
                &ctx.0,
            )
            .await?;
            result.insert(rel_model_id.clone(), new_model);
        }
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Creating or referencing models
    ///
    /// 创建或引用模型
    #[oai(path = "/copy_models_by_template_id/:from_template_id/:to_template_id", method = "post")]
    async fn copy_models_by_template_id(
        &self,
        from_template_id: Path<String>,
        to_template_id: Path<String>,
        ctx: TardisContextExtractor,
        _request: &Request,
    ) -> TardisApiResult<HashMap<String, FlowModelAggResp>> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        let orginal_models = FlowModelServ::clean_rel_models(Some(to_template_id.0.clone()), None, None, &funs, &ctx.0).await?;
        let mut result = HashMap::new();
        for from_model in FlowModelServ::find_detail_items(
            &FlowModelFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(
                        FlowRelServ::find_to_simple_rels(&FlowRelKind::FlowModelTemplate, &from_template_id.0, None, None, &funs, &ctx.0)
                            .await?
                            .into_iter()
                            .map(|rel| rel.rel_id)
                            .collect_vec(),
                    ),
                    ignore_scope: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            None,
            None,
            &funs,
            &ctx.0,
        )
        .await?
        {
            let new_model = FlowModelServ::copy_or_reference_model(&from_model.rel_model_id, None, &FlowModelAssociativeOperationKind::Copy, Some(true), &funs, &ctx.0).await?;
            FlowRelServ::add_simple_rel(
                &FlowRelKind::FlowModelTemplate,
                &new_model.id,
                &to_template_id.0,
                None,
                None,
                false,
                true,
                None,
                &funs,
                &ctx.0,
            )
            .await?;
            FlowModelServ::modify_item(
                &new_model.id,
                &mut FlowModelModifyReq {
                    rel_model_id: Some(from_model.rel_model_id.clone()),
                    ..Default::default()
                },
                &funs,
                &ctx.0,
            )
            .await?;
            FlowInstServ::batch_update_when_switch_model(
                orginal_models.get(&new_model.tag).map(|orginal_model| orginal_model.id.clone()),
                &new_model.tag,
                &new_model.id,
                new_model.states.clone(),
                &new_model.init_state_id,
                &funs,
                &ctx.0,
            )
            .await?;
            result.insert(from_model.rel_model_id.clone(), new_model);
        }
        funs.commit().await?;
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
