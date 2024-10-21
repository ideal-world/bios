use tardis::db::sea_orm::prelude::*;

use tardis::db::sea_orm;

use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};

use crate::dto::event_dto::TopicAuth;
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "mq_auth")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    #[sea_orm(indexed)]
    pub topic: String,
    #[sea_orm(indexed)]
    pub ak: String,
    pub read: bool,
    pub write: bool,
}

impl Model {
    pub fn from_topic_auth(auth: TopicAuth) -> Self {
        let id = format!("{}/{}", auth.topic, auth.ak);
        Model {
            id,
            topic: auth.topic,
            ak: auth.ak,
            read: auth.read,
            write: auth.write,
        }
    }
    pub fn into_topic_auth(self) -> TopicAuth {
        TopicAuth {
            topic: self.topic,
            ak: self.ak,
            read: self.read,
            write: self.write,
        }
    }
}
