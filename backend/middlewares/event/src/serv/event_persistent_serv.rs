use tardis::basic::result::TardisResult;
use tardis::db::sea_orm::sea_query::{Expr, Query};
use tardis::db::sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use tardis::futures::{Stream, StreamExt};
use tardis::web::ws_processor::{TardisWebsocketReq, WsBroadcastContext};
use tardis::{serde_json, TardisFunsInst};

use crate::domain::event_persistent;

pub struct EventPersistentServ;
impl EventPersistentServ {
    pub async fn save_message(message: PersistentMessage, funs: &TardisFunsInst) -> TardisResult<()> {
        if let Some(id) = message.req.msg_id.to_owned() {
            let db = funs.db();
            let _ = event_persistent::Entity::insert(event_persistent::ActiveModel {
                id: Set(id),
                message: Set(serde_json::to_value(message.req).expect("TardisWebsocketReq cannot be converted to json")),
                status: Set(event_persistent::Status::Sending.to_string()),
                topic: Set(message.topic),
                inst_id: Set(message.context.inst_id.clone()),
                mgr_node: Set(message.context.mgr_node),
                subscribe_mode: Set(message.context.subscribe_mode),
                ..Default::default()
            })
            .exec(db.raw_conn())
            .await?;
        }
        Ok(())
    }
    pub async fn sending(id: String, funs: &TardisFunsInst) -> TardisResult<()> {
        use tardis::db::sea_orm::StatementBuilder;
        let db = funs.db().raw_conn();
        let query = Query::update()
            .table(event_persistent::Entity)
            .value(event_persistent::Column::RetryTimes, Expr::col(event_persistent::Column::RetryTimes).add(1))
            .cond_where(event_persistent::Column::Id.eq(id))
            .to_owned();
        let statement = StatementBuilder::build(&query, &db.get_database_backend());
        db.execute(statement).await?;

        Ok(())
    }
    pub async fn send_success(id: String, funs: &TardisFunsInst) -> TardisResult<()> {
        let db = funs.db().raw_conn();
        event_persistent::Entity::update(event_persistent::ActiveModel {
            id: Set(id),
            status: Set(event_persistent::Status::Success.to_string()),
            ..Default::default()
        })
        .filter(event_persistent::Column::Status.eq(event_persistent::Status::Sending.as_str()))
        .exec(db)
        .await?;
        Ok(())
    }
    pub async fn send_fail(id: String, error: impl Into<String>, funs: &TardisFunsInst) -> TardisResult<()> {
        let db = funs.db().raw_conn();
        event_persistent::Entity::update(event_persistent::ActiveModel {
            id: Set(id),
            status: Set(event_persistent::Status::Success.to_string()),
            error: Set(Some(error.into())),
            ..Default::default()
        })
        .filter(event_persistent::Column::Status.eq(event_persistent::Status::Sending.as_str()))
        .exec(db)
        .await?;
        Ok(())
    }

    pub async fn scan_failed(funs: &TardisFunsInst, threshold: i32) -> TardisResult<impl Stream<Item = PersistentMessage> + '_> {
        let db = funs.db().raw_conn();
        Ok(event_persistent::Entity::find()
            .filter(event_persistent::Column::Status.eq(event_persistent::Status::Failed.as_str()).and(event_persistent::Column::RetryTimes.lt(threshold)))
            .order_by_desc(event_persistent::Column::UpdateTime)
            .stream(db)
            .await?
            .filter_map(|item| async move {
                let item = item.ok()?;
                let req = serde_json::from_value::<TardisWebsocketReq>(item.message).ok()?;
                let topic = item.topic;
                let context = WsBroadcastContext {
                    inst_id: item.inst_id,
                    mgr_node: item.mgr_node,
                    subscribe_mode: item.subscribe_mode,
                };
                Some(PersistentMessage { req, context, topic })
            }))
    }
}

pub struct PersistentMessage {
    pub req: TardisWebsocketReq,
    pub context: WsBroadcastContext,
    pub topic: String,
}
