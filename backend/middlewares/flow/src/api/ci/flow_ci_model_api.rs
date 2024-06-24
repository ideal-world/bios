use std::collections::HashMap;

use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::helper::rbum_scope_helper::{self, check_without_owner_and_unsafe_fill_ctx};
use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use itertools::Itertools;
use tardis::basic::dto::TardisContext;
use tardis::log::warn;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use crate::dto::flow_model_dto::{
    FlowModelAddCustomModelReq, FlowModelAddCustomModelResp, FlowModelAggResp, FlowModelCopyOrReferenceCiReq, FlowModelFilterReq, FlowModelFindRelStateResp,
};
use crate::flow_constants;
use crate::serv::flow_model_serv::FlowModelServ;
use crate::serv::flow_rel_serv::{FlowRelKind, FlowRelServ};
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
        let result = FlowModelServ::get_item_detail_aggs(&model_id, &funs, &ctx.0).await?;
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
    /// 创建或引用模型（rel_model_id：关联模型ID, op：关联模型操作类型（复制或者引用），is_create_copy：是否创建副本（当op为复制时需指定，默认不需要））
    #[oai(path = "/copy_or_reference_model", method = "post")]
    async fn copy_or_reference_model(&self, req: Json<FlowModelCopyOrReferenceCiReq>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<HashMap<String, String>> {
        let mut funs = flow_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        warn!("ctx:{:?}", ctx.0);
        funs.begin().await?;
        // find rel models
        let rel_model_ids = FlowRelServ::find_to_simple_rels(&FlowRelKind::FlowModelTemplate, &req.0.rel_template_id.unwrap_or_default(), None, None, &funs, &ctx.0)
            .await?
            .into_iter()
            .map(|rel| rel.rel_id)
            .collect_vec();
        let mut result = HashMap::new();
        let mock_ctx = TardisContext {
            own_paths: rbum_scope_helper::get_path_item(RbumScopeLevelKind::L1.to_int(), &ctx.0.own_paths).unwrap_or_default(),
            ..ctx.0.clone()
        };
        for rel_model_id in rel_model_ids {
            let added_model = FlowModelServ::copy_or_reference_model(None, &rel_model_id, Some(ctx.0.own_paths.clone()), &req.0.op, Some(false), &funs, &mock_ctx).await?;
            result.insert(rel_model_id.clone(), added_model.id.clone());
        }
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }
}
