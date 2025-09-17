use std::collections::HashMap;

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemRelFilterReq};
use bios_basic::rbum::rbum_enumeration::RbumRelFromKind;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use itertools::Itertools;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use crate::dto::flow_model_dto::{
    FlowModelAddReq, FlowModelAggResp, FlowModelBindStateReq, FlowModelCopyOrReferenceCiReq, FlowModelDetailResp, FlowModelFIndOrCreatReq, FlowModelFilterReq, FlowModelFindRelNameByTemplateIdsReq, FlowModelFindRelStateResp, FlowModelKind, FlowModelModifyReq, FlowModelSortStatesReq, FlowModelStatus, FlowModelSummaryResp, FlowModelUnbindStateReq
};
use crate::dto::flow_model_version_dto::{FlowModelVersionBindState, FlowModelVersionDetailResp, FlowModelVersionModifyReq, FlowModelVersionModifyState};
use crate::dto::flow_state_dto::FlowStateRelModelModifyReq;
use crate::dto::flow_transition_dto::{FlowTransitionDetailResp, FlowTransitionSortStatesReq};
use crate::flow_constants;
use crate::helper::task_handler_helper;
use crate::serv::flow_model_serv::FlowModelServ;
use crate::serv::flow_rel_serv::{FlowRelKind, FlowRelServ};
#[derive(Clone)]
pub struct FlowCcModelApi;

/// Flow model process API
#[poem_openapi::OpenApi(prefix_path = "/cc/model")]
impl FlowCcModelApi {
    /// Add Model
    ///
    /// 添加模型
    #[oai(path = "/", method = "post")]
    async fn add(&self, mut add_req: Json<FlowModelAddReq>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<FlowModelAggResp> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        let model_id = FlowModelServ::add_item(&mut add_req.0, &funs, &ctx.0).await?;
        let result = FlowModelServ::get_item_detail_aggs(&model_id, true, &funs, &ctx.0).await?;
        funs.commit().await?;
        task_handler_helper::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Modify Model By Model Id
    ///
    /// 修改模型
    #[oai(path = "/:flow_model_id", method = "patch")]
    async fn modify(&self, flow_model_id: Path<String>, mut modify_req: Json<FlowModelModifyReq>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<Void> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        FlowModelServ::modify_model(&flow_model_id.0, &mut modify_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        task_handler_helper::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// GET Editing Model Version By Model Id
    ///
    /// 通过模型ID获取正在编辑的模型版本信息
    #[oai(path = "/:flow_model_id/find_editing_verion", method = "get")]
    async fn find_editing_verion(&self, flow_model_id: Path<String>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<FlowModelVersionDetailResp> {
        let funs = flow_constants::get_tardis_inst();
        let result = FlowModelServ::find_editing_verion(&flow_model_id.0, &funs, &ctx.0).await?;
        task_handler_helper::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Copy Model By Model Id
    ///
    /// 复制模型
    #[oai(path = "/copy/:flow_model_id", method = "patch")]
    async fn copy(&self, flow_model_id: Path<String>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<FlowModelAggResp> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        let rel_model = FlowModelServ::get_item(
            &flow_model_id.0,
            &FlowModelFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ignore_scope: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            &funs,
            &ctx.0,
        )
        .await?;
        let new_model_id = FlowModelServ::add_item(
            &mut FlowModelAddReq {
                name: format!("{}-副本", rel_model.name.clone()).into(),
                ..rel_model.clone().into()
            },
            &funs,
            &ctx.0,
        )
        .await?;
        let new_model = FlowModelServ::get_item_detail_aggs(&new_model_id, true, &funs, &ctx.0).await?;
        funs.commit().await?;
        task_handler_helper::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(new_model)
    }

    /// Get Model By Model Id
    ///
    /// 获取模型
    #[oai(path = "/:flow_model_id", method = "get")]
    async fn get(&self, flow_model_id: Path<String>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<FlowModelAggResp> {
        let funs = flow_constants::get_tardis_inst();
        let result = FlowModelServ::get_item_detail_aggs(&flow_model_id.0, true, &funs, &ctx.0).await?;
        task_handler_helper::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Get the list of models by template ID.
    /// Specific rules: If no template ID is specified, then get the template with empty template ID in the corresponding tag.
    /// Even if the template ID is specified, we need to get the template with empty template ID in the corresponding tag.
    ///
    /// 通过模板ID获取模型列表。
    /// 具体规则：未指定模板ID，则获取对应tag中置空模板ID的模板。
    /// 即使是指定模板ID，也需要获取对应tag中置空模板ID的模板
    #[oai(path = "/find_by_rel_template_id", method = "get")]
    async fn find_models_by_rel_template_id(
        &self,
        tag: Query<String>,
        template: Query<Option<bool>>,
        rel_template_id: Query<Option<String>>,
        ctx: TardisContextExtractor,
        _request: &Request,
    ) -> TardisApiResult<Vec<FlowModelSummaryResp>> {
        let funs = flow_constants::get_tardis_inst();
        TardisResp::ok(FlowModelServ::find_models_by_rel_template_id(tag.0, template.0, rel_template_id.0, &funs, &ctx.0).await?)
    }

    /// Find Models
    ///
    /// 获取模型列表
    #[oai(path = "/", method = "get")]
    #[allow(clippy::too_many_arguments)]
    async fn paginate(
        &self,
        flow_model_ids: Query<Option<String>>,
        name: Query<Option<String>>,
        tag: Query<Option<String>>,
        enabled: Query<Option<bool>>,
        status: Query<Option<FlowModelStatus>>,
        kind: Query<Option<FlowModelKind>>,
        rel_template_id: Query<Option<String>>,
        main: Query<Option<bool>>,
        with_sub: Query<Option<bool>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
        _request: &Request,
    ) -> TardisApiResult<TardisPage<FlowModelDetailResp>> {
        let funs = flow_constants::get_tardis_inst();
        let result = FlowModelServ::paginate_detail_items(
            &FlowModelFilterReq {
                basic: RbumBasicFilterReq {
                    ids: flow_model_ids.0.map(|ids| ids.split(',').map(|id| id.to_string()).collect::<Vec<String>>()),
                    own_paths: if rel_template_id.0.is_some() { Some("".to_string()) } else { None },
                    name: name.0,
                    with_sub_own_paths: with_sub.0.unwrap_or(false),
                    enabled: enabled.0,
                    ..Default::default()
                },
                main: main.0,
                rel_template_id: rel_template_id.0,
                tags: tag.0.map(|tag| vec![tag]),
                status: status.0,
                kinds: kind.0.map(|s| vec![s]),
                ..Default::default()
            },
            page_number.0,
            page_size.0,
            desc_by_create.0,
            desc_by_update.0,
            &funs,
            &ctx.0,
        )
        .await?;
        task_handler_helper::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find the specified main models, or create it if it doesn't exist.
    ///
    /// 查找关联的主流程model。
    ///
    /// # Parameters
    /// - `temp_id` - associated template_id
    /// - `is_shared` - whether the associated template is shared
    #[oai(path = "/find_rel_models", method = "put")]
    async fn find_rel_models(&self, temp_id: Query<Option<String>>, tag_ids: Query<Option<String>>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<HashMap<String, FlowModelSummaryResp>> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        let result = FlowModelServ::find_rel_model_map(temp_id.0, tag_ids.0.map(|s| s.split(",").map(|tag| tag.to_string()).collect_vec()), true, &funs, &ctx.0).await?;
        funs.commit().await?;
        task_handler_helper::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Delete Model By Model Id
    ///
    /// 删除模型
    ///
    /// Valid only when model is not used
    ///
    /// 仅在模型没被使用时有效
    #[oai(path = "/:flow_model_id", method = "delete")]
    async fn delete(&self, flow_model_id: Path<String>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<Void> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        FlowModelServ::delete_item(&flow_model_id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        task_handler_helper::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Bind State By Model Id [Deprecated]
    ///
    /// 绑定状态 [已废弃]
    #[oai(path = "/:flow_model_id/bind_state", method = "post")]
    async fn bind_state(&self, flow_model_id: Path<String>, req: Json<FlowModelBindStateReq>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<Void> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        FlowModelServ::modify_model(
            &flow_model_id.0,
            &mut FlowModelModifyReq {
                modify_version: Some(FlowModelVersionModifyReq {
                    bind_states: Some(vec![FlowModelVersionBindState {
                        exist_state: Some(req.0),
                        ..Default::default()
                    }]),
                    ..Default::default()
                }),
                ..Default::default()
            },
            &funs,
            &ctx.0,
        )
        .await?;
        funs.commit().await?;
        task_handler_helper::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Unbind State By Model Id
    ///
    /// 解绑状态
    #[oai(path = "/:flow_model_id/unbind_state", method = "post")]
    async fn unbind_state(&self, flow_model_id: Path<String>, req: Json<FlowModelUnbindStateReq>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<Void> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        FlowModelServ::unbind_state(&flow_model_id.0, &req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        task_handler_helper::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Resort states [Deprecated]
    ///
    /// 状态重新排序 [已废弃]
    #[oai(path = "/:flow_model_id/resort_state", method = "post")]
    async fn resort_state(&self, flow_model_id: Path<String>, req: Json<FlowModelSortStatesReq>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<Void> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        FlowModelServ::modify_model(
            &flow_model_id.0,
            &mut FlowModelModifyReq {
                modify_version: Some(FlowModelVersionModifyReq {
                    modify_states: Some(
                        req.0
                            .sort_states
                            .into_iter()
                            .map(|state| FlowModelVersionModifyState {
                                id: Some(state.state_id.clone()),
                                modify_rel: Some(FlowStateRelModelModifyReq {
                                    id: state.state_id,
                                    sort: Some(state.sort),
                                    show_btns: None,
                                    is_edit: None,
                                }),
                                modify_state: None,
                                add_transitions: None,
                                modify_transitions: None,
                                delete_transitions: None,
                            })
                            .collect_vec(),
                    ),
                    ..Default::default()
                }),
                ..Default::default()
            },
            &funs,
            &ctx.0,
        )
        .await?;
        funs.commit().await?;
        task_handler_helper::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Resort transitions [Deprecated]
    ///
    /// 动作重新排序 [已废弃]
    #[oai(path = "/:flow_model_id/resort_transition", method = "post")]
    async fn resort_transition(
        &self,
        flow_model_id: Path<String>,
        req: Json<FlowTransitionSortStatesReq>,
        ctx: TardisContextExtractor,
        _request: &Request,
    ) -> TardisApiResult<Void> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        funs.begin().await?;
        FlowModelServ::resort_transition(&flow_model_id.0, &req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        task_handler_helper::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// find rel states by model_id
    ///
    /// 获取关联状态
    #[oai(path = "/find_rel_status", method = "get")]
    async fn find_rel_states(
        &self,
        tag: Query<String>,
        rel_template_id: Query<Option<String>>,
        ctx: TardisContextExtractor,
        _request: &Request,
    ) -> TardisApiResult<Vec<FlowModelFindRelStateResp>> {
        let funs = flow_constants::get_tardis_inst();
        let result = FlowModelServ::find_rel_states(tag.0.split(',').collect(), rel_template_id.0, &funs, &ctx.0).await?;
        task_handler_helper::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// modify related state [Deprecated]
    ///
    /// 编辑关联的状态 [已废弃]
    #[oai(path = "/:flow_model_id/modify_rel_state", method = "patch")]
    async fn modify_rel_state(&self, flow_model_id: Path<String>, req: Json<FlowStateRelModelModifyReq>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<Void> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        FlowModelServ::modify_model(
            &flow_model_id.0,
            &mut FlowModelModifyReq {
                modify_version: Some(FlowModelVersionModifyReq {
                    modify_states: Some(vec![FlowModelVersionModifyState {
                        id: Some(req.0.id.clone()),
                        modify_rel: Some(req.0),
                        modify_state: None,
                        add_transitions: None,
                        modify_transitions: None,
                        delete_transitions: None,
                    }]),
                    ..Default::default()
                }),
                ..Default::default()
            },
            &funs,
            &ctx.0,
        )
        .await?;
        funs.commit().await?;
        task_handler_helper::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Get the operations associated with the model
    ///
    /// 获取模型关联的操作
    #[oai(path = "/:flow_model_id/get_transitions", method = "get")]
    async fn get_rel_transitions(&self, flow_model_id: Path<String>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<Vec<FlowTransitionDetailResp>> {
        let funs = flow_constants::get_tardis_inst();
        let result = FlowModelServ::get_rel_transitions(&flow_model_id.0, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }

    /// Get all templates by template id, if the template is not found, then new template according to the default template.
    ///
    /// 获取当前模板id下的所有模板，若不存在则按默认模板新建。
    #[oai(path = "/find_or_create", method = "post")]
    async fn find_or_create(&self, req: Json<FlowModelFIndOrCreatReq>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<HashMap<String, FlowModelSummaryResp>> {
        let funs = flow_constants::get_tardis_inst();
        let result = FlowModelServ::find_or_create(&req.0, &funs, &ctx.0).await?;
        task_handler_helper::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
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

    /// Creating or referencing models
    ///
    ///
    #[oai(path = "/copy_or_reference_model", method = "post")]
    async fn copy_or_reference_model(
        &self,
        req: Json<FlowModelCopyOrReferenceCiReq>,
        ctx: TardisContextExtractor,
        _request: &Request,
    ) -> TardisApiResult<HashMap<String, String>> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        let orginal_models = FlowModelServ::find_rel_model_map(req.0.target_template_id.clone(), None, true, &funs, &ctx.0).await?;
        // find rel models
        let rel_models = FlowModelServ::find_items(
            &FlowModelFilterReq {
                basic: RbumBasicFilterReq {
                    enabled: Some(true),
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                rel: Some(RbumItemRelFilterReq {
                    optional: false,
                    rel_by_from: true,
                    tag: Some(FlowRelKind::FlowModelTemplate.to_string()),
                    from_rbum_kind: Some(RbumRelFromKind::Item),
                    rel_item_id: Some(req.0.rel_template_id.clone().unwrap_or_default()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            None,
            None,
            &funs,
            &ctx.0,
        )
        .await?;
        let mut result = HashMap::new();
        // 需要新增和替换的主模型
        let rel_main_models = rel_models.iter().filter(|model| model.main).cloned().collect_vec();
        let rel_main_tags = rel_main_models.iter().map(|m| m.tag.clone()).collect_vec();
        for rel_main_model in rel_main_models {
            let update_states = req.update_states.as_ref().map(|update_states| update_states.get(&rel_main_model.tag).cloned().unwrap_or_default());
            let new_model = FlowModelServ::copy_or_reference_main_model(&rel_main_model.id, &req.0.op, if req.0.target_template_id.is_none() { FlowModelKind::AsModel } else { FlowModelKind::AsTemplateAndAsModel }, req.0.target_template_id.clone(), &update_states, None, &funs, &ctx.0).await?;
            result.insert(rel_main_model.id.clone(), new_model.id.clone());
        }
        // 需要删除的主模型
        let delete_main_models = orginal_models.iter().filter(|(orginal_tag, _)| !rel_main_tags.contains(*orginal_tag)).map(|(_, orginal_model)| orginal_model).cloned().collect_vec();
        for delete_main_model in delete_main_models {
            FlowModelServ::delete_item(&delete_main_model.id, &funs, &ctx.0).await?;
        }
        let rel_non_main_models = rel_models.iter().filter(|model| !model.main).cloned().collect::<Vec<_>>();
        for rel_non_main_model in rel_non_main_models {
            let _ = FlowModelServ::copy_or_reference_non_main_model(&rel_non_main_model.id, &req.0.op, if req.0.target_template_id.is_none() { FlowModelKind::AsModel } else { FlowModelKind::AsTemplateAndAsModel }, req.0.target_template_id.clone(), None, &funs, &ctx.0).await?;
        }
        funs.commit().await?;
        task_handler_helper::execute_async_task(&ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }
}
