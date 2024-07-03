use std::collections::HashMap;

use crate::dto::flow_model_dto::{
    FlowModelAddCustomModelReq, FlowModelAddCustomModelResp, FlowModelAggResp, FlowModelAssociativeOperationKind, FlowModelCopyOrReferenceCiReq, FlowModelExistRelByTemplateIdsReq,
    FlowModelFilterReq, FlowModelFindRelStateResp,
};
use crate::flow_constants;
use crate::serv::flow_model_serv::FlowModelServ;
use crate::serv::flow_rel_serv::{FlowRelKind, FlowRelServ};
use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::helper::rbum_scope_helper::{self, check_without_owner_and_unsafe_fill_ctx};
use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use itertools::Itertools;
use tardis::log::warn;
use std::iter::Iterator;
use tardis::basic::dto::TardisContext;
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
    #[oai(path = "/add_custom_model", method = "post")]
    async fn add_custom_model(
        &self,
        req: Json<FlowModelAddCustomModelReq>,
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<FlowModelAddCustomModelResp>> {
        let mut funs = flow_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        funs.begin().await?;
        let proj_template_id = req.0.proj_template_id;
        let mut result = vec![];
        for item in req.0.bind_model_objs {
            let model_id = FlowModelServ::add_custom_model(&item.tag, proj_template_id.clone(), None, &funs, &ctx.0).await.ok();
            result.push(FlowModelAddCustomModelResp { tag: item.tag, model_id });
        }
        funs.commit().await?;
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
        FlowModelServ::clean_rel_models(None, None, &funs, &ctx.0).await?;
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
        let mut result = HashMap::new();
        let mut mock_ctx = ctx.0.clone();
        if rbum_scope_helper::get_scope_level_by_context(&ctx.0)? == RbumScopeLevelKind::L2 {
            mock_ctx = match req.0.op {
                FlowModelAssociativeOperationKind::Copy => ctx.0.clone(),
                FlowModelAssociativeOperationKind::Reference => TardisContext {
                    own_paths: rbum_scope_helper::get_path_item(RbumScopeLevelKind::L1.to_int(), &ctx.0.own_paths).unwrap_or_default(),
                    ..ctx.0.clone()
                },
            };
        }
        for rel_model_id in rel_model_ids {
            result.insert(
                rel_model_id.clone(),
                FlowModelServ::copy_or_reference_model(&rel_model_id, Some(ctx.0.own_paths.clone()), &req.0.op, Some(false), &funs, &mock_ctx).await?.id,
            );
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
            let added_model =
                FlowModelServ::copy_or_reference_model(&from_model.rel_model_id, None, &FlowModelAssociativeOperationKind::Copy, Some(true), &funs, &ctx.0).await?;
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
        let mut result = vec![];
        for rel_template_id in req.0.rel_template_ids {
            if !FlowRelServ::find_to_simple_rels(&FlowRelKind::FlowModelTemplate, &rel_template_id, None, None, &funs, &ctx.0).await?.is_empty() {
                result.push(rel_template_id.clone());
            }
        }

        TardisResp::ok(result)
    }
}
