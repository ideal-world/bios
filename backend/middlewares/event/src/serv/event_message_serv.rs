use asteroid_mq::{
    prelude::{DurableMessage, MessageId, TopicCode},
    protocol::{node::raft::proposal::MessageStateUpdate, topic::durable_message::DurableMessageQuery},
};
use tardis::{
    basic::{error::TardisError, result::TardisResult},
    chrono::Utc,
    db::sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, IntoActiveModel, QueryFilter, QuerySelect, Set, Unchanged},
    TardisFunsInst,
};

use crate::domain::event_message::{ActiveModel, Column, Entity, Model};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EventMessageServ;

impl EventMessageServ {
    pub async fn save(&self, topic: TopicCode, message: DurableMessage, funs: &TardisFunsInst) -> TardisResult<()> {
        let model: Model = Model::from_durable_message(topic, message);
        let model: ActiveModel = model.into_active_model();
        let conn = funs.reldb().conn();
        let raw_conn = conn.raw_conn();
        let _result = model.insert(raw_conn).await?;
        Ok(())
    }
    pub async fn archive(&self, topic: TopicCode, message_id: MessageId, funs: &TardisFunsInst) -> TardisResult<()> {
        let update = Entity::update(ActiveModel {
            message_id: Unchanged(message_id.to_base64()),
            archived: Set(true),
            ..Default::default()
        })
        .filter(Column::Topic.eq(topic.to_string()));
        let conn = funs.reldb().conn();
        let raw_conn = conn.raw_conn();
        let result = update.exec(raw_conn).await;
        match result {
            Err(DbErr::RecordNotUpdated) => return Ok(()),
            _result => _result,
        }?;
        Ok(())
    }
    pub async fn batch_retrieve(&self, topic: TopicCode, query: DurableMessageQuery, funs: &TardisFunsInst) -> TardisResult<Vec<DurableMessage>> {
        let DurableMessageQuery { limit, offset, .. } = query;
        let select = Entity::find()
            .filter(Column::Archived.eq(false))
            .filter(Column::Time.gte(Utc::now()))
            .filter(Column::Topic.eq(topic.to_string()))
            .limit(Some(limit as u64))
            .offset(Some(offset as u64));
        let conn = funs.reldb().conn();
        let raw_conn = conn.raw_conn();
        let models = select.all(raw_conn).await?;
        models.into_iter().map(|model| model.try_into_durable_message()).collect::<TardisResult<Vec<DurableMessage>>>()
    }
    pub async fn retrieve(&self, topic: TopicCode, message_id: MessageId, funs: &TardisFunsInst) -> TardisResult<DurableMessage> {
        let select = Entity::find()
            .filter(Column::Archived.eq(false))
            .filter(Column::Time.gte(Utc::now()))
            .filter(Column::Topic.eq(topic.to_string()))
            .filter(Column::MessageId.eq(message_id.to_base64()));
        let conn = funs.reldb().conn();
        let raw_conn = conn.raw_conn();
        let model = select.one(raw_conn).await?;
        model.ok_or_else(|| TardisError::not_found(&format!("event message {} not found", message_id), "event-message-not-found"))?.try_into_durable_message()
    }
    pub async fn update_status(&self, topic: TopicCode, update: MessageStateUpdate, funs: &TardisFunsInst) -> TardisResult<()> {
        let MessageStateUpdate { message_id, status, .. } = update;
        let select = Entity::find().filter(Column::Archived.eq(false)).filter(Column::Topic.eq(topic.to_string())).filter(Column::MessageId.eq(message_id.to_base64()));
        let conn = funs.reldb().conn();
        let raw_conn = conn.raw_conn();
        let model = select.one(raw_conn).await?;
        if let Some(mut model) = model {
            model.status_update(status);
            Entity::update(ActiveModel {
                message_id: Unchanged(message_id.to_base64()),
                status: Set(model.status),
                ..Default::default()
            })
            .filter(Column::Topic.eq(topic.to_string()))
            .exec(raw_conn)
            .await?;
        }
        Ok(())
    }
}
