use crate::domain::trigger_scene;
use crate::dto::*;
use bios_basic::rbum::serv::rbum_crud_serv::{RbumCrudOperation, RbumCrudQueryPackage};
use tardis::async_trait::async_trait;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm::sea_query::{Query, SelectStatement};
use tardis::db::sea_orm::*;
use tardis::{TardisFuns, TardisFunsInst};

pub struct ReachTriggerSceneService;

#[async_trait]
impl
    RbumCrudOperation<
        trigger_scene::ActiveModel,
        ReachTriggerSceneAddReq,
        ReachTriggerSceneModifyReq,
        ReachTriggerSceneSummaryResp,
        ReachTriggerSceneDetailResp,
        ReachTriggerSceneFilterReq,
    > for ReachTriggerSceneService
{
    fn get_table_name() -> &'static str {
        trigger_scene::Entity.table_name()
    }
    async fn package_add(add_req: &ReachTriggerSceneAddReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<trigger_scene::ActiveModel> {
        let mut model = trigger_scene::ActiveModel::from(add_req);
        model.id = Set(TardisFuns::field.nanoid());
        model.fill_ctx(ctx, true);
        Ok(model)
    }

    async fn before_add_rbum(add_req: &mut ReachTriggerSceneAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if let Some(pid) = &add_req.pid {
            if !pid.trim().is_empty() {
                return Self::check_ownership(pid, funs, ctx).await;
            }
        }
        add_req.pid = Some(String::default());
        Ok(())
    }

    async fn package_modify(id: &str, modify_req: &ReachTriggerSceneModifyReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<trigger_scene::ActiveModel> {
        let mut model = trigger_scene::ActiveModel::from(modify_req);
        model.fill_ctx(ctx, false);
        model.id = Set(id.into());
        Ok(model)
    }

    async fn package_query(is_detail: bool, filter: &ReachTriggerSceneFilterReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        query.columns(trigger_scene::Column::iter().map(|c| (trigger_scene::Entity, c)));
        query.from(trigger_scene::Entity);

        if let Some(code) = &filter.code {
            query.and_where(trigger_scene::Column::Code.starts_with(code));
        }
        if let Some(name) = &filter.name {
            query.and_where(trigger_scene::Column::Name.eq(name));
        }
        query.with_filter(Self::get_table_name(), &filter.base_filter.basic, is_detail, false, ctx);
        Ok(query)
    }
}

#[allow(dead_code)]
impl ReachTriggerSceneService {
    pub async fn init(tree: &ReachTriggerSceneTree, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let count = funs.db().count(Query::select().from(trigger_scene::Entity)).await?;
        if count > 0 {
            return Ok(());
        }
        tree.add(None, funs, ctx).await?;
        Ok(())
    }
}
