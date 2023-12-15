use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use bios_basic::test::test_http_client::TestHttpClient;
use bios_mw_event::dto::event_dto::{EventListenerRegisterReq, EventListenerRegisterResp, EventMessageMgrWrap, EventTopicAddOrModifyReq};
use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::serde_json::json;
use tardis::tokio::time::sleep;
use tardis::web::tokio_tungstenite;
use tardis::web::tokio_tungstenite::tungstenite::Message;
use tardis::web::ws_processor::{TardisWebsocketMessage, TardisWebsocketMgrMessage, TardisWebsocketReq, WS_SYSTEM_EVENT_AVATAR_ADD};
use tardis::{futures, rand, tardis_static, TardisFuns};
use tokio::sync::Mutex;

pub const USR_SK: &str = "ut001";
pub const MGR_SK: &str = "mt001";
pub const TOPIC_CODE: &str = "testinmails";

tardis_static! {
    pub mailbox_alice: Arc<Mutex<Vec<TardisWebsocketMessage>>>;
    pub mailbox_bob: Arc<Mutex<Vec<TardisWebsocketMessage>>>;
    pub mailbox_clancy: Arc<Mutex<Vec<TardisWebsocketMessage>>>;
}

pub enum Action<'a> {
    Dm { peer: &'a str },
    Push { topic: &'a str },
    Broadcast {},
}

pub fn auth_action(from: &str, action: Action) -> bool {
    match from {
        "inmails/client/Alice" => match action {
            Action::Dm { peer } => peer == "inmails/client/Bob",
            Action::Push { topic } => topic == "publish",
            Action::Broadcast {} => true,
        },
        "inmails/client/Bob" => match action {
            Action::Dm { peer } => peer == "inmails/client/Alice" || peer == "inmails/client/Clancy",
            Action::Push { topic } => topic == "publish" || topic == "warning",
            Action::Broadcast {} => false,
        },
        "inmails/client/Clancy" => match action {
            Action::Dm { peer } => peer == "inmails/client/Bob",
            Action::Push { topic } => topic == "publish" || topic == "warning",
            Action::Broadcast {} => false,
        },
        _ => false,
    }
}

pub fn mgr_on_message(message: Message) -> Option<Message> {
    let receive_msg = TardisFuns::json.str_to_obj::<TardisWebsocketMgrMessage>(message.to_string().as_str()).unwrap();
    dbg!(&receive_msg);
    let mut ori_msg = TardisFuns::json.json_to_obj::<EventMessageMgrWrap>(receive_msg.msg.clone()).unwrap();
    let from = ori_msg.ori_from_avatar.as_str();
    let Some(to) = ori_msg.ori_to_avatars.as_ref().and_then(|v| v.first()) else {
        return None;
    };
    let action = match receive_msg.event.as_deref() {
        Some("dm") => Action::Dm { peer: to },
        Some("push") => Action::Push { topic: to },
        Some("broadcast") => {
            ori_msg.ori_to_avatars = Some(vec!["inmails/broadcast".into()]);
            Action::Broadcast {}
        }
        Some(_) => {
            unreachable!("unknown event");
        }
        None => {
            return None;
        }
    };
    if auth_action(from, action) {
        return Some(Message::text(
            TardisFuns::json.obj_to_string(&receive_msg.into_req(ori_msg.msg, ori_msg.ori_from_avatar, ori_msg.ori_to_avatars)).unwrap(),
        ));
    }
    None
}
pub async fn test(http_clients: &[&TestHttpClient]) -> TardisResult<()> {
    prepare(http_clients).await?;
    // mgr_node 1
    let url = add_listener(vec![], true, vec!["dm".into(), "push".into(), "broadcast".into()], http_clients).await?;
    TardisFuns::ws_client(&url, |message| async move { mgr_on_message(message) }).await?;

    // mgr node 2
    let url = add_listener(vec![], true, vec!["todo".into()], http_clients).await?;
    TardisFuns::ws_client(&url, |message| async move { mgr_on_message(message) }).await?;

    let url = add_listener(
        vec!["inmails/client/alice".into()],
        false,
        vec!["dm".into(), "push".into(), "broadcast".into()],
        http_clients,
    )
    .await?;
    let client_alice = TardisFuns::ws_client(&url, |message| async move {
        info!("client alice receive message {message}");
        mailbox_alice().lock().await.push(TardisFuns::json.str_to_obj::<TardisWebsocketMessage>(message.to_string().as_str()).unwrap());
        None
    })
    .await?;

    let url = add_listener(
        vec!["inmails/client/bob".into()],
        false,
        vec!["dm".into(), "push".into(), "broadcast".into()],
        http_clients,
    )
    .await?;
    let client_bob = TardisFuns::ws_client(&url, |message| async move {
        info!("client bob receive message {message}");
        mailbox_bob().lock().await.push(TardisFuns::json.str_to_obj::<TardisWebsocketMessage>(message.to_string().as_str()).unwrap());
        None
    })
    .await?;

    let url = add_listener(
        vec!["inmails/client/clancy".into(), "inmails/broadcast".into()],
        false,
        vec!["dm".into(), "push".into(), "broadcast".into()],
        http_clients,
    )
    .await?;
    let client_clancy = TardisFuns::ws_client(&url, |message| async move {
        info!("client clancy receive message {message}");
        mailbox_clancy().lock().await.push(TardisFuns::json.str_to_obj::<TardisWebsocketMessage>(message.to_string().as_str()).unwrap());
        None
    })
    .await?;

    client_bob
        .send_obj(&TardisWebsocketReq {
            to_avatars: Some(vec!["inmails/client/alice".into()]),
            event: Some("dm".into()),
            msg: TardisFuns::json.obj_to_json(&"hello bob").unwrap(),
            from_avatar: "inmails/client/bob".into(),
            ..Default::default()
        })
        .await?;
    tokio::time::sleep(Duration::from_secs(1)).await;
    info!("{:?}", mailbox_alice().lock().await.pop());
    info!("{:?}", mailbox_alice().lock().await.pop());

    // send to self should be none
    client_bob
        .send_obj(&TardisWebsocketReq {
            to_avatars: Some(vec!["inmails/client/bob".into()]),
            event: Some("dm".into()),
            msg: TardisFuns::json
                .obj_to_json(&InMails::Dm {
                    message: "halo".into(),
                    target: "uno".into(),
                })
                .unwrap(),
            from_avatar: "inmails/client/bob".into(),
            ..Default::default()
        })
        .await?;
    tokio::time::sleep(Duration::from_secs(1)).await;
    assert!(mailbox_bob().lock().await.pop().is_none());

    // bob has no permission to send to a broadcast topic
    client_bob
        .send_obj(&TardisWebsocketReq {
            to_avatars: None,
            event: Some("broadcast".into()),
            msg: json!("broadcast from bob"),
            from_avatar: "inmails/client/bob".into(),
            ..Default::default()
        })
        .await?;
    tokio::time::sleep(Duration::from_secs(1)).await;
    assert!(mailbox_clancy().lock().await.pop().is_none());

    // alice has permission to send to a broadcast topic
    client_alice
        .send_obj(&TardisWebsocketReq {
            to_avatars: None,
            event: Some("broadcast".into()),
            msg: json!("broadcast from alice"),
            from_avatar: "inmails/client/alice".into(),
            ..Default::default()
        })
        .await?;
    tokio::time::sleep(Duration::from_secs(1)).await;
    assert!(mailbox_clancy().lock().await.pop().is_none());
    Ok(())
}

async fn prepare(http_clients: &[&TestHttpClient]) -> TardisResult<()> {
    let idx = rand::random::<usize>() % http_clients.len();
    let http_client = &http_clients[idx];
    let _: String = http_client
        .post(
            "/topic",
            &EventTopicAddOrModifyReq {
                code: TrimString(TOPIC_CODE),
                name: TrimString("站内信".to_string()),
                save_message: true,
                need_mgr: true,
                queue_size: 1024,
                use_sk: Some(USR_SK.to_string()),
                mgr_sk: Some(MGR_SK.to_string()),
            },
        )
        .await;
    Ok(())
}

async fn add_listener(avatars: Vec<TrimString>, mgr: bool, events: Vec<TrimString>, http_clients: &[&TestHttpClient]) -> TardisResult<String> {
    let idx = rand::random::<usize>() % http_clients.len();
    let http_client = &http_clients[idx];
    let resp: EventListenerRegisterResp = http_client
        .post(
            "/listener",
            &EventListenerRegisterReq {
                topic_code: TrimString(TOPIC_CODE),
                topic_sk: if mgr { Some(MGR_SK.to_string()) } else { Some(USR_SK.to_string()) },
                events: if events.is_empty() { None } else { Some(events) },
                avatars,
                subscribe_mode: !mgr,
            },
        )
        .await;
    Ok(resp.ws_addr)
}

#[derive(Deserialize, Serialize, Clone, Debug)]
struct EbWebsocketMessage {
    pub content: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum InMails {
    Subscribe { topic: String },
    Publish { topic: String, message: String },
    Dm { message: String, target: String },
}
