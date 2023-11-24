use tardis::cluster::cluster_hashmap::ClusterStaticHashMap;
use tardis::tardis_static;
use tardis::TardisFunsInst;
use tardis::{basic::result::TardisResult, TardisFuns};

use crate::dto::event_dto::{EventListenerInfo, EventListenerRegisterReq, EventListenerRegisterResp};
use crate::event_config::EventConfig;
use std::{collections::HashMap, sync::Arc};

use lazy_static::lazy_static;
use tardis::tokio::sync::RwLock;

use super::event_topic_serv::topics;

tardis_static! {
    pub(crate) listeners: ClusterStaticHashMap<String, EventListenerInfo> = ClusterStaticHashMap::new("bios/event/listeners");
    // (topic, event) => avatar
    pub(crate) mgr_listeners: ClusterStaticHashMap<(String, String), String> = ClusterStaticHashMap::new("bios/event/msg_listeners");
}

lazy_static! {
    pub static ref LISTENERS: Arc<RwLock<HashMap<String, EventListenerInfo>>> = Arc::new(RwLock::new(HashMap::new()));
    // topic => event => avatar
    pub static ref MGR_LISTENERS: Arc<RwLock<HashMap<String, HashMap<String, String>>>> = Arc::new(RwLock::new(HashMap::new()));
}

const MGR_LISTENER_AVATAR_PREFIX: &str = "_";

pub(crate) async fn register(listener: EventListenerRegisterReq, funs: &TardisFunsInst) -> TardisResult<EventListenerRegisterResp> {
    if let Some(topic) = topics().get(listener.topic_code.to_string()).await? {
        let sk = listener.topic_sk.clone().unwrap_or("".to_string());
        let mgr = if sk == topic.use_sk {
            false
        } else if sk == topic.mgr_sk {
            true
        } else {
            return Err(funs.err().unauthorized("listener", "register", "sk do not match", "401-event-listener-sk-not-match"));
        };
        let avatars = if mgr {
            // let mut mgr_listeners = MGR_LISTENERS.write().await;
            // let mgr_listeners_with_topic = mgr_listeners.entry(listener.topic_code.to_string()).or_insert_with(HashMap::new);
            let topic = listener.topic_code.to_string();
            match &listener.events {
                Some(events) => {
                    let pairs = events.iter().map(|event| ((topic.to_string(), event.to_string()), format!("{MGR_LISTENER_AVATAR_PREFIX}{event}"))).collect();
                    let avatars = events.iter().map(|event| format!("{MGR_LISTENER_AVATAR_PREFIX}{event}")).collect::<Vec<_>>();
                    mgr_listeners().batch_insert(pairs).await?;
                    avatars
                }
                None => {
                    mgr_listeners().insert((topic.to_string(), String::default()), MGR_LISTENER_AVATAR_PREFIX.to_string()).await?;
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

        let listener_code = TardisFuns::crypto.base64.encode(TardisFuns::field.nanoid());
        let token = TardisFuns::field.nanoid_len(32);

        let listener_info = EventListenerInfo {
            topic_code: listener.topic_code.to_string(),
            events: listener.events.map(|v| v.iter().map(|v| v.to_string()).collect()),
            mgr,
            subscribe_mode: listener.subscribe_mode,
            token: token.clone(),
            avatars: avatars.clone(),
        };
        listeners().insert(listener_code.clone(), listener_info).await?;
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
    if let Some(listener) = listeners().get(listener_code.to_string()).await? {
        if listener.token == token {
            listeners().remove(listener_code.to_string()).await?;
            if listener.mgr {
                let to_removes = match listener.events {
                    Some(events) => events.into_iter().map(|event_code| (listener.topic_code.clone(), event_code)).collect(),
                    None => {
                        vec![(listener.topic_code.clone(), String::default())]
                    }
                };
                mgr_listeners().batch_remove(to_removes).await?;
            }
            Ok(())
        } else {
            Err(funs.err().unauthorized("listener", "remove", "token do not match", "401-event-listener-token-not-match"))
        }
    } else {
        Err(funs.err().not_found("listener", "remove", "listener not found", "404-event-listener-not-exist"))
    }
}
