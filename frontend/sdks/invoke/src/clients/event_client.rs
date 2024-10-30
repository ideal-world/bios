use std::{future::Future, marker::PhantomData};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tardis::{
    basic::{dto::TardisContext, error::TardisError, result::TardisResult},
    web::poem_openapi::{self, Object},
    TardisFuns, TardisFunsInst,
};

use crate::invoke_config::InvokeConfigApi;

/******************************************************************************************************************
 *                                                Http Client
 ******************************************************************************************************************/

#[derive(Clone)]
pub struct EventClient<'a> {
    pub funs: &'a TardisFunsInst,
    pub base_url: &'a str,
}

impl<'a> EventClient<'a> {
    pub fn new(url: &'a str, funs: &'a TardisFunsInst) -> Self {
        Self { base_url: url, funs }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventListenerRegisterReq {
    // #[oai(validator(pattern = r"^[a-z0-9]+$"))]
    pub topic_code: String,
    pub topic_sk: Option<String>,
    // #[oai(validator(pattern = r"^[a-z0-9-_]+$"))]
    pub events: Option<Vec<String>>,
    pub avatars: Vec<String>,
    pub subscribe_mode: bool,
}

#[derive(Serialize, Deserialize, Debug, Object)]
pub struct EventListenerRegisterResp {
    pub ws_addr: String,
    pub listener_code: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct EventTopicAddOrModifyReq {
    // #[oai(validator(pattern = r"^[a-z0-9]+$"))]
    pub code: String,
    pub name: String,
    pub save_message: bool,
    pub need_mgr: bool,
    pub queue_size: i32,
    pub use_sk: Option<String>,
    pub mgr_sk: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContextEvent<T> {
    pub ctx: TardisContext,
    pub event: T,
}

impl<T> ContextEvent<T> {
    pub fn from_funs(funs: &TardisFunsInst, event: T) -> Self {
        Self {
            ctx: TardisContext {
                ak: funs.invoke_conf_spi_app_id(),
                ..Default::default()
            },
            event,
        }
    }
}

impl<T> asteroid_mq::prelude::EventAttribute for ContextEvent<T>
where
    T: asteroid_mq::prelude::EventAttribute,
{
    const BROADCAST: bool = T::BROADCAST;
    const EXPECT_ACK_KIND: asteroid_mq::prelude::MessageAckExpectKind = T::EXPECT_ACK_KIND;
    const SUBJECT: asteroid_mq::prelude::Subject = T::SUBJECT;
}

impl<T> ContextEvent<T> {
    #[inline(always)]
    pub fn unpack(self) -> (TardisContext, T) {
        (self.ctx, self.event)
    }
}

/// Adapter for event with a tardis context
#[derive(Debug, Clone)]
pub struct FnContextEventHandler<E>(PhantomData<E>);
use asteroid_mq::{
    event_handler::json::Json,
    prelude::{EventAttribute, Topic, TopicCode},
};
#[derive(Debug, Clone)]
pub struct ContextHandler<F>(pub F);
pub struct EventCenterClient {
    pub topic_code: TopicCode,
}

impl EventCenterClient {
    pub fn get_topic(&self) -> TardisResult<asteroid_mq::prelude::Topic> {
        mq_node().get_topic(&self.topic_code).ok_or_else(|| TardisError::internal_error(&self.topic_code.to_string(), "topic-not-initialized"))
    }
}
pub trait EventAttributeExt: EventAttribute {
    fn inject_context(self, funs: &TardisFunsInst, ctx: &TardisContext) -> ContextEvent<Self>
    where
        Self: Sized,
    {
        let mut ctx = ctx.clone();
        ctx.ak = funs.invoke_conf_spi_app_id();
        ContextEvent { ctx, event: self }
    }
    fn json(self) -> Json<Self>
    where
        Self: Serialize + DeserializeOwned,
    {
        Json(self)
    }
}

impl<T> EventAttributeExt for T where T: EventAttribute {}

impl<F, E, Fut> asteroid_mq::event_handler::Handler<FnContextEventHandler<E>> for ContextHandler<F>
where
    Json<ContextEvent<E>>: asteroid_mq::event_handler::Event,
    E: Serialize + DeserializeOwned + Send,
    F: Fn(E, TardisContext) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = TardisResult<()>> + Send,
{
    type Event = Json<ContextEvent<E>>;
    type Error = TardisError;
    fn handle(self, event: Json<ContextEvent<E>>) -> impl Future<Output = TardisResult<()>> + Send {
        let (ctx, evt) = event.0.unpack();
        (self.0)(evt, ctx)
    }
}

pub fn mq_node() -> asteroid_mq::prelude::Node {
    TardisFuns::store().get_singleton::<asteroid_mq::prelude::Node>().expect("mq node not initialized")
}
pub fn mq_node_opt() -> Option<asteroid_mq::prelude::Node> {
    TardisFuns::store().get_singleton::<asteroid_mq::prelude::Node>()
}
pub fn get_topic(code: &TopicCode) -> Option<Topic> {
    mq_node_opt()?.get_topic(code)
}
pub const SPI_RPC_TOPIC: TopicCode = TopicCode::const_new("spi");

pub fn mq_error(err: asteroid_mq::Error) -> TardisError {
    TardisError::internal_error(&err.to_string(), "mq-error")
}
pub use asteroid_mq;
