use std::{
    collections::HashMap,
    future::Future,
    iter,
    marker::PhantomData,
    pin::Pin,
    sync::{Arc, OnceLock},
    time::Duration,
};

use crossbeam::sync::ShardedLock;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tardis::{
    basic::{component::ComponentKey, dto::TardisContext, error::TardisError, result::TardisResult},
    log::{debug, trace, warn},
    serde_json, tokio,
    web::{
        poem_openapi::{self, Object},
        tokio_tungstenite::tungstenite::Message,
        web_resp::{TardisResp, Void},
        ws_client::TardisWSClient,
        ws_processor::{TardisWebsocketMessage, TardisWebsocketReq},
    },
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

    pub async fn register(&self, req: &EventListenerRegisterReq) -> TardisResult<EventListenerRegisterResp> {
        let url = format!("{}/listener", self.base_url.trim_end_matches('/'));

        let resp = self.funs.web_client().post::<EventListenerRegisterReq, TardisResp<EventListenerRegisterResp>>(&url, req, iter::empty()).await?;
        if let Some(resp) = resp.body {
            if let Some(data) = resp.data {
                return Ok(data);
            } else {
                return Err(self.funs.err().internal_error("event", "register", &resp.msg, ""));
            }
        }
        return Err(self.funs.err().internal_error("event", "register", "failed to register event listener", ""));
    }

    pub async fn remove(&self, listener_code: &str, token: &str) -> TardisResult<()> {
        let url = format!("{}/listener/{}?token={}", self.base_url.trim_end_matches('/'), listener_code, token);
        let resp = self.funs.web_client().delete::<TardisResp<Void>>(&url, iter::empty()).await?;
        if let Some(resp) = resp.body {
            if resp.data.is_some() {
                return Ok(());
            } else {
                return Err(self.funs.err().internal_error("event", "register", &resp.msg, ""));
            }
        }
        return Err(self.funs.err().internal_error("event", "register", "failed to register event listener", ""));
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

/******************************************************************************************************************
 *                                                Event Client
 ******************************************************************************************************************/

// GLOBAL EVENT BUS
pub const TOPIC_BIOS_WORKER_QUEUE: &str = "bios/worker-queue";
pub const TOPIC_BIOS_PUB_SUB: &str = "bios/pub-sub";
pub const TOPIC_PUBLIC: &str = "public";

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct EventTopicConfig {
    pub topic_code: String,
    pub topic_sk: Option<String>,
    pub events: Option<Vec<String>>,
    pub avatars: Vec<String>,
    pub subscribe_mode: bool,
    pub base_url: String,
    pub in_event: bool,
}

impl From<EventTopicConfig> for EventListenerRegisterReq {
    fn from(val: EventTopicConfig) -> Self {
        EventListenerRegisterReq {
            topic_code: val.topic_code,
            topic_sk: val.topic_sk,
            events: val.events,
            avatars: val.avatars,
            subscribe_mode: val.subscribe_mode,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct EventCenterConfig {
    pub base_url: String,
    pub topic_sk: String,
    pub topic_code: String,
    /// The phrase "subscribe" here is **extremely** bad.
    /// 
    /// The difference between subscribe and not subscribe is actually 
    /// the difference between a pub/sub system and a worker queue.
    /// 
    /// We may need to change this to a more meaningful name.
    pub subscribe: bool,
    pub avatars: Vec<String>,
}

impl Default for EventCenterConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8080".to_string(),
            topic_sk: String::default(),
            topic_code: TOPIC_BIOS_WORKER_QUEUE.to_string(),
            subscribe: false,
            avatars: vec![],
        }
    }
}
type WsEventCenterHandler = dyn Fn(serde_json::Value) -> Pin<Box<dyn Future<Output = TardisResult<()>> + Send>> + Send + Sync;
type WsHandlersMap = HashMap<&'static str, Vec<Arc<WsEventCenterHandler>>>;

#[derive(Clone)]
struct WsEventCenter {
    cfg: Arc<EventCenterConfig>,
    handlers: Arc<ShardedLock<WsHandlersMap>>,
    ws_client: Arc<OnceLock<TardisWSClient>>,
}

impl From<EventCenterConfig> for WsEventCenter {
    fn from(cfg: EventCenterConfig) -> Self {
        Self {
            cfg: Arc::new(cfg),
            handlers: Arc::new(ShardedLock::new(HashMap::new())),
            ws_client: Arc::new(OnceLock::new()),
        }
    }
}

impl From<Arc<EventCenterConfig>> for WsEventCenter {
    fn from(cfg: Arc<EventCenterConfig>) -> Self {
        Self {
            cfg,
            handlers: Arc::new(ShardedLock::new(HashMap::new())),
            ws_client: Arc::new(OnceLock::new()),
        }
    }
}

impl std::fmt::Debug for WsEventCenter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let handlers_key = self.handlers.read().expect("should never be poisoned").iter().map(|(k, v)| (*k, v.len())).collect::<HashMap<&'static str, usize>>();
        f.debug_struct("WsEventCenter").field("handlers", &handlers_key).field("ws_client", &self.ws_client).finish()
    }
}

impl EventCenter for WsEventCenter {
    fn init(&self) -> TardisResult<()> {
        let this = self.clone();
        const RETRY_INTERVAL: Duration = Duration::from_secs(1);
        tokio::spawn(async move {
            let config = this.cfg.clone();
            let url = config.base_url.as_str();
            let funs = TardisFuns::inst("", None);
            // wait for web server to start
            loop {
                if TardisFuns::web_server().is_running().await {
                    break;
                } else {
                    tokio::task::yield_now().await
                }
            }
            let events = this.handlers.read().expect("never poisoned").keys().map(|s| String::from(*s)).collect::<Vec<_>>();
            let client = EventClient::new(url, &funs);
            let resp = client
                .register(&EventListenerRegisterReq {
                    topic_code: config.topic_code.to_string(),
                    topic_sk: Some(config.topic_sk.clone()),
                    events: Some(events),
                    avatars: config.avatars.clone(),
                    subscribe_mode: config.subscribe,
                })
                .await
                .expect("fail to register event center");
            let ws_client = loop {
                let ws_client = {
                    let this = this.clone();
                    TardisFuns::ws_client(&resp.ws_addr, move |message| {
                        let this = this.clone();
                        async move {
                            let Message::Text(text) = message else { return None };
                            let Ok(TardisWebsocketMessage { msg, event, .. }) = TardisFuns::json.str_to_obj(&text) else {
                                return None;
                            };
                            if let Some(evt) = event {
                                let evt: Arc<str> = Arc::from(evt);
                                let handlers = { this.handlers.read().expect("never poisoned").get(evt.as_ref()).cloned().unwrap_or_default() };
                                for h in handlers {
                                    let evt = evt.clone();
                                    let msg = msg.clone();
                                    tokio::spawn(async move {
                                        let result = (h)(msg.clone()).await;
                                        if let Err(e) = result {
                                            warn!("encounter an error when processing event [{evt}]: {e}");
                                        }
                                    });
                                }
                            }
                            None
                        }
                    })
                    .await
                };
                match ws_client {
                    Ok(ws_client) => break ws_client,
                    Err(e) => {
                        warn!("fail to connect event center {e}");
                        tokio::time::sleep(RETRY_INTERVAL).await;
                        continue;
                    }
                }
            };
            this.ws_client.get_or_init(|| ws_client);
            loop {
                if let Err(e) = this.ws_client.get().expect("should be initialized").reconnect().await {
                    warn!("ws client fails to reconnect server {e}");
                    tokio::time::sleep(RETRY_INTERVAL).await;
                }
            }
        });
        Ok(())
    }
    async fn publish<E: Event>(&self, event: E) -> TardisResult<()> {
        if let Some(client) = self.ws_client.get() {
            client
                .send_obj(&TardisWebsocketReq {
                    msg_id: event.id(),
                    msg: TardisFuns::json.obj_to_json(&event)?,
                    from_avatar: event.source(),
                    to_avatars: event.targets(),
                    event: Some(E::CODE.to_string()),
                    ignore_self: Some(false),
                    ..Default::default()
                })
                .await
        } else {
            Err(TardisError::internal_error("event center not initialized", ""))
        }
    }

    fn subscribe<A, H: EventHandler<A>>(&self, handler: H) {
        let wrapped_handler: Arc<WsEventCenterHandler> = Arc::new(move |value: serde_json::Value| {
            let handler = handler.clone();
            Box::pin(async move {
                trace!("[EventCenter] handling event [{code}]: {req}", code = H::Event::CODE, req = value);
                let event: H::Event = serde_json::from_value(value).map_err(|e| TardisError::internal_error(&format!("can't deserialize event message: {e}"), ""))?;
                handler.handle(event).await
            })
        });
        let key = H::Event::CODE;
        self.handlers.write().expect("never poisoned").entry(key).or_default().push(wrapped_handler);
    }
}

pub trait EventCenter {
    fn init(&self) -> TardisResult<()>;
    fn publish<E: Event>(&self, event: E) -> impl Future<Output = TardisResult<()>>;
    fn subscribe<A, H: EventHandler<A>>(&self, handler: H);
}

impl<T> EventCenter for Arc<T>
where
    T: EventCenter,
{
    fn init(&self) -> TardisResult<()> {
        T::init(self)
    }

    fn publish<E: Event>(&self, event: E) -> impl Future<Output = TardisResult<()>> {
        self.as_ref().publish(event)
    }

    fn subscribe<A, H: EventHandler<A>>(&self, handler: H) {
        self.as_ref().subscribe(handler)
    }
}
#[derive(Debug, Clone)]
pub struct BiosEventCenter {
    inner: WsEventCenter,
}

impl BiosEventCenter {
    pub fn from_config(config: EventCenterConfig) -> Self {
        Self {
            inner: WsEventCenter::from(config),
        }
    }
    pub fn from_domain(domain: &str) -> Self {
        #[derive(Deserialize)]
        struct EventConfigView {
            event_center: EventCenterConfig,
        }
        let cfg = TardisFuns::cs_config::<EventConfigView>(domain);
        Self {
            inner: WsEventCenter::from(cfg.event_center.clone()),
        }
    }
    #[deprecated = "There's no global instance by now..."]
    pub fn global() -> Option<Self> {
        TardisFuns::store().get_singleton::<Self>()
    }
    #[inline(always)]
    pub fn worker_queue() -> Option<Self> {
        TardisFuns::store().get(TOPIC_BIOS_WORKER_QUEUE.as_bytes())
    }
    #[inline(always)]
    pub fn set_as_worker_queue(&self) {
        TardisFuns::store().insert(ComponentKey::named(TOPIC_BIOS_WORKER_QUEUE.as_bytes()), self.clone());
    }
    #[inline(always)]
    pub fn pub_sub() -> Option<Self> {
        TardisFuns::store().get(TOPIC_BIOS_PUB_SUB.as_bytes())
    }
    #[inline(always)]
    pub fn set_pub_sub(self) {
        TardisFuns::store().insert(ComponentKey::named(TOPIC_BIOS_PUB_SUB.as_bytes()), self);
    }
    #[inline(always)]
    pub fn public() -> Option<Self> {
        TardisFuns::store().get(TOPIC_PUBLIC.as_bytes())
    }
    #[inline(always)]
    pub fn set_public(self) {
        TardisFuns::store().insert(ComponentKey::named(TOPIC_PUBLIC.as_bytes()), self);
    }
}

impl EventCenter for BiosEventCenter {
    fn init(&self) -> TardisResult<()> {
        self.inner.init()
    }
    async fn publish<E: Event>(&self, event: E) -> TardisResult<()> {
        self.inner.publish(event).await
    }

    fn subscribe<A, H: EventHandler<A>>(&self, handler: H) {
        debug!("subscribe event handler for event [{}]", H::Event::CODE);
        self.inner.subscribe(handler);
    }
}

/******************************************************************************************************************
 *                                                Event Trait
 ******************************************************************************************************************/

pub trait Event: Serialize + DeserializeOwned {
    const CODE: &'static str;
    fn source(&self) -> String {
        String::default()
    }
    fn targets(&self) -> Option<Vec<String>> {
        Self::CODE.split_once('/').map(|(service, _)| vec![service.to_string()])
    }
    fn id(&self) -> Option<String> {
        None
    }
}
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct WithSource<E> {
    #[serde(flatten)]
    inner: E,
    #[serde(skip)]
    source: String,
}

impl<E> Event for WithSource<E>
where
    E: Event,
{
    const CODE: &'static str = E::CODE;
    fn source(&self) -> String {
        self.source.clone()
    }
    fn targets(&self) -> Option<Vec<String>> {
        self.inner.targets()
    }
    fn id(&self) -> Option<String> {
        self.inner.id()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct WithTargets<E> {
    #[serde(flatten)]
    inner: E,
    #[serde(skip)]
    targets: Option<Vec<String>>,
}

impl<E> Event for WithTargets<E>
where
    E: Event,
{
    const CODE: &'static str = E::CODE;
    fn source(&self) -> String {
        self.inner.source()
    }
    fn targets(&self) -> Option<Vec<String>> {
        self.targets.clone()
    }
    fn id(&self) -> Option<String> {
        self.inner.id()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct WithId<E> {
    #[serde(flatten)]
    inner: E,
    #[serde(skip)]
    id: Option<String>,
}

impl<E> Event for WithId<E>
where
    E: Event,
{
    const CODE: &'static str = E::CODE;
    fn source(&self) -> String {
        self.inner.source()
    }
    fn targets(&self) -> Option<Vec<String>> {
        self.inner.targets()
    }
    fn id(&self) -> Option<String> {
        self.id.clone()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContextEvent<T> {
    pub ctx: TardisContext,
    pub event: T,
}

impl<T> ContextEvent<T> {
    #[inline(always)]
    pub fn unpack(self) -> (TardisContext, T) {
        (self.ctx, self.event)
    }
}

impl<E: Event> Event for ContextEvent<E> {
    const CODE: &'static str = E::CODE;
    fn source(&self) -> String {
        self.event.source()
    }
    fn targets(&self) -> Option<Vec<String>> {
        self.event.targets()
    }
}
pub trait EventExt {
    fn with_source(self, source: impl Into<String>) -> WithSource<Self>
    where
        Self: Sized,
    {
        WithSource {
            inner: self,
            source: source.into(),
        }
    }
    fn with_targets(self, targets: impl Into<Option<Vec<String>>>) -> WithTargets<Self>
    where
        Self: Sized,
    {
        WithTargets {
            inner: self,
            targets: targets.into(),
        }
    }
    fn with_context(self, ctx: TardisContext) -> ContextEvent<Self>
    where
        Self: Sized,
    {
        ContextEvent { ctx, event: self }
    }
    fn inject_context(self, funs: &TardisFunsInst, ctx: &TardisContext) -> ContextEvent<Self>
    where
        Self: Sized,
    {
        ContextEvent {
            ctx: funs.invoke_conf_inject_context(ctx),
            event: self,
        }
    }
    fn with_id(self, id: Option<String>) -> WithId<Self>
    where
        Self: Sized,
    {
        WithId { inner: self, id }
    }
    fn with_nanoid(self) -> WithId<Self>
    where
        Self: Sized,
    {
        WithId {
            inner: self,
            id: Some(TardisFuns::field.nanoid()),
        }
    }
}

impl<E> EventExt for E where E: Event {}

/******************************************************************************************************************
 *                                                Event Handler
 ******************************************************************************************************************/

pub trait EventHandler<A>: Clone + Sync + Send + 'static {
    type Event: Event;
    fn handle(self, event: Self::Event) -> impl Future<Output = TardisResult<()>> + Send;
}
/// Adapter for event without a tardis context
#[derive(Debug, Clone)]
pub struct FnEventHandler<E>(PhantomData<E>);
impl<E, F, Fut> EventHandler<FnEventHandler<E>> for F
where
    E: Event,
    F: Fn(E) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = TardisResult<()>> + Send,
{
    type Event = E;
    fn handle(self, event: E) -> impl Future<Output = TardisResult<()>> + Send {
        (self)(event)
    }
}

/// Adapter for event with a tardis context
#[derive(Debug, Clone)]
pub struct FnContextEventHandler<E>(PhantomData<E>);
impl<E, F, Fut> EventHandler<FnContextEventHandler<E>> for F
where
    ContextEvent<E>: Event,
    F: Fn(E, TardisContext) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = TardisResult<()>> + Send,
{
    type Event = ContextEvent<E>;
    fn handle(self, event: ContextEvent<E>) -> impl Future<Output = TardisResult<()>> + Send {
        let (ctx, evt) = event.unpack();
        (self)(evt, ctx)
    }
}
