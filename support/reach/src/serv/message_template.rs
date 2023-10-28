use crate::domain::message_template;
use crate::dto::*;

use tardis::async_trait::async_trait;

use bios_basic::rbum::serv::rbum_crud_serv::{RbumCrudOperation, RbumCrudQueryPackage};
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisActiveModel;

use tardis::db::sea_orm::sea_query::{Query, SelectStatement};
use tardis::db::sea_orm::{ColumnTrait, Set};
use tardis::db::sea_orm::{EntityName, Iterable};
use tardis::{TardisFuns, TardisFunsInst};

pub struct ReachMessageTemplateServ;

#[async_trait]
impl
    RbumCrudOperation<
        message_template::ActiveModel,
        ReachMessageTemplateAddReq,
        ReachMessageTemplateModifyReq,
        ReachMessageTemplateSummaryResp,
        ReachMessageTemplateDetailResp,
        ReachMessageTemplateFilterReq,
    > for ReachMessageTemplateServ
{
    fn get_table_name() -> &'static str {
        message_template::Entity.table_name()
    }
    async fn package_add(add_req: &ReachMessageTemplateAddReq, _funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<message_template::ActiveModel> {
        let mut model = message_template::ActiveModel::from(add_req);
        model.id = Set(TardisFuns::field.nanoid());
        model.fill_ctx(ctx, true);
        Ok(model)
    }
    async fn before_add_rbum(_add_req: &mut ReachMessageTemplateAddReq, _funs: &TardisFunsInst, _ctx: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    async fn package_modify(id: &str, modify_req: &ReachMessageTemplateModifyReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<message_template::ActiveModel> {
        let mut model = message_template::ActiveModel::from(modify_req);
        model.id = Set(id.into());
        model.fill_ctx(ctx, false);
        Ok(model)
    }

    async fn package_query(is_detail: bool, filter: &ReachMessageTemplateFilterReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        query.from(message_template::Entity);
        query.columns(message_template::Column::iter().map(|c| (message_template::Entity, c)));
        if let Some(chan) = filter.rel_reach_channel {
            query.and_where(message_template::Column::RelReachChannel.eq(chan));
        }
        if let Some(level_kind) = filter.level_kind {
            query.and_where(message_template::Column::LevelKind.eq(level_kind));
        }
        if let Some(kind) = filter.kind {
            query.and_where(message_template::Column::Kind.eq(kind));
        }
        if let Some(rel_reach_verify_code_strategy_id) = filter.rel_reach_verify_code_strategy_id {
            query.and_where(message_template::Column::RelReachVerifyCodeStrategyId.eq(rel_reach_verify_code_strategy_id));
        }
        query.with_filter(Self::get_table_name(), &filter.base_filter, is_detail, true, ctx);
        Ok(query)
    }
}

impl ReachMessageTemplateServ {
    pub async fn get_by_id(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<ReachMessageTemplateDetailResp> {
        let rbum = Self::get_rbum(id, &ReachMessageTemplateFilterReq::default(), funs, ctx).await?;
        Ok(rbum)
    }
}
