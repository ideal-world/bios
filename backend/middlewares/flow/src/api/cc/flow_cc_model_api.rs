use std::collections::HashMap;

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumRelFilterReq};
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_basic::rbum::serv::rbum_rel_serv::RbumRelServ;
use itertools::Itertools;
use tardis::basic::dto::TardisContext;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use crate::dto::flow_model_dto::{
    FlowModelAddCustomModelReq, FlowModelAddCustomModelResp, FlowModelAddReq, FlowModelAggResp, FlowModelBindStateReq, FlowModelFilterReq, FlowModelFindRelStateResp,
    FlowModelModifyReq, FlowModelSortStatesReq, FlowModelSummaryResp, FlowModelUnbindStateReq,
};
use crate::dto::flow_state_dto::FlowStateRelModelModifyReq;
use crate::dto::flow_transition_dto::{FlowTransitionModifyReq, FlowTransitionSortStatesReq};
use crate::flow_constants;
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
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Get Model By Model Id
    ///
    /// 获取模型
    #[oai(path = "/:flow_model_id", method = "get")]
    async fn get(&self, flow_model_id: Path<String>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<FlowModelAggResp> {
        let funs = flow_constants::get_tardis_inst();
        let result = FlowModelServ::get_item_detail_aggs(&flow_model_id.0, true, &funs, &ctx.0).await?;
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
        with_sub: Query<Option<bool>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
        _request: &Request,
    ) -> TardisApiResult<TardisPage<FlowModelSummaryResp>> {
        let funs = flow_constants::get_tardis_inst();
        let result = FlowModelServ::paginate_items(
            &FlowModelFilterReq {
                basic: RbumBasicFilterReq {
                    ids: flow_model_ids.0.map(|ids| ids.split(',').map(|id| id.to_string()).collect::<Vec<String>>()),
                    name: name.0,
                    with_sub_own_paths: with_sub.0.unwrap_or(false),
                    enabled: enabled.0,
                    ..Default::default()
                },
                tags: tag.0.map(|tag| vec![tag]),
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
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find the specified models, or create it if it doesn't exist.
    ///
    /// 查找关联的model，如果不存在则创建。创建规则遵循add_custom_model接口逻辑。
    ///
    /// # Parameters
    /// - `tag_ids` - list of tag_id
    /// - `temp_id` - associated template_id
    /// - `is_shared` - whether the associated template is shared
    #[oai(path = "/find_or_add_models", method = "put")]
    async fn find_or_add_models(
        &self,
        tag_ids: Query<String>,
        temp_id: Query<Option<String>>,
        is_shared: Query<Option<bool>>,
        ctx: TardisContextExtractor,
        _request: &Request,
    ) -> TardisApiResult<HashMap<String, FlowModelSummaryResp>> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        let tag_ids = tag_ids.split(',').map(|tag_id| tag_id.to_string()).collect_vec();
        let result = FlowModelServ::find_or_add_models(tag_ids, temp_id.0, is_shared.unwrap_or(false), &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find the specified models, or create it if it doesn't exist.
    ///
    /// 查找关联的model。
    ///
    /// # Parameters
    /// - `temp_id` - associated template_id
    /// - `is_shared` - whether the associated template is shared
    #[oai(path = "/find_rel_models", method = "put")]
    async fn find_rel_models(
        &self,
        temp_id: Query<Option<String>>,
        is_shared: Query<Option<bool>>,
        ctx: TardisContextExtractor,
        _request: &Request,
    ) -> TardisApiResult<HashMap<String, FlowModelSummaryResp>> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        let result = FlowModelServ::find_rel_models(temp_id.0, is_shared.unwrap_or(false), &funs, &ctx.0).await?;
        funs.commit().await?;
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
                bind_states: Some(vec![req.0]),
                ..Default::default()
            },
            &funs,
            &ctx.0,
        )
        .await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Unbind State By Model Id [Deprecated]
    ///
    /// 解绑状态 [已废弃]
    #[oai(path = "/:flow_model_id/unbind_state", method = "post")]
    async fn unbind_state(&self, flow_model_id: Path<String>, req: Json<FlowModelUnbindStateReq>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<Void> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        FlowModelServ::modify_model(
            &flow_model_id.0,
            &mut FlowModelModifyReq {
                unbind_states: Some(vec![req.state_id.clone()]),
                ..Default::default()
            },
            &funs,
            &ctx.0,
        )
        .await?;
        funs.commit().await?;
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
                modify_states: Some(
                    req.0
                        .sort_states
                        .into_iter()
                        .map(|state| FlowStateRelModelModifyReq {
                            id: state.state_id,
                            sort: Some(state.sort),
                            show_btns: None,
                        })
                        .collect_vec(),
                ),
                ..Default::default()
            },
            &funs,
            &ctx.0,
        )
        .await?;
        funs.commit().await?;
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
        let modify_trans = req
            .0
            .sort_states
            .into_iter()
            .map(|sort_req| FlowTransitionModifyReq {
                id: sort_req.id.clone().into(),
                sort: Some(sort_req.sort),
                ..Default::default()
            })
            .collect_vec();
        FlowModelServ::modify_model(
            &flow_model_id.0,
            &mut FlowModelModifyReq {
                modify_transitions: Some(modify_trans),
                ..Default::default()
            },
            &funs,
            &ctx.0,
        )
        .await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// copy parent model to current own_paths
    ///
    /// 复制父级模型到当前 own_paths
    /// 实际创建规则：按照 tags 创建模型，若传入proj_template_id，则优先寻找对应的父级模型，否则则获取默认模板模型生成对应的自定义模型。
    #[oai(path = "/add_custom_model", method = "post")]
    async fn add_custom_model(&self, req: Json<FlowModelAddCustomModelReq>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<Vec<FlowModelAddCustomModelResp>> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        let mut result = vec![];
        for item in &req.0.bind_model_objs {
            let model_id = FlowModelServ::add_custom_model(&item.tag, req.0.proj_template_id.clone(), req.0.rel_template_id.clone(), &funs, &ctx.0).await.ok();
            result.push(FlowModelAddCustomModelResp { tag: item.tag.clone(), model_id });
        }
        funs.commit().await?;
        TardisResp::ok(result)
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
                modify_states: Some(vec![req.0]),
                ..Default::default()
            },
            &funs,
            &ctx.0,
        )
        .await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// batch add rels with template and app
    ///
    /// 批量添加模板和应用的关联关系
    #[oai(path = "/batch_add_template_app_rels", method = "get")]
    async fn batch_add_template_app_rels(&self, _request: &Request) -> TardisApiResult<Void> {
        let mut funs = flow_constants::get_tardis_inst();
        let global_ctx = TardisContext::default();
        funs.begin().await?;
        let rels = RbumRelServ::find_rels(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                tag: Some("FlowModelPath".to_string()),
                ..Default::default()
            },
            None,
            None,
            &funs,
            &global_ctx,
        )
        .await?;
        for rel in rels {
            let ctx = TardisContext {
                own_paths: rel.rel.own_paths,
                owner: rel.rel.owner,
                ..Default::default()
            };
            let rel_model_id = rel.rel.from_rbum_id;
            if let Some(template_id) =
                FlowRelServ::find_from_simple_rels(&FlowRelKind::FlowModelTemplate, &rel_model_id, None, None, &funs, &ctx).await?.pop().map(|rel| rel.rel_id)
            {
                FlowRelServ::add_simple_rel(
                    &FlowRelKind::FlowAppTemplate,
                    &rel.rel.to_rbum_item_id.split('/').collect::<Vec<&str>>().last().map(|s| s.to_string()).unwrap_or_default(),
                    &template_id,
                    None,
                    None,
                    true,
                    true,
                    None,
                    &funs,
                    &ctx,
                )
                .await?;
            }
        }
        funs.commit().await?;
        TardisResp::ok(Void)
    }
}
