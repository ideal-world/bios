use std::collections::HashMap;
use std::sync::Arc;

use bios_basic::process::ci_processor;
use lazy_static::lazy_static;
use tardis::log::{info, warn};
use tardis::tokio::sync::broadcast::Sender;
use tardis::tokio::sync::RwLock;
use tardis::web::poem::web::websocket::{BoxWebSocketUpgraded, WebSocket};
use tardis::web::ws_processor::{ws_broadcast, ws_echo, TardisWebsocketResp};
use tardis::{tokio, TardisFuns, TardisFunsInst};

use crate::dto::event_dto::EventMessageMgrWrap;
use crate::event_config::EventConfig;

use super::event_listener_serv::{LISTENERS, MGR_LISTENERS};
use super::event_topic_serv::TOPICS;

lazy_static! {
    static ref SENDERS: Arc<RwLock<HashMap<String, Sender<String>>>> = Arc::new(RwLock::new(HashMap::new()));
}

pub(crate) async fn ws_process(listener_code: String, token: String, websocket: WebSocket, funs: &TardisFunsInst) -> BoxWebSocketUpgraded {
    if let Some(listener) = LISTENERS.read().await.get(&listener_code) {
        if listener.token == token {
            let topic = TOPICS.read().await;
            let topic = topic.get(&listener.topic_code).unwrap();
            let need_mgr = topic.need_mgr;
            let save_message = topic.save_message;
            let is_mgr = listener.mgr;

            if !SENDERS.read().await.contains_key(&listener.topic_code) {
                SENDERS.write().await.insert(listener.topic_code.clone(), tokio::sync::broadcast::channel::<String>(topic.queue_size as usize).0);
            }
            let sender = SENDERS.read().await.get(&listener.topic_code).unwrap().clone();
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
                        if ext.get("log_url").unwrap() == "/" {
                            info!("[Event] MESSAGE LOG: {}", TardisFuns::json.obj_to_string(&req_msg).unwrap());
                        } else {
                            TardisFuns::web_client()
                                .post_obj_to_str(
                                    &format!("{}/ci/item", ext.get("log_url").unwrap()),
                                    &req_msg,
                                    Some(ci_processor::signature(&TardisFuns::json.str_to_obj(ext.get("app_key").unwrap()).unwrap(), "post", "/ci/item", "", Vec::new()).unwrap()),
                                )
                                .await
                                .unwrap();
                        }
                    }
                    if !need_mgr || is_mgr {
                        return Some(TardisWebsocketResp {
                            msg: req_msg.msg,
                            to_avatars: req_msg.to_avatars.unwrap_or(vec![]),
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
