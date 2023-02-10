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

const MGR_LISTENER_AVATAR_PREFIX: &str = "_";

pub(crate) async fn register(listener: EventListenerRegisterReq, funs: &TardisFunsInst) -> TardisResult<EventListenerRegisterResp> {
    if let Some(topic) = TOPICS.read().await.get(&listener.topic_code.to_string()) {
        let sk = listener.topic_sk.clone().unwrap_or("".to_string());
        let mgr = if sk == topic.use_sk {
            false
        } else if sk == topic.mgr_sk {
            true
        } else {
            return Err(funs.err().unauthorized("listener", "register", "sk do not match", "401-event-listener-sk-not-match"));
        };
        let avatars = if mgr {
            let mut mgr_listeners = MGR_LISTENERS.write().await;
            mgr_listeners.entry(listener.topic_code.to_string()).or_insert_with(HashMap::new);
            let mgr_listeners_with_topic = mgr_listeners.get_mut(&listener.topic_code.to_string()).unwrap();
            match &listener.events {
                Some(events) => events
                    .iter()
                    .map(|event_code| {
                        mgr_listeners_with_topic.insert(event_code.to_string(), format!("{MGR_LISTENER_AVATAR_PREFIX}{event_code}"));
                        format!("{MGR_LISTENER_AVATAR_PREFIX}{event_code}")
                    })
                    .collect(),
                None => {
                    mgr_listeners_with_topic.insert("".to_string(), MGR_LISTENER_AVATAR_PREFIX.to_string());
                    vec![MGR_LISTENER_AVATAR_PREFIX.to_string()]
                }
            }
        } else {
            if listener.avatars.is_empty() {
                return Err(funs.err().bad_request("listener", "register", "avatars can not be empty", "400-event-listener-avatars-empty"));
            }
            if listener.avatars.iter().any(|v| v.starts_with(MGR_LISTENER_AVATAR_PREFIX)) {
                return Err(funs.err().bad_request(
                    "listener",
                    "register",
                    &format!("non-management avatars can not start with '{MGR_LISTENER_AVATAR_PREFIX}'"),
                    "400-event-listener-avatars-invalid",
                ));
            }
            listener.avatars.iter().map(|v| v.to_string()).collect()
        };

        let listener_code = TardisFuns::crypto.base64.encode(&TardisFuns::field.nanoid());
        let token = TardisFuns::field.nanoid_len(32);

        let listener_info = EventListenerInfo {
            topic_code: listener.topic_code.to_string(),
            events: listener.events.map(|v| v.iter().map(|v| v.to_string()).collect()),
            mgr,
            subscribe_mode: listener.subscribe_mode,
            token: token.clone(),
            avatars: avatars.clone(),
        };
        LISTENERS.write().await.insert(listener_code.clone(), listener_info);
        let event_url = funs.conf::<EventConfig>().event_url();
        Ok(EventListenerRegisterResp {
            listener_code: listener_code.clone(),
            ws_addr: format!("{event_url}proc/{listener_code}?token={token}"),
        })
    } else {
        Err(funs.err().not_found("listener", "register", "topic not found", "404-event-topic-not-exist"))
    }
}

pub(crate) async fn remove(listener_code: &str, token: &str, funs: &TardisFunsInst) -> TardisResult<()> {
    if let Some(listener) = LISTENERS.read().await.get(listener_code) {
        if listener.token == token {
            LISTENERS.write().await.remove(listener_code);
            if listener.mgr {
                let mut mgr_listeners = MGR_LISTENERS.write().await;
                if let Some(event_code_info) = mgr_listeners.get_mut(&listener.topic_code) {
                    match &listener.events {
                        Some(events) => events.iter().for_each(|event_code| {
                            event_code_info.remove(event_code);
                        }),
                        None => {
                            event_code_info.remove("");
                        }
                    }
                }
            }
            Ok(())
        } else {
            Err(funs.err().unauthorized("listener", "remove", "token do not match", "401-event-listener-token-not-match"))
        }
    } else {
        Err(funs.err().not_found("listener", "remove", "listener not found", "404-event-listener-not-exist"))
    }
}
