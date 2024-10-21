//！ The adapter layer between mq and bios services
use std::sync::Arc;

use asteroid_mq::{
    prelude::{Durable, DurableError, NodeId},
    protocol::node::edge::{
        auth::{EdgeAuth, EdgeAuthError},
        EdgeRequestEnum,
    },
};
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use tardis::{basic::dto::TardisContext, TardisFunsInst};

use crate::{
    dto::event_dto::{EventTopicAddOrModifyReq, EventTopicFilterReq},
    serv::{event_auth_serv::EventAuthServ, event_message_serv::EventMessageServ, event_register_serv::EventRegisterServ, event_topic_serv::EventTopicServ},
};

/*
Durable Service Adapter
*/
pub struct BiosDurableAdapter {
    funs: Arc<TardisFunsInst>,
    message_serv: EventMessageServ,
    ctx: TardisContext,
}

impl BiosDurableAdapter {
    const CONTEXT: &str = "bios durable adapter";
    pub fn new(funs: Arc<TardisFunsInst>, ctx: TardisContext) -> Self {
        Self {
            funs,
            message_serv: EventMessageServ,
            ctx,
        }
    }
}

impl Durable for BiosDurableAdapter {
    async fn save(&self, topic: asteroid_mq::prelude::TopicCode, message: asteroid_mq::prelude::DurableMessage) -> Result<(), DurableError> {
        self.message_serv.save(topic, message, &self.funs).await.map_err(|e| DurableError::with_source(Self::CONTEXT, e))
    }
    async fn archive(&self, topic: asteroid_mq::prelude::TopicCode, message_id: asteroid_mq::prelude::MessageId) -> Result<(), asteroid_mq::prelude::DurableError> {
        self.message_serv.archive(topic, message_id, &self.funs).await.map_err(|e| DurableError::with_source(Self::CONTEXT, e))
    }
    async fn batch_retrieve(
        &self,
        topic: asteroid_mq::prelude::TopicCode,
        query: asteroid_mq::protocol::topic::durable_message::DurableMessageQuery,
    ) -> Result<Vec<asteroid_mq::prelude::DurableMessage>, asteroid_mq::prelude::DurableError> {
        self.message_serv.batch_retrieve(topic, query, &self.funs).await.map_err(|e| DurableError::with_source(Self::CONTEXT, e))
    }
    async fn retrieve(
        &self,
        topic: asteroid_mq::prelude::TopicCode,
        message_id: asteroid_mq::prelude::MessageId,
    ) -> Result<asteroid_mq::prelude::DurableMessage, asteroid_mq::prelude::DurableError> {
        self.message_serv.retrieve(topic, message_id, &self.funs).await.map_err(|e| DurableError::with_source(Self::CONTEXT, e))
    }
    async fn update_status(
        &self,
        topic: asteroid_mq::prelude::TopicCode,
        update: asteroid_mq::protocol::node::raft::proposal::MessageStateUpdate,
    ) -> Result<(), asteroid_mq::prelude::DurableError> {
        self.message_serv.update_status(topic, update, &self.funs).await.map_err(|e| DurableError::with_source(Self::CONTEXT, e))
    }
    async fn create_topic(&self, config: asteroid_mq::prelude::TopicConfig) -> Result<(), asteroid_mq::prelude::DurableError> {
        let mut req = EventTopicAddOrModifyReq::from_config(config);
        EventTopicServ::add_item(&mut req, &self.funs, &self.ctx).await.map_err(|e| DurableError::with_source(Self::CONTEXT, e))?;
        Ok(())
    }
    async fn delete_topic(&self, topic: asteroid_mq::prelude::TopicCode) -> Result<(), asteroid_mq::prelude::DurableError> {
        EventTopicServ::delete_item(&topic.to_string(), &self.funs, &self.ctx).await.map_err(|e| DurableError::with_source(Self::CONTEXT, e))?;
        Ok(())
    }
    async fn topic_code_list(&self) -> Result<Vec<asteroid_mq::prelude::TopicCode>, DurableError> {
        let ids = EventTopicServ::find_id_items(&EventTopicFilterReq { ..Default::default() }, None, None, &self.funs, &self.ctx)
            .await
            .map_err(|e| DurableError::with_source(Self::CONTEXT, e))?;
        Ok(ids.into_iter().map(asteroid_mq::prelude::TopicCode::from).collect())
    }
    async fn topic_list(&self) -> Result<Vec<asteroid_mq::prelude::TopicConfig>, DurableError> {
        let items = EventTopicServ::find_items(&EventTopicFilterReq { ..Default::default() }, None, None, &self.funs, &self.ctx)
            .await
            .map_err(|e| DurableError::with_source(Self::CONTEXT, e))?;
        Ok(items.into_iter().map(|item| item.into_topic_config()).collect())
    }
}

/*
Auth Service Adapter
*/

pub struct BiosEdgeAuthAdapter {
    funs: Arc<TardisFunsInst>,
    ctx: TardisContext,
    auth_serv: EventAuthServ,
    register_serv: EventRegisterServ,
}

impl BiosEdgeAuthAdapter {
    pub fn new(funs: Arc<TardisFunsInst>, ctx: TardisContext) -> Self {
        Self {
            funs,
            ctx,
            auth_serv: EventAuthServ {},
            register_serv: EventRegisterServ {},
        }
    }
}

impl EdgeAuth for BiosEdgeAuthAdapter {
    async fn check<'r>(&'r self, from: NodeId, request: &'r EdgeRequestEnum) -> Result<(), EdgeAuthError> {
        let funs = &self.funs;
        let ctx = &self.ctx;
        enum CheckOption {
            Write,
            Read,
        }

        let (topic, check_option) = match request {
            EdgeRequestEnum::SendMessage(edge_message) => (edge_message.header.topic.clone(), CheckOption::Write),
            EdgeRequestEnum::EndpointOnline(edge_endpoint_online) => (edge_endpoint_online.topic_code.clone(), CheckOption::Read),
            EdgeRequestEnum::EndpointOffline(edge_endpoint_offline) => (edge_endpoint_offline.topic_code.clone(), CheckOption::Read),
            EdgeRequestEnum::EndpointInterest(endpoint_interest) => (endpoint_interest.topic_code.clone(), CheckOption::Read),
            EdgeRequestEnum::SetState(set_state) => (set_state.topic.clone(), CheckOption::Read),
        };
        if !EventTopicServ::is_check_auth(&topic, funs, ctx).await.map_err(|e| EdgeAuthError::new("topic not found", e))? {
            return Ok(());
        }
        let ctx = self.register_serv.get_ctx(from).await.map_err(|e| EdgeAuthError::new("node_id not registered", e))?;
        let auth = self.auth_serv.get_auth(topic, &ctx.ak, funs).await.map_err(|e| EdgeAuthError::new("auth not found", e))?;

        if match check_option {
            CheckOption::Write => auth.write,
            CheckOption::Read => auth.read,
        } {
            Ok(())
        } else {
            Err(EdgeAuthError::new_local("no write permission"))
        }
    }
}
