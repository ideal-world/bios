use crate::domain::{message, message_log};
use crate::dto::*;
use bios_basic::rbum::serv::rbum_crud_serv::{RbumCrudOperation, RbumCrudQueryPackage};
use tardis::async_trait::async_trait;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisActiveModel;

use tardis::db::sea_orm::sea_query::{Query, SelectStatement, Expr, Alias};
use tardis::db::sea_orm::{ColumnTrait, Set};
use tardis::db::sea_orm::{EntityName, Iterable};
use tardis::{TardisFuns, TardisFunsInst};

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
        model.fill_ctx(ctx, false);
        Ok(model)
    }

    async fn package_query(is_detail: bool, filter: &ReachMsgLogFilterReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        query.columns(message_log::Column::iter().map(|c| (message_log::Entity, c)));
        query.expr_as(Expr::col((message::Entity, message::Column::ReachStatus)), Alias::new("reach_status"));
        query.expr_as(Expr::col((message::Entity, message::Column::ReceiveKind)), Alias::new("receive_kind"));
        query.expr_as(Expr::col((message::Entity, message::Column::ToResIds)), Alias::new("to_res_ids"));
        

        query.left_join(
            message::Entity,
            Expr::col((message_log::Entity, message_log::Column::RelReachMessageId)).equals((message::Entity, message::Column::Id)),
        );
        query.from(message_log::Entity);
        if let Some(id) = &filter.rel_reach_message_id {
            query.and_where(message_log::Column::RelReachMessageId.eq(id));
        }
        query.with_filter(Self::get_table_name(), &filter.base_filter.basic, is_detail, false, ctx);

        Ok(query)
    }
}
