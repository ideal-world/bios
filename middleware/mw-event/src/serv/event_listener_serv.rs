use tardis::TardisFunsInst;
use tardis::{basic::result::TardisResult, TardisFuns};

use crate::dto::event_dto::{EventListenerInfo, EventListenerRegisterReq, EventListenerRegisterResp};
use crate::event_config::EventConfig;
use std::{collections::HashMap, sync::Arc};

use lazy_static::lazy_static;
use tardis::tokio::sync::RwLock;

use super::event_def_serv::DEFS;

lazy_static! {
    pub static ref LISTENERS: Arc<RwLock<HashMap<String, EventListenerInfo>>> = Arc::new(RwLock::new(HashMap::new()));
    pub static ref MGR_LISTENERS: Arc<RwLock<HashMap<String, String>>> = Arc::new(RwLock::new(HashMap::new()));
}

pub(crate) async fn register(listener: EventListenerRegisterReq, funs: &TardisFunsInst) -> TardisResult<EventListenerRegisterResp> {
    if let Some(def) = DEFS.read().await.get(&listener.event_code.to_string()) {
        let sk = listener.event_sk.unwrap_or("".to_string());
        let mgr = if sk == def.use_sk {
            false
        } else if sk == def.mgr_sk {
            true
        } else {
            return Err(funs.err().unauthorized("listener", "register", "sk do not match", "401-mw-listener-sk-not-match"));
        };
        let listener_info = EventListenerInfo {
            mgr,
            channel: listener.channel,
            subscribe_mode: listener.subscribe_mode,
            token: TardisFuns::field.nanoid_len(32),
        };
        let token = listener_info.token.clone();
        LISTENERS.write().await.insert(format!("{}{}", listener.event_code, listener.listener_code), listener_info);
        if mgr {
            MGR_LISTENERS.write().await.insert(listener.event_code.to_string(), listener.listener_code.to_string());
        }
        let event_url = &funs.conf::<EventConfig>().event_url;
        Ok(EventListenerRegisterResp {
            ws_addr: Some(format!("{}/proc/ws/{}/{}?token={}", event_url, listener.event_code, listener.listener_code, token)),
            http_addr: Some(format!("{}/proc/http/{}/{}?token={}", event_url, listener.event_code, listener.listener_code, token)),
        })
    } else {
        Err(funs.err().not_found("listener", "register", "event code not found", "404-mw-event-code-not-exist"))
    }
}

pub(crate) async fn remove(event_code: &str, listener_code: &str, token: &str, funs: &TardisFunsInst) -> TardisResult<()> {
    if let Some(listener) = LISTENERS.read().await.get(&format!("{}{}", event_code, listener_code)) {
        if listener.token == token {
            LISTENERS.write().await.remove(&format!("{}{}", event_code, listener_code));
            if listener.mgr {
                MGR_LISTENERS.write().await.remove(event_code);
            }
            Ok(())
        } else {
            Err(funs.err().unauthorized("listener", "remove", "token do not match", "401-mw-listener-token-not-match"))
        }
    } else {
        Err(funs.err().not_found("listener", "remove", "listener not found", "404-mw-listener-not-exist"))
    }
}
