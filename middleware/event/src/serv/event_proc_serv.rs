use std::sync::Arc;
use std::{borrow::Cow, collections::HashMap};

use serde::{Deserialize, Serialize};
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::cluster::cluster_processor::{ClusterEventTarget, TardisClusterMessageReq};
use tardis::cluster::cluster_publish::publish_event_no_response;
use tardis::log::{info, warn};
use tardis::serde_json::Value;
use tardis::tokio::sync::RwLock;
use tardis::web::poem::web::websocket::{BoxWebSocketUpgraded, WebSocket};
use tardis::web::ws_processor::{ws_broadcast, ws_echo, TardisWebsocketMgrMessage, TardisWebsocketResp};
use tardis::{
    cluster::{cluster_broadcast::ClusterBroadcastChannel, cluster_processor::TardisClusterSubscriber},
    tardis_static,
};
use tardis::{TardisFuns, TardisFunsInst};

use crate::dto::event_dto::EventMessageMgrWrap;
use crate::event_config::EventConfig;
use crate::event_constants::{DOMAIN_CODE, SERVICE_EVENT_BUS_AVATAR};
use crate::event_initializer::ws_client;

use super::event_listener_serv::{listeners, mgr_listeners};
use super::event_topic_serv::topics;

tardis_static! {
    // temporary no cleaner for senders
    pub senders: Arc<RwLock<HashMap<String, Arc<ClusterBroadcastChannel<TardisWebsocketMgrMessage>>>>>;
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CreateRemoteSenderEvent {
    pub topic_code: String,
    pub capacity: usize,
}

pub struct CreateRemoteSenderSubscriber;

#[async_trait::async_trait]
impl TardisClusterSubscriber for CreateRemoteSenderSubscriber {
    fn event_name(&self) -> Cow<'static, str> {
        "bios/event/create_remote_sender".into()
    }
    async fn subscribe(&self, message_req: TardisClusterMessageReq) -> TardisResult<Option<Value>> {
        let CreateRemoteSenderEvent { topic_code, capacity } = TardisFuns::json.json_to_obj(message_req.msg)?;

        let clst_bc_tx = ClusterBroadcastChannel::new(topic_code.clone(), capacity);
        let mut wg = senders().write().await;
        wg.insert(topic_code, clst_bc_tx);

        Ok(None)
    }
}

pub async fn add_sender(topic_code: String, capacity: usize) {
    let clst_bc_tx = ClusterBroadcastChannel::new(topic_code.clone(), capacity);
    let mut wg = senders().write().await;
    wg.insert(topic_code.clone(), clst_bc_tx);
    drop(wg);
    if TardisFuns::fw_config().cluster.is_some() {
        let _ = publish_event_no_response(
            CreateRemoteSenderSubscriber.event_name(),
            TardisFuns::json.obj_to_json(&CreateRemoteSenderEvent { topic_code, capacity }).expect("invalid json"),
            ClusterEventTarget::Broadcast,
        )
        .await;
    }
}

pub(crate) async fn ws_process(listener_code: String, token: String, websocket: WebSocket, funs: &TardisFunsInst) -> BoxWebSocketUpgraded {
    let Ok(Some(listener)) = listeners().get(listener_code.clone()).await else {
        return ws_error(listener_code, "listener not found", websocket);
    };
    if listener.token != token {
        return ws_error(listener_code, "permission check failed", websocket);
    }

    let Ok(Some(topic)) = topics().get(listener.topic_code.clone()).await else {
        return ws_error(listener_code, "topic not found", websocket);
    };
    let need_mgr = topic.need_mgr;
    let save_message = topic.save_message;
    let is_mgr = listener.mgr;

    if !senders().read().await.contains_key(&listener.topic_code) {
        add_sender(listener.topic_code.clone(), topic.queue_size as usize).await;
    }
    let sender = senders().read().await.get(&listener.topic_code).expect("conflict on topic sender").clone();
    ws_broadcast(
        listener.avatars.clone(),
        listener.mgr,
        listener.subscribe_mode,
        HashMap::from([
            ("listener_code".to_string(), listener_code),
            ("topic_code".to_string(), listener.topic_code.clone()),
            ("spi_app_id".to_string(), funs.conf::<EventConfig>().spi_app_id.clone()),
        ]),
        websocket,
        sender,
        move |req_msg, ext| async move {
            if save_message {
                let spi_app_id = ext.get("spi_app_id").expect("spi_app_id was modified unexpectedly");
                if spi_app_id.is_empty() {
                    info!("[Event] MESSAGE LOG: {}", TardisFuns::json.obj_to_string(&req_msg).expect("req_msg not a valid json value"));
                } else {
                    use bios_sdk_invoke::clients::spi_log_client::{LogItemAddReq, SpiLogEventExt};
                    let ws_client = ws_client().await;
                    let ctx = TardisContext {
                        owner: spi_app_id.clone(),
                        ..Default::default()
                    };
                    let req = LogItemAddReq {
                        tag: DOMAIN_CODE.to_string(),
                        content: TardisFuns::json.obj_to_string(&req_msg).expect("req_msg not a valid json value"),
                        kind: None,
                        ext: None,
                        key: None,
                        op: None,
                        rel_key: None,
                        id: None,
                        ts: None,
                        owner: Some(ctx.owner.clone()),
                        own_paths: Some(ctx.own_paths.clone()),
                    };
                    if let Err(e) = ws_client.publish_add_log(&req, SERVICE_EVENT_BUS_AVATAR.to_string(), spi_app_id.clone(), &ctx).await {
                        warn!("[Bios.Event] publish log fail: {}", e);
                    }
                }
            }
            if !need_mgr || is_mgr {
                return Some(TardisWebsocketResp {
                    msg: req_msg.msg,
                    to_avatars: req_msg.to_avatars.unwrap_or_default(),
                    ignore_avatars: vec![],
                });
            }
            // TODO set cache
            let topic_code = ext.get("topic_code").expect("topic_code was modified unexpectedly");
            let msg_avatar = if let Some(req_event_code) = req_msg.event.clone() {
                mgr_listeners().get((topic_code.clone(), req_event_code)).await
            } else {
                mgr_listeners().get((topic_code.clone(), Default::default())).await
            };
            let Ok(Some(msg_avatar)) = msg_avatar else {
                warn!(
                    "[Event] topic [{}] event code [{}] management node not found",
                    topic_code,
                    &req_msg.event.unwrap_or_default()
                );
                return None;
            };
            Some(TardisWebsocketResp {
                msg: TardisFuns::json
                    .obj_to_json(&EventMessageMgrWrap {
                        msg: req_msg.msg,
                        ori_from_avatar: req_msg.from_avatar,
                        ori_to_avatars: req_msg.to_avatars,
                    })
                    .expect("EventMessageMgrWrap not a valid json value"),
                to_avatars: vec![msg_avatar.clone()],
                ignore_avatars: vec![],
            })
        },
        |_, _| async move {},
    )
    .await
}

fn ws_error(req_session: String, error: &str, websocket: WebSocket) -> BoxWebSocketUpgraded {
    ws_echo(
        req_session,
        HashMap::from([("error".to_string(), error.to_string())]),
        websocket,
        |_, _, ext| async move {
            let error = ext.get("error").expect("error was modified unexpectedly");
            warn!("[Event] Websocket connection error: {}", error);
            Some(format!("Websocket connection error: {}", error))
        },
        |_, _| async move {},
    )
}
