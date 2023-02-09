use tardis::TardisFunsInst;
use tardis::{basic::result::TardisResult, TardisFuns};

use crate::dto::event_dto::{EventListenerInfo, EventListenerRegisterReq, EventListenerRegisterResp};
use crate::event_config::EventConfig;
use std::{collections::HashMap, sync::Arc};

use lazy_static::lazy_static;
use tardis::tokio::sync::RwLock;

use super::event_topic_serv::TOPICS;

lazy_static! {
    pub static ref LISTENERS: Arc<RwLock<HashMap<String, EventListenerInfo>>> = Arc::new(RwLock::new(HashMap::new()));
    pub static ref MGR_LISTENERS: Arc<RwLock<HashMap<String, HashMap<String, String>>>> = Arc::new(RwLock::new(HashMap::new()));
}

pub(crate) async fn register(listener: EventListenerRegisterReq, funs: &TardisFunsInst) -> TardisResult<EventListenerRegisterResp> {
    if let Some(topic) = TOPICS.read().await.get(&listener.topic_code.to_string()) {
        let sk = listener.topic_sk.clone().unwrap_or("".to_string());
        let mgr = if sk == topic.use_sk {
            false
        } else if sk == topic.mgr_sk {
            true
        } else {
            return Err(funs.err().unauthorized("listener", "register", "sk do not match", "401-mw-listener-sk-not-match"));
        };
        let listener_info = EventListenerInfo {
            topic_code: listener.topic_code.to_string(),
            event_code: listener.event_code(),
            mgr,
            subscribe_mode: listener.subscribe_mode,
            token: TardisFuns::field.nanoid_len(32),
            avatars: listener.avatars.iter().map(|v| v.to_string()).collect(),
        };
        let token = listener_info.token.clone();
        let listener_code = TardisFuns::crypto.base64.encode(&TardisFuns::field.nanoid());
        LISTENERS.write().await.insert(listener_code.clone(), listener_info);
        if mgr {
            let mut mgr_listeners = MGR_LISTENERS.write().await;
            if !mgr_listeners.contains_key(&listener.topic_code.to_string()) {
                mgr_listeners.insert(listener.topic_code.to_string(), HashMap::new());
            }
            mgr_listeners.get_mut(&listener.topic_code.to_string()).unwrap().insert(listener.event_code(), listener_code.to_string());
        }
        let event_url = &funs.conf::<EventConfig>().event_url;
        Ok(EventListenerRegisterResp {
            listener_code: listener_code.clone(),
            ws_addr: format!("{}/proc/{}?token={}", event_url, listener_code, token),
        })
    } else {
        Err(funs.err().not_found("listener", "register", "event code not found", "404-mw-event-code-not-exist"))
    }
}

pub(crate) async fn remove(listener_code: &str, token: &str, funs: &TardisFunsInst) -> TardisResult<()> {
    if let Some(listener) = LISTENERS.read().await.get(listener_code) {
        if listener.token == token {
            LISTENERS.write().await.remove(listener_code);
            if listener.mgr {
                let mut mgr_listeners = MGR_LISTENERS.write().await;
                if let Some(event_code_info) = mgr_listeners.get_mut(&listener.topic_code) {
                    event_code_info.remove(&listener.event_code);
                }
            }
            Ok(())
        } else {
            Err(funs.err().unauthorized("listener", "remove", "token do not match", "401-mw-listener-token-not-match"))
        }
    } else {
        Err(funs.err().not_found("listener", "remove", "listener not found", "404-mw-listener-not-exist"))
    }
}
