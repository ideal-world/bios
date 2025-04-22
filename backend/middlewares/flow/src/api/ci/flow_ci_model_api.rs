use std::collections::HashMap;

use crate::dto::flow_model_dto::{
    FlowModelAggResp, FlowModelCopyOrReferenceCiReq, FlowModelExistRelByTemplateIdsReq, FlowModelFilterReq, FlowModelFindRelStateResp, FlowModelKind, FlowModelSyncModifiedFieldReq,
};
use crate::flow_constants;
use crate::serv::flow_inst_serv::FlowInstServ;
use crate::serv::flow_model_serv::FlowModelServ;
use crate::serv::flow_rel_serv::{FlowRelKind, FlowRelServ};
use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemRelFilterReq};
use bios_basic::rbum::helper::rbum_scope_helper::check_without_owner_and_unsafe_fill_ctx;
use bios_basic::rbum::rbum_enumeration::RbumRelFromKind;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use itertools::Itertools;
use std::iter::Iterator;
use tardis::futures::future::join_all;
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
        let rel_main_models = FlowModelServ::find_items(
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
        for rel_main_model in rel_main_models {
            let new_model = FlowModelServ::copy_or_reference_model(&rel_main_model.id, &req.0.op, FlowModelKind::AsModel, &funs, &ctx.0).await?;
            FlowInstServ::batch_update_when_switch_model(
                &new_model,
                new_model.rel_template_ids.first().cloned(),
                req.update_states.clone().map(|update_states| update_states.get(&new_model.tag).cloned().unwrap_or_default()),
                &funs,
                &ctx.0,
            )
            .await?;
            result.insert(rel_main_model.id.clone(), new_model.id.clone());
        }
        let rel_non_main_models = FlowModelServ::find_items(
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
                main: Some(false),
                ..Default::default()
            },
            None,
            None,
            &funs,
            &ctx.0,
        )
        .await?;
        for rel_non_main_model in rel_non_main_models {
            let _ = FlowModelServ::copy_or_reference_model(&rel_non_main_model.id, &req.0.op, FlowModelKind::AsModel, &funs, &ctx.0).await?;
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
        let result = FlowModelServ::copy_models_by_template_id(&from_template_id.0, &to_template_id.0, &funs, &ctx.0).await?;
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
        let result = join_all(
            req.0
                .rel_tag_by_template_ids
                .iter()
                .map(|(rel_template_id, current_tags)| async {
                    // 当前模板tag和需要支持的tag取交集，得到当前模板tag中需要检查的tag列表
                    let tags = current_tags.iter().filter(|current_tag| support_tags.contains(current_tag)).collect_vec();
                    if !tags.is_empty() {
                        // 当前模板关联的模型所支持的tag
                        let rel_model_tags = FlowModelServ::find_items(
                            &FlowModelFilterReq {
                                basic: RbumBasicFilterReq {
                                    own_paths: Some("".to_string()),
                                    with_sub_own_paths: true,
                                    ..Default::default()
                                },
                                rel: Some(RbumItemRelFilterReq {
                                    optional: false,
                                    rel_by_from: true,
                                    tag: Some(FlowRelKind::FlowModelTemplate.to_string()),
                                    from_rbum_kind: Some(RbumRelFromKind::Item),
                                    rel_item_id: Some(rel_template_id.clone()),
                                    ..Default::default()
                                }),
                                main: Some(true),
                                ..Default::default()
                            },
                            None,
                            None,
                            &funs,
                            &ctx.0,
                        )
                        .await
                        .unwrap_or_default()
                        .into_iter()
                        .map(|model| model.tag.clone())
                        .collect_vec();
                        // 如果出现了当前模板tag中需要检查的tag没有被当前模板关联，则说明当前关联模板不是可用状态
                        if !tags.into_iter().filter(|tag| !rel_model_tags.contains(tag)).collect_vec().is_empty() {
                            return None;
                        }
                    }
                    Some(rel_template_id.clone())
                })
                .collect_vec(),
        )
        .await
        .into_iter()
        .filter(|r| r.is_some())
        .map(|r| r.unwrap_or_default())
        .collect_vec();

        TardisResp::ok(result)
    }

    /// Synchronize modified fields
    ///
    /// 同步修改的字段
    #[oai(path = "/sync_modified_field", method = "post")]
    async fn sync_modified_field(&self, modify_req: Json<FlowModelSyncModifiedFieldReq>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let mut funs = flow_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        funs.begin().await?;
        FlowModelServ::sync_modified_field(&modify_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void)
    }

    /// 初始化评审模板（脚本）
    #[oai(path = "/init_review_model", method = "post")]
    async fn init_review_model(&self, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let mut funs = flow_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        funs.begin().await?;
        FlowModelServ::init_review_model(&funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void)
    }

    /// 初始化用例模板（脚本）
    #[oai(path = "/init_tc_model", method = "post")]
    async fn init_tc_model(&self, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let mut funs = flow_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        funs.begin().await?;
        FlowModelServ::init_tc_model(&funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void)
    }
}
