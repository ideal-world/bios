use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use bios_basic::test::test_http_client::TestHttpClient;
use bios_mw_event::dto::event_dto::{EventListenerRegisterReq, EventListenerRegisterResp, EventMessageMgrWrap, EventTopicAddOrModifyReq};
use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::tokio::time::sleep;
use tardis::web::tokio_tungstenite::tungstenite::Message;
use tardis::web::ws_processor::{TardisWebsocketMessage, TardisWebsocketMgrMessage, TardisWebsocketReq};
use tardis::{rand, TardisFuns};

pub async fn test(http_clients: &[&TestHttpClient]) -> TardisResult<()> {
    static NOTIFY_COUNTER: AtomicUsize = AtomicUsize::new(0);
    static U1_TO_U2_COUNTER: AtomicUsize = AtomicUsize::new(0);
    static U2_TO_U1_COUNTER: AtomicUsize = AtomicUsize::new(0);
    static TO_G1_COUNTER: AtomicUsize = AtomicUsize::new(0);

    prepare(http_clients).await?;

    // Register management listener
    let url = add_listener(Vec::new(), true, http_clients).await?;
    let mgr_client = TardisFuns::ws_client(&url, move |msg| async move {
        let receive_msg = TardisFuns::json.str_to_obj::<TardisWebsocketMgrMessage>(msg.to_string().as_str()).unwrap();
        let msg_id = receive_msg.msg_id.clone();

        let ori_msg = TardisFuns::json.json_to_obj::<EventMessageMgrWrap>(receive_msg.msg.clone()).unwrap();
        let raw_msg = TardisFuns::json.json_to_obj::<ImWebsocketMessage>(ori_msg.msg.clone()).unwrap();
        if raw_msg.content == "系统升级" {
            assert!(1 == 2);
            NOTIFY_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        if raw_msg.content == "你好" {
            U1_TO_U2_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        if raw_msg.content == "^_^" {
            U2_TO_U1_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        if raw_msg.content == "Hi" {
            TO_G1_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        Some(Message::text(
            TardisFuns::json.obj_to_string(&receive_msg.into_req(msg_id, ori_msg.msg, ori_msg.ori_from_avatar, ori_msg.ori_to_avatars)).unwrap(),
        ))
    })
    .await?;
    // Register user listener
    let url = add_listener(vec![TrimString("user01".to_string()), TrimString("group01".to_string())], false, http_clients).await?;
    let user01_client = TardisFuns::ws_client(&url, move |msg| async move {
        let receive_msg = TardisFuns::json.str_to_obj::<TardisWebsocketMessage>(msg.to_string().as_str()).unwrap();
        let raw_msg = TardisFuns::json.json_to_obj::<ImWebsocketMessage>(receive_msg.msg).unwrap();
        if raw_msg.content == "系统升级" {
            NOTIFY_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        if raw_msg.content == "你好" {
            assert!(1 == 2);
            U1_TO_U2_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        if raw_msg.content == "^_^" {
            U2_TO_U1_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        if raw_msg.content == "Hi" {
            TO_G1_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        None
    })
    .await?;

    let url = add_listener(vec![TrimString("user02".to_string())], false, http_clients).await?;
    let user02_client = TardisFuns::ws_client(&url, move |msg| async move {
        let receive_msg = TardisFuns::json.str_to_obj::<TardisWebsocketMessage>(msg.to_string().as_str()).unwrap();
        let raw_msg = TardisFuns::json.json_to_obj::<ImWebsocketMessage>(receive_msg.msg).unwrap();
        if raw_msg.content == "系统升级" {
            NOTIFY_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        if raw_msg.content == "你好" {
            U1_TO_U2_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        if raw_msg.content == "^_^" {
            assert!(1 == 2);
            U2_TO_U1_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        if raw_msg.content == "Hi" {
            assert!(1 == 2);
            TO_G1_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        None
    })
    .await?;

    let url = add_listener(vec![TrimString("user03".to_string()), TrimString("group01".to_string())], false, http_clients).await?;
    let user03_client = TardisFuns::ws_client(&url, move |msg| async move {
        let receive_msg = TardisFuns::json.str_to_obj::<TardisWebsocketMessage>(msg.to_string().as_str()).unwrap();
        let raw_msg = TardisFuns::json.json_to_obj::<ImWebsocketMessage>(receive_msg.msg).unwrap();
        if raw_msg.content == "系统升级" {
            NOTIFY_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        if raw_msg.content == "你好" {
            assert!(1 == 2);
            U1_TO_U2_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        if raw_msg.content == "^_^" {
            assert!(1 == 2);
            U2_TO_U1_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        if raw_msg.content == "Hi" {
            assert!(1 == 2);
            TO_G1_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        None
    })
    .await?;

    let url = add_listener(vec![TrimString("user04".to_string()), TrimString("group01".to_string())], false, http_clients).await?;
    TardisFuns::ws_client(&url, move |msg| async move {
        let receive_msg = TardisFuns::json.str_to_obj::<TardisWebsocketMessage>(msg.to_string().as_str()).unwrap();
        let raw_msg = TardisFuns::json.json_to_obj::<ImWebsocketMessage>(receive_msg.msg).unwrap();
        if raw_msg.content == "系统升级" {
            NOTIFY_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        if raw_msg.content == "你好" {
            assert!(1 == 2);
            U1_TO_U2_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        if raw_msg.content == "^_^" {
            assert!(1 == 2);
            U2_TO_U1_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        if raw_msg.content == "Hi" {
            TO_G1_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        None
    })
    .await?;
    let url = add_listener(vec![TrimString("user04".to_string()), TrimString("group01".to_string())], false, http_clients).await?;
    TardisFuns::ws_client(&url, move |msg| async move {
        let receive_msg = TardisFuns::json.str_to_obj::<TardisWebsocketMessage>(msg.to_string().as_str()).unwrap();
        let raw_msg = TardisFuns::json.json_to_obj::<ImWebsocketMessage>(receive_msg.msg).unwrap();
        if raw_msg.content == "系统升级" {
            NOTIFY_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        if raw_msg.content == "你好" {
            assert!(1 == 2);
            U1_TO_U2_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        if raw_msg.content == "^_^" {
            assert!(1 == 2);
            U2_TO_U1_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        if raw_msg.content == "Hi" {
            TO_G1_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        None
    })
    .await?;

    // notify
    mgr_client
        .send_obj(&TardisWebsocketReq {
            msg: TardisFuns::json
                .obj_to_json(&ImWebsocketMessage {
                    from: "管理员".to_string(),
                    content: "系统升级".to_string(),
                })
                .unwrap(),
            from_avatar: "_".to_string(),
            to_avatars: None,
            ..Default::default()
        })
        .await?;

    // user01 send to user02
    user01_client
        .send_obj(&TardisWebsocketReq {
            msg: TardisFuns::json
                .obj_to_json(&ImWebsocketMessage {
                    from: "用户1".to_string(),
                    content: "你好".to_string(),
                })
                .unwrap(),
            from_avatar: "user01".to_string(),
            to_avatars: Some(vec!["user02".to_string()]),
            ..Default::default()
        })
        .await?;

    // user02 send to user01
    user02_client
        .send_obj(&TardisWebsocketReq {
            msg: TardisFuns::json
                .obj_to_json(&ImWebsocketMessage {
                    from: "用户2".to_string(),
                    content: "^_^".to_string(),
                })
                .unwrap(),
            from_avatar: "user02".to_string(),
            to_avatars: Some(vec!["user01".to_string()]),
            ..Default::default()
        })
        .await?;

    // user03 send to group01
    user03_client
        .send_obj(&TardisWebsocketReq {
            msg: TardisFuns::json
                .obj_to_json(&ImWebsocketMessage {
                    from: "群组1".to_string(),
                    content: "Hi".to_string(),
                })
                .unwrap(),
            from_avatar: "user03".to_string(),
            to_avatars: Some(vec!["group01".to_string()]),
            ..Default::default()
        })
        .await?;

    sleep(Duration::from_millis(500)).await;
    assert_eq!(NOTIFY_COUNTER.load(Ordering::SeqCst), 5);
    assert_eq!(U1_TO_U2_COUNTER.load(Ordering::SeqCst), 2);
    assert_eq!(U2_TO_U1_COUNTER.load(Ordering::SeqCst), 2);
    assert_eq!(TO_G1_COUNTER.load(Ordering::SeqCst), 4);
    Ok(())
}

async fn prepare(http_clients: &[&TestHttpClient]) -> TardisResult<()> {
    let idx = rand::random::<usize>() % http_clients.len();
    let http_client = &http_clients[idx];
    let _: String = http_client
        .post(
            "/topic",
            &EventTopicAddOrModifyReq {
                code: TrimString("im".to_string()),
                name: TrimString("即时通讯".to_string()),
                save_message: false,
                need_mgr: true,
                queue_size: 1024,
                use_sk: Some("ut001".to_string()),
                mgr_sk: Some("mt001".to_string()),
            },
        )
        .await;
    Ok(())
}

async fn add_listener(avatars: Vec<TrimString>, mgr: bool, http_clients: &[&TestHttpClient]) -> TardisResult<String> {
    let idx = rand::random::<usize>() % http_clients.len();
    let http_client = &http_clients[idx];
    let resp: EventListenerRegisterResp = http_client
        .post(
            "/listener",
            &EventListenerRegisterReq {
                topic_code: TrimString("im".to_string()),
                topic_sk: if mgr { Some("mt001".to_string()) } else { Some("ut001".to_string()) },
                events: None,
                avatars,
                subscribe_mode: !mgr,
            },
        )
        .await;
    Ok(resp.ws_addr)
}

#[derive(Deserialize, Serialize, Clone, Debug)]
struct ImWebsocketMessage {
    pub from: String,
    pub content: String,
}
