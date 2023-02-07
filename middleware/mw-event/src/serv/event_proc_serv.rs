use std::collections::HashMap;
use std::sync::Arc;

use bios_basic::process::ci_processor;
use lazy_static::lazy_static;
use tardis::basic::result::TardisResult;
use tardis::log::warn;
use tardis::serde_json::Value;
use tardis::tokio::sync::broadcast::Sender;
use tardis::tokio::sync::RwLock;
use tardis::web::poem::web::websocket::{BoxWebSocketUpgraded, WebSocket};
use tardis::web::ws_processor::{ws_broadcast, ws_echo, TardisWebsocketResp};
use tardis::{tokio, TardisFuns, TardisFunsInst};

use crate::dto::event_dto::EventMsgReq;
use crate::event_config::EventConfig;

use super::event_def_serv::DEFS;
use super::event_listener_serv::{LISTENERS, MGR_LISTENERS};

lazy_static! {
    static ref SENDERS: Arc<RwLock<HashMap<String, Sender<String>>>> = Arc::new(RwLock::new(HashMap::new()));
}

pub(crate) async fn http_process(event_code: String, from_session: String, token: String, msg: Value, funs: &TardisFunsInst) -> TardisResult<()> {
    Ok(())
}

pub(crate) async fn ws_process(event_code: String, listener_code: String, token: String, websocket: WebSocket, funs: &TardisFunsInst) -> BoxWebSocketUpgraded {
    if let Some(listener) = LISTENERS.read().await.get(&format!("{}{}", event_code, listener_code)) {
        if listener.token == token {
            let def = DEFS.read().await;
            let def = def.get(&event_code).unwrap();
            let need_mgr = def.need_mgr;
            let save_message = def.save_message;
            if !SENDERS.read().await.contains_key(&event_code) {
                SENDERS.write().await.insert(event_code.clone(), tokio::sync::broadcast::channel::<String>(def.queue_size as usize).0);
            }
            let sender = SENDERS.read().await.get(&event_code).unwrap().clone();
            ws_broadcast(
                event_code.clone(),
                websocket,
                sender,
                listener_code,
                listener.subscribe_mode,
                HashMap::from([
                    ("event_code".to_string(), event_code),
                    ("log_url".to_string(), funs.conf::<EventConfig>().log_url.clone()),
                    ("app_key".to_string(), TardisFuns::json.obj_to_string(&funs.conf::<EventConfig>().app_key).unwrap()),
                ]),
                move |req_session, msg, ext| async move {
                    if save_message {
                        TardisFuns::web_client()
                            .post_str_to_str(
                                &format!("{}/ci/item", ext.get("log_url").unwrap()),
                                &format!("from:{}, msg:{}", req_session.clone(), msg.clone()),
                                Some(ci_processor::signature(&TardisFuns::json.str_to_obj(ext.get("app_key").unwrap()).unwrap(), "post", "/ci/item", "", Vec::new()).unwrap()),
                            )
                            .await
                            .unwrap();
                    }
                    let msg_req = TardisFuns::json.str_to_obj::<EventMsgReq>(&msg).unwrap();
                    let mgr_listener_code = MGR_LISTENERS.read().await;
                    let mgr_listener_code = mgr_listener_code.get(ext.get("event_code").unwrap());
                    if !msg_req.to_sessions.is_empty() && need_mgr && (mgr_listener_code.is_none() || mgr_listener_code.unwrap() == &req_session) {
                        return Some(TardisWebsocketResp {
                            msg: msg_req.msg,
                            from_seesion: req_session,
                            to_seesions: msg_req.to_sessions,
                            ignore_self: true,
                        });
                    }
                    // // 缓存信息，不要重复去mgr节点
                    if need_mgr && mgr_listener_code.is_some() {
                        return Some(TardisWebsocketResp {
                            msg: msg_req.msg,
                            from_seesion: req_session,
                            to_seesions: vec![mgr_listener_code.unwrap().to_string()],
                            ignore_self: true,
                        });
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
        websocket,
        req_session,
        HashMap::from([("error".to_string(), error.to_string())]),
        |_, _, ext| async move {
            warn!("[Event] Websocket connection error: {}", ext.get("error").unwrap());
            Some(format!("Websocket connection error: {}", ext.get("error").unwrap()))
        },
        |_, _| async move {},
    )
}
