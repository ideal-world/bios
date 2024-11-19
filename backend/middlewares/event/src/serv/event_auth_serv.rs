use asteroid_mq::prelude::TopicCode;
use tardis::{
    basic::{error::TardisError, result::TardisResult},
    db::sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter},
    TardisFunsInst,
};

use crate::{
    domain::event_auth::{ActiveModel, Column, Entity, Model},
    dto::event_dto::TopicAuth,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]

pub struct EventAuthServ {}

impl EventAuthServ {
    pub fn new() -> EventAuthServ {
        EventAuthServ {}
    }

    pub async fn set_auth(&self, auth: TopicAuth, funs: &TardisFunsInst) -> TardisResult<()> {
        let select = Entity::find().filter(Column::Topic.eq(&auth.topic)).filter(Column::Ak.eq(&auth.ak));
        let conn = funs.reldb().conn();
        let raw_conn = conn.raw_conn();
        let model = select.one(raw_conn).await?;
        if model.is_none() {
            let model: Model = Model::from_topic_auth(auth);
            let model: ActiveModel = model.into_active_model();
            model.insert(raw_conn).await?;
        }

        Ok(())
    }

    pub async fn get_auth(&self, topic: TopicCode, ak: &str, funs: &TardisFunsInst) -> TardisResult<TopicAuth> {
        let select = Entity::find().filter(Column::Topic.eq(topic.to_string())).filter(Column::Ak.eq(ak));
        let conn = funs.reldb().conn();
        let raw_conn = conn.raw_conn();
        let model = select.one(raw_conn).await?;
        let model = model.ok_or_else(|| TardisError::not_found(&format!("auth for topic {} and ak {} not found", topic, ak), "event-auth-not-found"))?;
        Ok(model.into_topic_auth())
    }
}
