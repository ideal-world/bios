use std::sync::Arc;
use std::{borrow::Cow, collections::HashMap};

use bios_basic::process::ci_processor;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tardis::basic::result::TardisResult;
use tardis::cluster::cluster_processor::TardisClusterMessageReq;
use tardis::log::{info, warn};
use tardis::serde_json::Value;
use tardis::tokio::sync::broadcast::Sender;
use tardis::tokio::sync::RwLock;
use tardis::web::poem::web::websocket::{BoxWebSocketUpgraded, WebSocket};
use tardis::web::ws_processor::{ws_broadcast, ws_echo, TardisWebsocketMgrMessage, TardisWebsocketResp};
use tardis::{
    cluster::{
        cluster_broadcast::ClusterBroadcastChannel,
        cluster_processor::{subscribe, unsubscribe, TardisClusterSubscriber},
    },
    tardis_static,
};
use tardis::{tokio, TardisFuns, TardisFunsInst};

use crate::dto::event_dto::EventMessageMgrWrap;
use crate::event_config::EventConfig;

use super::event_listener_serv::{LISTENERS, MGR_LISTENERS};
use super::event_topic_serv::TOPICS;

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
    wg.insert(topic_code, clst_bc_tx);
}

pub(crate) async fn ws_process(listener_code: String, token: String, websocket: WebSocket, funs: &TardisFunsInst) -> BoxWebSocketUpgraded {
    if let Some(listener) = LISTENERS.read().await.get(&listener_code) {
        if listener.token == token {
            let topic = TOPICS.read().await;
            let topic = topic.get(&listener.topic_code).unwrap();
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
                    ("log_url".to_string(), funs.conf::<EventConfig>().log_url()),
                    ("app_key".to_string(), TardisFuns::json.obj_to_string(&funs.conf::<EventConfig>().app_key).unwrap()),
                ]),
                websocket,
                sender,
                move |req_msg, ext| async move {
                    if save_message {
                        if let Some(log_url) = ext.get("log_url").map(String::as_str) {
                            if log_url == "/" {
                                info!("[Event] MESSAGE LOG: {}", TardisFuns::json.obj_to_string(&req_msg).expect("req_msg not a valid json value"));
                            } else {
                                let app_key = ext.get("app_key").expect("app_key not found");
                                let _ = TardisFuns::json.str_to_obj(ext.get("app_key").unwrap())
                                ci_processor::signature(&TardisFuns::json.str_to_obj(ext.get("app_key").unwrap()).unwrap(), "post", "/ci/item", "", Vec::new()).unwrap(),
                                TardisFuns::web_client()
                                    .post_obj_to_str(
                                        &format!("{}/ci/item", log_url),
                                        &req_msg,
                                        ci_processor::signature(&TardisFuns::json.str_to_obj(ext.get("app_key").unwrap()).unwrap(), "post", "/ci/item", "", Vec::new()).unwrap(),
                                    )
                                    .await
                            }
                        } else {
                            warn!("[Event] MESSAGE LOG ERROR: {}", e);
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
                    if let Some(msg_event_code_info) = MGR_LISTENERS.read().await.get(ext.get("topic_code").unwrap()) {
                        let msg_avatar = if let Some(req_event_code) = &req_msg.event {
                            msg_event_code_info.get(req_event_code)
                        } else {
                            msg_event_code_info.get("")
                        };
                        if let Some(msg_avatar) = msg_avatar {
                            return Some(TardisWebsocketResp {
                                msg: TardisFuns::json
                                    .obj_to_json(&EventMessageMgrWrap {
                                        msg: req_msg.msg,
                                        ori_from_avatar: req_msg.from_avatar,
                                        ori_to_avatars: req_msg.to_avatars,
                                    })
                                    .unwrap(),
                                to_avatars: vec![msg_avatar.clone()],
                                ignore_avatars: vec![],
                            });
                        } else {
                            warn!(
                                "[Event] topic [{}] event code [{}] management node not found",
                                ext.get("topic_code").unwrap(),
                                &req_msg.event.unwrap_or("".to_string())
                            );
                        }
                    }
                    None
                },
                |_, _| async move {},
            )
            .await
        } else {
            ws_error(listener_code, "permission check failed", websocket)
        }
    } else {
        ws_error(listener_code, "listener not found", websocket)
    }
}

fn ws_error(req_session: String, error: &str, websocket: WebSocket) -> BoxWebSocketUpgraded {
    ws_echo(
        req_session,
        HashMap::from([("error".to_string(), error.to_string())]),
        websocket,
        |_, _, ext| async move {
            warn!("[Event] Websocket connection error: {}", ext.get("error").unwrap());
            Some(format!("Websocket connection error: {}", ext.get("error").unwrap()))
        },
        |_, _| async move {},
    )
}
