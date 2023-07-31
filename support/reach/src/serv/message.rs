use crate::domain::{message, message_template};
use crate::dto::*;
use crate::serv::message_signature::ReachMessageSignatureServ;
use crate::serv::message_template::ReachMessageTemplateServ;
use bios_basic::rbum::serv::rbum_crud_serv::{RbumCrudOperation, RbumCrudQueryPackage};
use tardis::async_trait::async_trait;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisActiveModel;

use tardis::db::sea_orm::sea_query::{Expr, Query, SelectStatement};
use tardis::db::sea_orm::*;
use tardis::{TardisFuns, TardisFunsInst};

pub struct ReachMessageServ;
#[async_trait]
impl RbumCrudOperation<message::ActiveModel, ReachMessageAddReq, ReachMessageModifyReq, ReachMessageSummaryResp, ReachMessageDetailResp, ReachMessageFilterReq>
    for ReachMessageServ
{
    fn get_table_name() -> &'static str {
        message::Entity.table_name()
    }
    async fn package_add(add_req: &ReachMessageAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<message::ActiveModel> {
        Ok(message::ActiveModel {
            id: Set(TardisFuns::field.nanoid()),
            from_res: Set(add_req.from_res.to_string()),
            rel_reach_channel: Set(add_req.rel_reach_channel),
            receive_kind: Set(add_req.receive_kind),
            to_res_ids: Set(add_req.to_res_ids.to_string()),
            rel_reach_msg_signature_id: Set(add_req.rel_reach_msg_signature_id.to_string()),
            rel_reach_msg_template_id: Set(add_req.rel_reach_msg_template_id.to_string()),
            content_replace: Set(add_req.rel_reach_msg_template_id.to_string()),
            reach_status: Set(add_req.reach_status),
            ..Default::default()
        })
    }

    async fn before_add_rbum(add_req: &mut ReachMessageAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        ReachMessageSignatureServ::check_ownership(&add_req.rel_reach_msg_signature_id, funs, ctx).await?;
        ReachMessageTemplateServ::check_scope(&add_req.rel_reach_msg_template_id, ReachMessageTemplateServ::get_table_name(), funs, ctx).await?;
        Ok(())
    }

    async fn package_modify(id: &str, modify_req: &ReachMessageModifyReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<message::ActiveModel> {
        let mut model = message::ActiveModel::from(modify_req);
        model.id = Set(id.into());
        model.fill_ctx(ctx, true);
        Ok(model)
    }

    async fn package_query(is_detail: bool, filter: &ReachMessageFilterReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();

        query.left_join(
            message_template::Entity,
            Expr::col((message_template::Entity, message_template::Column::Id)).equals((message::Entity, message::Column::RelReachMsgTemplateId)),
        );
        if let Some(status) = filter.reach_status {
            query.and_where(message::Column::ReachStatus.eq(status));
        }
        query.with_filter(Self::get_table_name(), &filter.rbum_item_basic_filter_req.basic, is_detail, false, ctx);
        Ok(query)
    }
}

impl ReachMessageServ {
    pub async fn resend(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let mut modify_req = ReachMessageModifyReq {
            reach_status: Some(ReachStatusKind::Pending),
            ..Default::default()
        };
        Self::modify_rbum(id, &mut modify_req, funs, ctx).await?;
        Ok(())
    }
    pub async fn update_status(id: impl Into<String>, status: ReachStatusKind, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        funs.db()
            .update_one(
                message::ActiveModel {
                    id: Set(id.into()),
                    reach_status: Set(status),
                    ..Default::default()
                },
                ctx,
            )
            .await
    }
}
