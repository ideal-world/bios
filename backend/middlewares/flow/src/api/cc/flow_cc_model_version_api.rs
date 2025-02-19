use crate::dto::flow_model_version_dto::{FlowModelVersionAddReq, FlowModelVersionDetailResp, FlowModelVersionFilterReq, FlowModelVersionModifyReq, FlowModelVesionState};
use crate::flow_constants;
use crate::serv::flow_model_version_serv::FlowModelVersionServ;
use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

#[derive(Clone)]
pub struct FlowCcModelVersionApi;

/// Flow model process API
#[poem_openapi::OpenApi(prefix_path = "/cc/model_version")]
impl FlowCcModelVersionApi {
    /// Add Model Version
    ///
    /// 添加模型版本
    #[oai(path = "/", method = "post")]
    async fn add(&self, mut add_req: Json<FlowModelVersionAddReq>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<FlowModelVersionDetailResp> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        let version_id = FlowModelVersionServ::add_item(&mut add_req.0, &funs, &ctx.0).await?;
        let result = FlowModelVersionServ::get_item(&version_id, &FlowModelVersionFilterReq::default(), &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Modify Model Version
    ///
    /// 修改模型版本
    #[oai(path = "/:flow_version_id", method = "patch")]
    async fn modify(
        &self,
        flow_version_id: Path<String>,
        mut modify_req: Json<FlowModelVersionModifyReq>,
        ctx: TardisContextExtractor,
        _request: &Request,
    ) -> TardisApiResult<Void> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        FlowModelVersionServ::modify_item(&flow_version_id.0, &mut modify_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void)
    }

    /// Get Model By Model Id
    ///
    /// 获取模型
    #[oai(path = "/:flow_version_id", method = "get")]
    async fn get(&self, flow_version_id: Path<String>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<FlowModelVersionDetailResp> {
        let funs = flow_constants::get_tardis_inst();
        let result = FlowModelVersionServ::get_item(&flow_version_id.0, &FlowModelVersionFilterReq::default(), &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Get Model By Model Id
    ///
    /// 获取模型使用全局owner
    #[oai(path = "/:flow_version_id/global", method = "get")]
    async fn global_get(&self, flow_version_id: Path<String>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<FlowModelVersionDetailResp> {
        let funs = flow_constants::get_tardis_inst();
        let result = FlowModelVersionServ::get_item(
            &flow_version_id.0,
            &FlowModelVersionFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            &funs,
            &ctx.0,
        )
        .await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Models
    ///
    /// 获取模型列表
    #[oai(path = "/", method = "get")]
    #[allow(clippy::too_many_arguments)]
    async fn paginate(
        &self,
        ids: Query<Option<String>>,
        name: Query<Option<String>>,
        rel_model_id: Query<Option<String>>,
        status: Query<Option<FlowModelVesionState>>,
        enabled: Query<Option<bool>>,
        with_sub: Query<Option<bool>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        desc_by_publish: Query<Option<bool>>,
        ctx: TardisContextExtractor,
        _request: &Request,
    ) -> TardisApiResult<TardisPage<FlowModelVersionDetailResp>> {
        let funs = flow_constants::get_tardis_inst();
        let result = FlowModelVersionServ::paginate_detail_items(
            &FlowModelVersionFilterReq {
                basic: RbumBasicFilterReq {
                    ids: ids.0.map(|ids| ids.split(',').map(|id| id.to_string()).collect::<Vec<String>>()),
                    name: name.0,
                    with_sub_own_paths: with_sub.0.unwrap_or(false),
                    enabled: enabled.0,
                    ..Default::default()
                },
                rel_model_ids: rel_model_id.0.map(|rel_model_id| vec![rel_model_id]),
                status: Some(status.0.map_or(vec![FlowModelVesionState::Enabled, FlowModelVesionState::Disabled], |status| vec![status])),
                desc_by_publish: desc_by_publish.0,
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

    /// Creating the version being edited
    ///
    /// 创建正在编辑的版本
    #[oai(path = "/:flow_version_id/create_editing_version", method = "post")]
    async fn create_editing_version(&self, flow_version_id: Path<String>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<FlowModelVersionDetailResp> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        let result = FlowModelVersionServ::create_editing_version(&flow_version_id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }
}
