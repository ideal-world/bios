use crate::domain::message_log;
use crate::dto::*;
use bios_basic::rbum::serv::rbum_crud_serv::{RbumCrudOperation, RbumCrudQueryPackage};
use tardis::async_trait::async_trait;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisActiveModel;

use tardis::db::sea_orm::sea_query::{Query, SelectStatement};
use tardis::db::sea_orm::EntityName;
use tardis::db::sea_orm::{ColumnTrait, Set};
use tardis::{TardisFunsInst, TardisFuns};



pub struct ReachMessageLogServ;
#[async_trait]
impl RbumCrudOperation<message_log::ActiveModel, ReachMsgLogAddReq, ReachMsgLogModifyReq, ReachMsgLogSummaryResp, ReachMsgLogDetailResp, ReachMsgLogFilterReq>
    for ReachMessageLogServ
{
    fn get_table_name() -> &'static str {
        message_log::Entity.table_name()
    }
    async fn package_add(add_req: &ReachMsgLogAddReq, _funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<message_log::ActiveModel> {
        let mut model = message_log::ActiveModel::from(add_req);
        model.id = Set(TardisFuns::field.nanoid());
        model.fill_ctx(ctx, true);
        Ok(model)
    }
    async fn before_add_rbum(_add_req: &mut ReachMsgLogAddReq, _funs: &TardisFunsInst, _ctx: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    async fn package_modify(id: &str, modify_req: &ReachMsgLogModifyReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<message_log::ActiveModel> {
        let mut model = message_log::ActiveModel::from(modify_req);
        model.id = Set(id.into());
        model.fill_ctx(ctx, true);
        Ok(model)
    }

    async fn package_query(is_detail: bool, filter: &ReachMsgLogFilterReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        if let Some(id) = &filter.rel_reach_message_id {
            query.and_where(message_log::Column::RelReachMessageId.eq(id));
        }
        query.with_filter(Self::get_table_name(), &filter.base_filter.basic, is_detail, false, ctx);
        Ok(query)
    }
}
