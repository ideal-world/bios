use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::helper::rbum_scope_helper::check_without_owner_and_unsafe_fill_ctx;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use crate::dto::flow_model_dto::{FlowModelAddCustomModelReq, FlowModelAddCustomModelResp, FlowModelAggResp, FlowModelFilterReq, FlowModelFindRelStateResp};
use crate::flow_constants;
use crate::serv::flow_model_serv::FlowModelServ;
use crate::serv::flow_rel_serv::FlowRelServ;
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
}
