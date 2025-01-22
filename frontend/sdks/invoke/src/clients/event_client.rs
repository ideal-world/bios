use std::{future::Future, marker::PhantomData};

use crate::{invoke_config::InvokeConfigApi, invoke_enumeration::InvokeModuleKind};
use asteroid_mq_sdk::{
    model::{
        event::{json::Json, EventAttribute},
        *,
    },
    ClientNode,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tardis::{
    basic::{dto::TardisContext, error::TardisError, result::TardisResult},
    web::{
        poem_openapi::{self, Object},
        web_resp::{TardisResp, Void},
    },
    TardisFuns, TardisFunsInst,
};

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

impl<T> EventAttribute for ContextEvent<T>
where
    T: EventAttribute,
{
    const BROADCAST: bool = T::BROADCAST;
    const EXPECT_ACK_KIND: MessageAckExpectKind = T::EXPECT_ACK_KIND;
    const SUBJECT: asteroid_mq_sdk::model::Subject = T::SUBJECT;
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
use asteroid_mq_sdk::model::event::*;
#[derive(Debug, Clone)]
pub struct ContextHandler<F>(pub F);
pub struct EventCenterClient {
    pub topic_code: TopicCode,
}

impl EventCenterClient {}
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

impl<F, E, Fut> Handler<FnContextEventHandler<E>> for ContextHandler<F>
where
    Json<ContextEvent<E>>: Event,
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

pub fn mq_client_node() -> ClientNode {
    TardisFuns::store().get_singleton::<ClientNode>().expect("mq node not initialized")
}
pub fn mq_client_node_opt() -> Option<ClientNode> {
    TardisFuns::store().get_singleton::<ClientNode>()
}

pub const SPI_RPC_TOPIC: TopicCode = TopicCode::const_new("spi");

pub fn mq_error(err: asteroid_mq_sdk::ClientNodeError) -> TardisError {
    TardisError::internal_error(&err.to_string(), "mq-error")
}

pub use asteroid_mq_sdk;

use super::base_spi_client::BaseSpiClient;

#[derive(Clone, Debug, Default)]
pub struct EventClient;

#[derive(Serialize, Debug, Clone)]
pub struct EventTopicConfig {
    pub topic_code: String,
    pub blocking: bool,
    pub overflow_policy: Option<String>,
    pub overflow_size: i32,
    pub check_auth: bool,
}
#[derive(Object, Serialize, Deserialize, Debug, Clone)]

pub struct EventTopicInfoResp {
    // #[oai(validator(pattern = r"^[a-z0-9]+$"))]
    pub code: String,
    pub name: String,
    pub blocking: bool,
    pub topic_code: String,
    pub overflow_policy: String,
    pub overflow_size: i32,
    pub check_auth: bool,
}
impl EventClient {
    pub async fn create_topic(config: &EventTopicConfig, ctx: &TardisContext, funs: &TardisFunsInst) -> TardisResult<String> {
        let url = BaseSpiClient::module_url(InvokeModuleKind::Event, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        let resp = funs.web_client().post::<_, TardisResp<String>>(&format!("{url}/ci/topic"), config, headers.clone()).await?;
        let Some(resp) = resp.body else {
            tardis::tracing::error!("create topic failed: {:?}", resp);
            return Err(TardisError::internal_error("create topic failed", ""));
        };
        if !resp.code.starts_with('2') || resp.data.is_none() {
            tardis::tracing::error!("create topic failed: {:?}", resp);
            return Err(TardisError::internal_error("create topic failed", ""));
        }
        Ok(resp.data.expect("this is checked"))
    }
    pub async fn check_topic_exist(code: &str, ctx: &TardisContext, funs: &TardisFunsInst) -> TardisResult<bool> {
        let url = BaseSpiClient::module_url(InvokeModuleKind::Event, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        let response = funs.web_client().get::<TardisResp<Option<EventTopicInfoResp>>>(&format!("{url}/ci/topic?topic_code={code}"), headers.clone()).await?;
        let Some(resp) = response.body else {
            tardis::tracing::error!("check topic exist failed: {:?}", response);
            return Err(TardisError::internal_error("check topic exist failed", ""));
        };
        if !resp.code.starts_with('2') {
            tardis::tracing::error!("check topic exist failed: {:?}", resp);
            return Err(TardisError::internal_error("check topic exist failed", ""));
        }
        Ok(resp.data.is_some())
    }
    pub async fn register_user(topic_code: &str, read: bool, write: bool,ctx: &TardisContext, funs: &TardisFunsInst) -> TardisResult<()> {
        let url = BaseSpiClient::module_url(InvokeModuleKind::Event, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        let response = funs.web_client().put_to_obj::<TardisResp<Void>>(&format!("{url}/ci/topic/{topic_code}/register?read={read}&write={write}"), "", headers.clone()).await?;
        let Some(resp) = response.body else {
            tardis::tracing::error!("check topic exist failed: {:?}", response);
            return Err(TardisError::internal_error("check topic exist failed", ""));
        };
        if !resp.code.starts_with('2') {
            tardis::tracing::error!("check topic exist failed: {:?}", resp);
            return Err(TardisError::internal_error("check topic exist failed", ""));
        }
        Ok(())
    }
}

#[derive(Object, Deserialize, Serialize, Clone, Debug)]
pub struct EventRegisterResp {
    pub node_id: String,
}
pub async fn create_client_node(ctx: &TardisContext, funs: &TardisFunsInst) -> TardisResult<ClientNode> {
    let module_url = funs.invoke_conf_module_url();
    let url = module_url.get("event").ok_or_else(|| TardisError::internal_error("event module url not found", ""))?;
    let register_url = format!("{url}/ca/register");
    let headers = BaseSpiClient::headers(None, funs, ctx).await?;
    let resp = funs.web_client().put_to_obj::<TardisResp<EventRegisterResp>>(&register_url, "", headers.clone()).await?;
    let Some(resp) = resp.body else {
        return Err(TardisError::internal_error("register event node failed", ""));
    };
    let Some(resp) = resp.data else {
        return Err(TardisError::internal_error("register event node failed", ""));
    };
    let node_id = &resp.node_id;
    let url = url.replace("http://", "ws://").replace("https://", "wss://");
    let connect_url = format!("{url}/ca/connect?node_id={node_id}&codec=bincode");
    let node = ClientNode::connect_ws2_bincode(connect_url).await.map_err(mq_error)?;
    Ok(node)
}

pub async fn init_client_node(max_retry: Option<usize>, retry_duration: std::time::Duration, ctx: &TardisContext, funs: &TardisFunsInst) {
    let mut retry = 0;
    loop {
        match create_client_node(ctx, funs).await {
            Ok(node) => {
                TardisFuns::store().insert_singleton(node);
                tardis::tracing::info!("create client node success");
                break;
            }
            Err(err) => {
                retry += 1;
                if let Some(max_retry) = max_retry {
                    if retry >= max_retry {
                        tardis::log::error!("create client node failed, the max retry times ({max_retry}) was exceeded: {:?}", err);
                        break;
                    }
                } else {
                    tardis::log::warn!("create client node failed: {:?}", err);
                }
                tardis::tokio::time::sleep(retry_duration).await;
            }
        }
    }
}
