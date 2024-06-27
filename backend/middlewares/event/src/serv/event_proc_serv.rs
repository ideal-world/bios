use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::cluster::cluster_processor::{ClusterEventTarget, TardisClusterMessageReq};
use tardis::cluster::cluster_publish::publish_event_no_response;
use tardis::futures::StreamExt;
use tardis::log::{self as tracing, instrument, warn};
use tardis::serde_json::Value;
use tardis::tokio::sync::RwLock;
use tardis::web::poem::web::websocket::{BoxWebSocketUpgraded, WebSocket};
use tardis::web::ws_processor::{ws_echo, TardisWebsocketMgrMessage, TardisWebsocketReq, TardisWebsocketResp, WsBroadcast, WsBroadcastContext, WsHooks};
use tardis::{
    cluster::{cluster_broadcast::ClusterBroadcastChannel, cluster_processor::ClusterHandler},
    tardis_static,
};
use tardis::{TardisFuns, TardisFunsInst};

use crate::dto::event_dto::EventMessageMgrWrap;
use crate::event_config::EventConfig;

use super::event_listener_serv::{listeners, mgr_listeners};
use super::event_persistent_serv::PersistentMessage;
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

pub struct CreateRemoteSenderHandler;

impl ClusterHandler for CreateRemoteSenderHandler {
    fn event_name(&self) -> String {
        "bios/event/create_remote_sender".into()
    }
    async fn handle(self: Arc<Self>, message_req: TardisClusterMessageReq) -> TardisResult<Option<Value>> {
        let CreateRemoteSenderEvent { topic_code, capacity } = TardisFuns::json.json_to_obj(message_req.msg)?;

        let clst_bc_tx = ClusterBroadcastChannel::new(topic_code.clone(), capacity);
        let mut wg = senders().write().await;
        wg.insert(topic_code, clst_bc_tx);

        Ok(None)
    }
}

pub async fn get_or_init_sender(topic_code: String, capacity: usize) -> Arc<ClusterBroadcastChannel<TardisWebsocketMgrMessage>> {
    let mut wg = senders().write().await;
    if let Some(chan) = wg.get(&topic_code) {
        tardis::log::trace!("clone existed sender: {topic_code}");
        chan.clone()
    } else {
        tardis::log::trace!("create new sender: {topic_code}:{capacity}");
        let clst_bc_tx = ClusterBroadcastChannel::new(topic_code.clone(), capacity);
        wg.insert(topic_code.clone(), clst_bc_tx.clone());
        drop(wg);
        if TardisFuns::fw_config().cluster.is_some() {
            let _ = publish_event_no_response(
                CreateRemoteSenderHandler.event_name(),
                TardisFuns::json.obj_to_json(&CreateRemoteSenderEvent { topic_code, capacity }).expect("invalid json"),
                ClusterEventTarget::Broadcast,
            )
            .await;
        }
        clst_bc_tx
    }
}

pub struct Hooks {
    persistent: bool,
    need_mgr: bool,
    topic_code: String,
    funs: Arc<TardisFunsInst>,
}

impl WsHooks for Hooks {
    async fn on_fail(&self, id: String, error: TardisError, _context: &WsBroadcastContext) {
        if self.persistent {
            let result = super::event_persistent_serv::EventPersistentServ::send_fail(id, error.to_string(), &self.funs).await;
            if let Err(error) = result {
                warn!("[Event] send fail failed: {error}");
            }
        }
    }
    async fn on_success(&self, id: String, _context: &WsBroadcastContext) {
        if self.persistent {
            let result = super::event_persistent_serv::EventPersistentServ::send_success(id, &self.funs).await;
            if let Err(error) = result {
                warn!("[Event] send fail failed: {error}");
            }
        }
    }
    #[instrument(skip(self))]
    async fn on_process(&self, req_msg: TardisWebsocketReq, context: &WsBroadcastContext) -> Option<TardisWebsocketResp> {
        if self.persistent {
            let result = super::event_persistent_serv::EventPersistentServ::save_message(
                PersistentMessage {
                    req: req_msg.clone(),
                    context: context.clone(),
                    topic: self.topic_code.clone(),
                },
                &self.funs,
            )
            .await;
            if let Err(error) = result {
                warn!("[Event] save message failed: {error}");
            }
        }
        if !self.need_mgr || context.mgr_node {
            return Some(TardisWebsocketResp {
                msg: req_msg.msg,
                to_avatars: req_msg.to_avatars.unwrap_or_default(),
                ignore_avatars: vec![],
            });
        }
        // TODO set cache
        let topic_code = self.topic_code.clone();
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
    }
}

pub(crate) async fn ws_process(listener_code: String, token: String, websocket: WebSocket, funs: TardisFunsInst) -> BoxWebSocketUpgraded {
    let Ok(Some(listener)) = listeners().get(listener_code.clone()).await else {
        return ws_error(listener_code, "listener not found", websocket);
    };
    if listener.token != token {
        return ws_error(listener_code, "permission check failed", websocket);
    }
    let Ok(Some(topic)) = topics().get(listener.topic_code.clone()).await else {
        return ws_error(listener_code, "topic not found", websocket);
    };
    let sender = get_or_init_sender(listener.topic_code.clone(), topic.queue_size as usize).await;
    tardis::log::trace!("[Bios.Event] create {topic:?} process for {token}");
    WsBroadcast::new(
        sender,
        Hooks {
            persistent: topic.save_message,
            need_mgr: topic.need_mgr,
            topic_code: listener.topic_code.clone(),
            funs: Arc::new(funs),
        },
        WsBroadcastContext::new(listener.mgr, listener.subscribe_mode),
    )
    .run(listener.avatars.clone(), websocket)
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

pub async fn scan_and_resend(funs: Arc<TardisFunsInst>) -> TardisResult<()> {
    let config = funs.conf::<EventConfig>();
    let threshold = config.resend_threshold;
    let scanner = super::event_persistent_serv::EventPersistentServ::scan_failed(&funs, threshold as i32).await?;
    let mut scanner = std::pin::pin!(scanner);
    while let Some(PersistentMessage { req, context, topic }) = scanner.next().await {
        let funs = funs.clone();
        tardis::tokio::spawn(async move {
            let Some(id) = req.msg_id.clone() else {
                warn!("[Bios.Event] msg_id not found: {req:?}");
                return Ok(());
            };
            let Ok(Some(topic_resp)) = topics().get(topic.clone()).await else {
                warn!("[Bios.Event] topic not found: {topic}");
                return Ok(());
            };
            let sender = get_or_init_sender(topic_resp.code.clone(), topic_resp.queue_size as usize).await;
            if !super::event_persistent_serv::EventPersistentServ::sending(id.clone(), &funs).await? {
                // state conflict
                return TardisResult::Ok(());
            }
            let broadcast = WsBroadcast::new(
                sender,
                Hooks {
                    persistent: topic_resp.save_message,
                    need_mgr: topic_resp.need_mgr,
                    topic_code: topic_resp.code,
                    funs: funs.clone(),
                },
                context,
            );
            let resend_result = broadcast.handle_req(req).await;
            if let Err(error) = resend_result {
                super::event_persistent_serv::EventPersistentServ::send_fail(id, error, &funs).await?;
            }
            TardisResult::Ok(())
        });
    }
    Ok(())
}
