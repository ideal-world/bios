use crate::domain::reach_vcode_strategy;
use crate::dto::*;

use bios_basic::rbum::serv::rbum_crud_serv::{RbumCrudOperation, RbumCrudQueryPackage};
use tardis::async_trait::async_trait;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm::sea_query::{Query, SelectStatement};
use tardis::db::sea_orm::*;
use tardis::{TardisFunsInst, TardisFuns};



pub struct VcodeStrategeServ;

#[async_trait]
impl
    RbumCrudOperation<
        reach_vcode_strategy::ActiveModel,
        ReachVCodeStrategyAddReq,
        ReachVCodeStrategyModifyReq,
        ReachVCodeStrategySummaryResp,
        ReachVCodeStrategyDetailResp,
        ReachVCodeStrategyFilterReq,
    > for VcodeStrategeServ
{
    fn get_table_name() -> &'static str {
        reach_vcode_strategy::Entity.table_name()
    }
    async fn package_add(add_req: &ReachVCodeStrategyAddReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<reach_vcode_strategy::ActiveModel> {
        let mut model = reach_vcode_strategy::ActiveModel::from(add_req);
        model.id = Set(TardisFuns::field.nanoid());
        model.fill_ctx(ctx, true);
        Ok(model)
    }

    async fn before_add_rbum(_add_req: &mut ReachVCodeStrategyAddReq, _funs: &TardisFunsInst, _ctx: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    async fn package_modify(id: &str, modify_req: &ReachVCodeStrategyModifyReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<reach_vcode_strategy::ActiveModel> {
        let mut model = reach_vcode_strategy::ActiveModel::from(modify_req);
        model.id = Set(id.into());
        model.fill_ctx(ctx, true);
        Ok(model)
    }

    async fn package_query(is_detail: bool, filter: &ReachVCodeStrategyFilterReq, _funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        if let Some(rel_reach_set_id) = &filter.rel_reach_set_id {
            query.and_where(reach_vcode_strategy::Column::RelReachSetId.eq(rel_reach_set_id));
        }
        query.with_filter(Self::get_table_name(), &filter.base_filter.basic, is_detail, false, ctx);
        Ok(query)
    }
}
