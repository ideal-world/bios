use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::helper::rbum_scope_helper;
use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use crate::dto::flow_state_dto::{
    FlowStateAddReq, FlowStateCountGroupByStateReq, FlowStateCountGroupByStateResp, FlowStateDetailResp, FlowStateFilterReq, FlowStateKind, FlowStateModifyReq, FlowStateNameResp,
    FlowStateSummaryResp, FlowSysStateKind,
};
use crate::flow_constants;
use crate::serv::flow_state_serv::FlowStateServ;
#[derive(Clone)]
pub struct FlowCcStateApi;

/// Flow state process API
#[poem_openapi::OpenApi(prefix_path = "/cc/state")]
impl FlowCcStateApi {
    /// Add State / 添加状态
    #[oai(path = "/", method = "post")]
    async fn add(&self, mut add_req: Json<FlowStateAddReq>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<String> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        let result = FlowStateServ::add_item(&mut add_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(result)
    }

    /// Modify State By State Id / 修改状态
    #[oai(path = "/:id", method = "patch")]
    async fn modify(&self, id: Path<String>, mut modify_req: Json<FlowStateModifyReq>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<Void> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        FlowStateServ::modify_item(&id.0, &mut modify_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Get State By State Id / 获取状态
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<FlowStateDetailResp> {
        let funs = flow_constants::get_tardis_inst();
        let result = FlowStateServ::get_item(
            &id.0,
            &FlowStateFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            &funs,
            &ctx.0,
        )
        .await?;
        TardisResp::ok(result)
    }

    /// Find States / 获取状态列表
    #[oai(path = "/", method = "get")]
    #[allow(clippy::too_many_arguments)]
    async fn paginate(
        &self,
        ids: Query<Option<String>>,
        name: Query<Option<String>>,
        tag: Query<Option<String>>,
        sys_state: Query<Option<FlowSysStateKind>>,
        state_kind: Query<Option<FlowStateKind>>,
        enabled: Query<Option<bool>>,
        template: Query<Option<bool>>,
        with_sub: Query<Option<bool>>,
        is_global: Query<Option<bool>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
        _request: &Request,
    ) -> TardisApiResult<TardisPage<FlowStateSummaryResp>> {
        let funs = flow_constants::get_tardis_inst();

        let (scope_level, with_sub_own_paths) = if let Some(is_global) = is_global.0 {
            if is_global {
                // get global state
                (Some(RbumScopeLevelKind::Root), false)
            } else {
                // get custom state
                (Some(rbum_scope_helper::get_scope_level_by_context(&ctx.0)?), true)
            }
        } else {
            // get all state
            (None, with_sub.0.unwrap_or(false))
        };

        let result = FlowStateServ::paginate_items(
            &FlowStateFilterReq {
                basic: RbumBasicFilterReq {
                    ids: ids.0.map(|ids| ids.split(',').map(|id| id.to_string()).collect::<Vec<String>>()),
                    name: name.0,
                    with_sub_own_paths,
                    enabled: enabled.0,
                    scope_level,
                    ..Default::default()
                },
                tag: tag.0,
                sys_state: sys_state.0,
                state_kind: state_kind.0,
                template: template.0,
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

        TardisResp::ok(result)
    }

    /// Delete State By State Id / 删除状态
    ///
    /// Valid only when state is not used
    ///
    /// 仅在状态没被使用时有效
    #[oai(path = "/:id", method = "delete")]
    async fn delete(&self, id: Path<String>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<Void> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        FlowStateServ::delete_item(&id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Find Names By Ids
    #[oai(path = "/names", method = "get")]
    async fn find_names(
        &self,
        ids: Query<Option<Vec<String>>>,
        tag: Query<Option<String>>,
        app_ids: Query<Option<String>>,
        ctx: TardisContextExtractor,
        _request: &Request,
    ) -> TardisApiResult<Vec<FlowStateNameResp>> {
        let funs = flow_constants::get_tardis_inst();
        let resp = FlowStateServ::find_names(
            ids.0,
            tag.0,
            app_ids.0.map(|ids| ids.split(',').map(|id| id.to_string()).collect::<Vec<String>>()),
            &funs,
            &ctx.0,
        )
        .await?;
        TardisResp::ok(resp)
    }

    /// Count Group By State / 按状态分组统计
    #[oai(path = "/count_group_by_state", method = "post")]
    async fn count_group_by_state(
        &self,
        req: Json<FlowStateCountGroupByStateReq>,
        ctx: TardisContextExtractor,
        _request: &Request,
    ) -> TardisApiResult<Vec<FlowStateCountGroupByStateResp>> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        let result = FlowStateServ::count_group_by_state(&req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(result)
    }
}
