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
    pub client_1_mailbox: Arc<Mutex<Vec<TardisWebsocketMessage>>>;
    pub client_2_mailbox: Arc<Mutex<Vec<TardisWebsocketMessage>>>;
}

pub async fn test(http_clients: &[&TestHttpClient]) -> TardisResult<()> {
    prepare(http_clients).await?;

    let url = add_listener(vec![], true, vec!["todo".into()], http_clients).await?;
    TardisFuns::ws_client(&url, |message| async move {
        let receive_msg = TardisFuns::json.str_to_obj::<TardisWebsocketMgrMessage>(message.to_string().as_str()).unwrap();
        dbg!(&receive_msg);
        let ori_msg = TardisFuns::json.json_to_obj::<EventMessageMgrWrap>(receive_msg.msg.clone()).unwrap();
        let raw_msg = TardisFuns::json.json_to_obj::<InMails>(ori_msg.msg.clone()).unwrap();
        dbg!(&raw_msg);
        if let Some("todo") = receive_msg.event.as_deref() {
            match raw_msg {
                InMails::Subscribe { topic } => {
                    println!("topic: {}", topic);
                    return Some(Message::text(
                        TardisFuns::json.obj_to_string(&receive_msg.into_req(ori_msg.msg, ori_msg.ori_from_avatar, ori_msg.ori_to_avatars)).unwrap(),
                    ));
                }
                InMails::Dm { message, target } => {
                    println!("dm: {} -> {}", message, target);
                    return Some(Message::text(
                        TardisFuns::json.obj_to_string(&receive_msg.into_req(ori_msg.msg, ori_msg.ori_from_avatar, ori_msg.ori_to_avatars)).unwrap(),
                    ));
                }
                InMails::Publish { topic, message } => {
                    println!("publish: {} -> {}", topic, message);
                    return None;
                }
            }
        }
        None
    })
    .await?;

    // mgr node 2
    let url = add_listener(vec![], true, vec!["todo".into()], http_clients).await?;
    TardisFuns::ws_client(&url, |message| async move {
        let receive_msg = TardisFuns::json.str_to_obj::<TardisWebsocketMgrMessage>(message.to_string().as_str()).unwrap();
        dbg!(&receive_msg);
        let ori_msg = TardisFuns::json.json_to_obj::<EventMessageMgrWrap>(receive_msg.msg.clone()).unwrap();
        let raw_msg = TardisFuns::json.json_to_obj::<InMails>(ori_msg.msg.clone()).unwrap();
        dbg!(&raw_msg);
        if let Some("todo") = receive_msg.event.as_deref() {
            match raw_msg {
                InMails::Subscribe { topic: message } => {
                    println!("push: {}", message);
                    return None;
                }
                InMails::Dm { message, target } => {
                    println!("dm: {} -> {}", message, target);
                    return Some(Message::text(
                        TardisFuns::json.obj_to_string(&receive_msg.into_req(ori_msg.msg, ori_msg.ori_from_avatar, ori_msg.ori_to_avatars)).unwrap(),
                    ));
                }
                InMails::Publish { topic, message } => {
                    println!("publish: {} -> {}", topic, message);
                    return None;
                }
            }
        }
        return None;
    })
    .await?;

    let url = add_listener(vec!["inmails/client/user1".into()], false, vec!["todo".into()], http_clients).await?;
    let client1 = TardisFuns::ws_client(&url, |message| async move {
        info!("client 1 receive message {message}");
        client_1_mailbox().lock().await.push(TardisFuns::json.str_to_obj::<TardisWebsocketMessage>(message.to_string().as_str()).unwrap());
        None
    })
    .await?;

    let url = add_listener(vec!["inmails/client/user2".into()], false, vec!["todo".into()], http_clients).await?;
    let client2 = TardisFuns::ws_client(&url, |message| async move {
        info!("client 2 receive message {message}");
        client_2_mailbox().lock().await.push(TardisFuns::json.str_to_obj::<TardisWebsocketMessage>(message.to_string().as_str()).unwrap());
        None
    })
    .await?;

    client2
        .send_obj(&TardisWebsocketReq {
            to_avatars: Some(vec!["inmails/client/user1".into()]),
            event: Some("todo".into()),
            msg: TardisFuns::json
                .obj_to_json(&InMails::Dm {
                    message: "halo".into(),
                    target: "uno".into(),
                })
                .unwrap(),
            from_avatar: "inmails/client/user2".into(),
            ..Default::default()
        })
        .await?;
    tokio::time::sleep(Duration::from_secs(1)).await;
    info!("{:?}", client_1_mailbox().lock().await.pop());
    info!("{:?}", client_1_mailbox().lock().await.pop());

    // send to self should be none
    client2
        .send_obj(&TardisWebsocketReq {
            to_avatars: Some(vec!["inmails/client/user2".into()]),
            event: Some("todo".into()),
            msg: TardisFuns::json
                .obj_to_json(&InMails::Dm {
                    message: "halo".into(),
                    target: "uno".into(),
                })
                .unwrap(),
            from_avatar: "inmails/client/user2".into(),
            ..Default::default()
        })
        .await?;
    assert!(client_2_mailbox().lock().await.pop().is_none());
    tokio::time::sleep(Duration::from_secs(1)).await;

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
