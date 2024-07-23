use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::helper::rbum_scope_helper::{self, check_without_owner_and_unsafe_fill_ctx};
use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use tardis::basic::dto::TardisContext;
use tardis::log;
use tardis::tokio;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use crate::dto::flow_state_dto::{FlowStateCountGroupByStateReq, FlowStateCountGroupByStateResp, FlowStateFilterReq, FlowStateKind, FlowStateSummaryResp, FlowSysStateKind};
use crate::flow_constants;
use crate::serv::flow_state_serv::FlowStateServ;
#[derive(Clone)]
pub struct FlowCiStateApi;

/// Flow Config process API
#[poem_openapi::OpenApi(prefix_path = "/ci/state")]
impl FlowCiStateApi {
    /// Find States
    ///
    /// 获取状态列表
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
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<TardisPage<FlowStateSummaryResp>> {
        let funs = flow_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;

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
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Count Group By State
    ///
    /// 按状态分组统计
    #[oai(path = "/count_group_by_state", method = "post")]
    async fn count_group_by_state(
        &self,
        req: Json<FlowStateCountGroupByStateReq>,
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<FlowStateCountGroupByStateResp>> {
        let mut funs = flow_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        funs.begin().await?;
        let result = FlowStateServ::count_group_by_state(&req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    ///Script: merge global states with the same name
    ///
    /// 脚本：合并相同名称的全局状态
    #[oai(path = "/merge_state_by_name", method = "post")]
    async fn merge_state_by_name(&self) -> TardisApiResult<Void> {
        let funs = flow_constants::get_tardis_inst();
        let global_ctx = TardisContext::default();
        tokio::spawn(async move {
            match FlowStateServ::merge_state_by_name(&funs, &global_ctx).await {
                Ok(_) => {
                    log::trace!("[Flow.Inst] add log success")
                }
                Err(e) => {
                    log::warn!("[Flow.Inst] failed to add log:{e}")
                }
            }
        });
        TardisResp::ok(Void {})
    }
}
