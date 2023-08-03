use crate::domain::message_signature;
pub struct ReachMessageSignatureServ;
use crate::dto::*;
use bios_basic::rbum::serv::rbum_crud_serv::{RbumCrudOperation, RbumCrudQueryPackage};
use tardis::async_trait::async_trait;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisActiveModel;

use tardis::db::sea_orm::sea_query::{Query, SelectStatement};
use tardis::db::sea_orm::{EntityName, Iterable};
use tardis::db::sea_orm::{ColumnTrait, Set};
use tardis::{TardisFuns, TardisFunsInst};

#[async_trait]
impl
    RbumCrudOperation<
        message_signature::ActiveModel,
        ReachMsgSignatureAddReq,
        ReachMsgSignatureModifyReq,
        ReachMsgSignatureSummaryResp,
        ReachMsgSignatureDetailResp,
        ReachMsgSignatureFilterReq,
    > for ReachMessageSignatureServ
{
    fn get_table_name() -> &'static str {
        message_signature::Entity.table_name()
    }
    async fn package_add(add_req: &ReachMsgSignatureAddReq, _funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<message_signature::ActiveModel> {
        let mut model = message_signature::ActiveModel::from(add_req);
        model.id = Set(TardisFuns::field.nanoid());
        model.fill_ctx(ctx, true);
        Ok(model)
    }
    async fn before_add_rbum(_add_req: &mut ReachMsgSignatureAddReq, _funs: &TardisFunsInst, _ctx: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    async fn package_modify(id: &str, modify_req: &ReachMsgSignatureModifyReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<message_signature::ActiveModel> {
        let mut model = message_signature::ActiveModel::from(modify_req);
        model.id = Set(id.into());
        model.fill_ctx(ctx, true);
        Ok(model)
    }

    async fn package_query(is_detail: bool, filter: &ReachMsgSignatureFilterReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        query.from(message_signature::Entity);
        query.columns(message_signature::Column::iter().map(|c|(message_signature::Entity, c)));
        query.with_filter(Self::get_table_name(), &filter.base_filter, is_detail, false, ctx);
        if !filter.name.is_empty() {
            query.and_where(message_signature::Column::Name.contains(filter.name.as_str()));
        }
        if let Some(chan) = filter.rel_reach_channel {
            query.and_where(message_signature::Column::RelReachChannel.eq(chan));
        }
        Ok(query)
    }
}
