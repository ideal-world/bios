use std::collections::HashMap;

use crate::dto::flow_model_dto::{
    FlowModelAggResp, FlowModelAssociativeOperationKind, FlowModelCopyOrReferenceCiReq, FlowModelExistRelByTemplateIdsReq, FlowModelFilterReq, FlowModelFindRelStateResp,
    FlowModelKind,
};
use crate::flow_constants;
use crate::serv::flow_inst_serv::FlowInstServ;
use crate::serv::flow_model_serv::FlowModelServ;
use crate::serv::flow_rel_serv::{FlowRelKind, FlowRelServ};
use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumRelFilterReq};
use bios_basic::rbum::helper::rbum_scope_helper::check_without_owner_and_unsafe_fill_ctx;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_basic::rbum::serv::rbum_rel_serv::RbumRelServ;
use itertools::Itertools;
use std::iter::Iterator;
use tardis::basic::dto::TardisContext;
use tardis::log::warn;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};
#[derive(Clone)]
pub struct FlowCiModelApi;

/// Flow Config process API
#[poem_openapi::OpenApi(prefix_path = "/ci/model")]
impl FlowCiModelApi {
    /// Get model detail
    ///
    /// 获取模型详情
    #[oai(path = "/detail", method = "get")]
    async fn get_detail(
        &self,
        id: Query<Option<String>>,
        tag: Query<Option<String>>,
        rel_template_id: Query<Option<String>>,
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<FlowModelAggResp> {
        let funs = flow_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        let model_id = FlowModelServ::find_one_item(
            &FlowModelFilterReq {
                basic: RbumBasicFilterReq {
                    ids: id.0.map(|id| vec![id]),
                    ..Default::default()
                },
                tags: tag.0.map(|tag| vec![tag]),
                rel: FlowRelServ::get_template_rel_filter(rel_template_id.0.as_deref()),
                ..Default::default()
            },
            &funs,
            &ctx.0,
        )
        .await?
        .ok_or_else(|| funs.err().internal_error("flow_ci_model_api", "get_detail", "model is not exist", "404-flow-model-not-found"))?
        .id;
        let result = FlowModelServ::get_item_detail_aggs(&model_id, true, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
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
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<FlowModelFindRelStateResp>> {
        let funs = flow_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        let result = FlowModelServ::find_rel_states(tag.0.split(',').collect(), rel_template_id.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// add custom model by template_id
    ///
    /// 添加自定义模型
    // #[oai(path = "/add_custom_model", method = "post")]
    // async fn add_custom_model(
    //     &self,
    //     req: Json<FlowModelAddCustomModelReq>,
    //     mut ctx: TardisContextExtractor,
    //     request: &Request,
    // ) -> TardisApiResult<Vec<FlowModelAddCustomModelResp>> {
    //     let mut funs = flow_constants::get_tardis_inst();
    //     check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
    //     funs.begin().await?;
    //     let proj_template_id = req.0.proj_template_id;
    //     let mut result = vec![];
    //     for item in req.0.bind_model_objs {
    //         let model_id = FlowModelServ::add_custom_model(&item.tag, proj_template_id.clone(), None, &funs, &ctx.0).await.ok();
    //         result.push(FlowModelAddCustomModelResp { tag: item.tag, model_id });
    //     }
    //     funs.commit().await?;
    //     ctx.0.execute_task().await?;
    //     TardisResp::ok(result)
    // }

    /// Creating or referencing models
    ///
    ///
    #[oai(path = "/copy_or_reference_model", method = "post")]
    async fn copy_or_reference_model(
        &self,
        req: Json<FlowModelCopyOrReferenceCiReq>,
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<HashMap<String, String>> {
        let mut funs = flow_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        funs.begin().await?;
        warn!("ci copy_or_reference_model req: {:?}", req.0);
        let _orginal_models = FlowModelServ::clean_rel_models(None, None, None, &funs, &ctx.0).await?;
        // find rel models
        let rel_model_ids = FlowRelServ::find_to_simple_rels(
            &FlowRelKind::FlowModelTemplate,
            &req.0.rel_template_id.clone().unwrap_or_default(),
            None,
            None,
            &funs,
            &ctx.0,
        )
        .await?
        .into_iter()
        .map(|rel| rel.rel_id)
        .collect_vec();
        let rel_models = FlowModelServ::find_items(
            &FlowModelFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(rel_model_ids),
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                main: Some(true),
                ..Default::default()
            },
            None,
            None,
            &funs,
            &ctx.0,
        )
        .await?;
        let mut result = HashMap::new();
        for rel_model in rel_models {
            let new_model = FlowModelServ::copy_or_reference_model(&rel_model.id, &req.0.op, FlowModelKind::AsModel, &funs, &ctx.0).await?;
            FlowInstServ::batch_update_when_switch_model(
                new_model.rel_template_ids.first().cloned(),
                &new_model.tag,
                &new_model.current_version_id,
                new_model.states.clone(),
                &new_model.init_state_id,
                &funs,
                &ctx.0,
            )
            .await?;
            result.insert(rel_model.id.clone(), new_model.id.clone());
        }
        funs.commit().await?;
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
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<HashMap<String, FlowModelAggResp>> {
        let mut funs = flow_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        funs.begin().await?;
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
            let added_model = FlowModelServ::copy_or_reference_model(
                &from_model.rel_model_id,
                &FlowModelAssociativeOperationKind::ReferenceOrCopy,
                FlowModelKind::AsTemplateAndAsModel,
                &funs,
                &ctx.0,
            )
            .await?;
            FlowRelServ::add_simple_rel(
                &FlowRelKind::FlowModelTemplate,
                &added_model.id,
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
            result.insert(from_model.rel_model_id.clone(), added_model);
        }
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// batch delete models by rel_template_id
    ///
    /// 通过关联模板ID删除模型
    #[oai(path = "/delete_by_rel_template_id/:rel_template_id", method = "delete")]
    async fn delete_by_rel_template_id(&self, rel_template_id: Path<String>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let mut funs = flow_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        funs.begin().await?;
        for rel in FlowRelServ::find_to_simple_rels(&FlowRelKind::FlowModelTemplate, &rel_template_id.0, None, None, &funs, &ctx.0).await? {
            FlowModelServ::delete_item(&rel.rel_id, &funs, &ctx.0).await?;
        }
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void)
    }

    /// Check if there is an association by template ID, return the associated template ID
    ///
    /// 通过模板ID检查是否存在关联，返回关联的模板ID
    #[oai(path = "/exist_rel_by_template_ids", method = "post")]
    async fn exist_rel_by_template_ids(&self, req: Json<FlowModelExistRelByTemplateIdsReq>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Vec<String>> {
        let funs = flow_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        warn!("ci exist_rel_by_template_ids req: {:?}", req.0);
        let support_tags = req.0.support_tags;
        let mut result = vec![];
        for (rel_template_id, current_tags) in req.0.rel_tag_by_template_ids {
            // 当前模板tag和需要支持的tag取交集，得到当前模板tag中需要检查的tag列表
            let tags = current_tags.into_iter().filter(|current_tag| support_tags.contains(current_tag)).collect_vec();
            if !tags.is_empty() {
                // 当前模板关联的模型所支持的tag
                let rel_model_tags = FlowModelServ::find_items(
                    &FlowModelFilterReq {
                        basic: RbumBasicFilterReq {
                            ids: Some(
                                FlowRelServ::find_to_simple_rels(&FlowRelKind::FlowModelTemplate, &rel_template_id, None, None, &funs, &ctx.0)
                                    .await?
                                    .into_iter()
                                    .map(|rel| rel.rel_id)
                                    .collect_vec(),
                            ),
                            own_paths: Some("".to_string()),
                            with_sub_own_paths: true,
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
                .into_iter()
                .map(|model| model.tag.clone())
                .collect_vec();
                // 如果出现了当前模板tag中需要检查的tag没有被当前模板关联，则说明当前关联模板不是可用状态
                if !tags.into_iter().filter(|tag| !rel_model_tags.contains(tag)).collect_vec().is_empty() {
                    continue;
                }
            }
            result.push(rel_template_id.clone());
        }

        TardisResp::ok(result)
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
