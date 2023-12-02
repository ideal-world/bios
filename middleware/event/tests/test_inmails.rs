use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use bios_basic::test::test_http_client::TestHttpClient;
use bios_mw_event::dto::event_dto::{EventListenerRegisterReq, EventListenerRegisterResp, EventMessageMgrWrap, EventTopicAddOrModifyReq};
use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::serde_json::json;
use tardis::tokio::time::sleep;
use tardis::web::tokio_tungstenite::tungstenite::Message;
use tardis::web::ws_processor::{TardisWebsocketMessage, TardisWebsocketMgrMessage, TardisWebsocketReq, WS_SYSTEM_EVENT_AVATAR_ADD};
use tardis::{rand, TardisFuns};

pub const USR_SK: &str = "ut001";
pub const MGR_SK: &str = "mt001";
pub const TOPIC_CODE: &str = "testinmails";

pub async fn test(http_clients: &[&TestHttpClient]) -> TardisResult<()> {
    prepare(http_clients).await?;

    let url = add_listener(vec!["inmails/server/notify".into()], true, vec!["todo".into()], http_clients).await?;
    TardisFuns::ws_client(&url, |message| async move {
        let message = message.to_text().expect("expect message");
        dbg!(&message);
        let message = TardisFuns::json.str_to_obj::<TardisWebsocketMessage>(message).expect("invalid message");
        if let Some("notify") = message.event.as_deref() {
            let message = TardisFuns::json.json_to_obj::<InMails>(message.msg.clone()).unwrap();
            match message {
                InMails::Push { message } => {
                    println!("push: {}", message);
                }
                InMails::Dm { message, target } => {
                    println!("dm: {} -> {}", message, target);
                }
            }
        }
        None
    })
    .await?;

    let url = add_listener(vec!["inmails/client/user1".into()], false, vec!["todo".into()], http_clients).await?;
    let client1 = TardisFuns::ws_client(&url, |message| async move {
        dbg!(message);
        None
    })
    .await?;

    let url = add_listener(vec!["inmails/client/user2".into()], false, vec!["todo".into()], http_clients).await?;
    let client2 = TardisFuns::ws_client(&url, |message| async move {
        dbg!(message);
        None
    })
    .await?;

    client2
        .send_obj(&TardisWebsocketReq {
            to_avatars: Some(vec!["inmails/server/notify".into()]),
            event: Some("notify".to_string()),
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
                topic_sk: if mgr { Some(USR_SK.to_string()) } else { Some(MGR_SK.to_string()) },
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
    Push { message: String },
    Dm { message: String, target: String },
}
