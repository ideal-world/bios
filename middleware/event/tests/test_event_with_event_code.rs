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

pub async fn test(http_clients: &[&TestHttpClient]) -> TardisResult<()> {
    static TEST_LOG_COUNTER: AtomicUsize = AtomicUsize::new(0);
    static FEED_FROM_MGR_COUNTER: AtomicUsize = AtomicUsize::new(0);
    static FEED_FROM_USER_COUNTER: AtomicUsize = AtomicUsize::new(0);

    prepare(http_clients).await?;

    // Register management listener
    let url = add_listener(Vec::new(), true, vec![TrimString("test_log_append")], http_clients).await?;
    let mgr_test_log_client = TardisFuns::ws_client(&url, move |msg| async move {
        let receive_msg = TardisFuns::json.str_to_obj::<TardisWebsocketMgrMessage>(msg.to_string().as_str()).unwrap();
        let ori_msg = TardisFuns::json.json_to_obj::<EventMessageMgrWrap>(receive_msg.msg).unwrap();
        let raw_msg = TardisFuns::json.json_to_obj::<EbWebsocketMessage>(ori_msg.msg).unwrap();
        if raw_msg.content == "test xxxx" {
            assert!(1 == 2);
            TEST_LOG_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        if raw_msg.content == "feed xxxx add" {
            assert!(1 == 2);
            FEED_FROM_MGR_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        if raw_msg.content == "feed yyyy add" {
            assert!(1 == 2);
            FEED_FROM_USER_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        None
    })
    .await?;
    let url = add_listener(
        Vec::new(),
        true,
        vec![TrimString("feed_add".to_string()), TrimString(WS_SYSTEM_EVENT_AVATAR_ADD.to_string())],
        http_clients,
    )
    .await?;
    let mgr_feed_client = TardisFuns::ws_client(&url, move |msg| async move {
        let receive_msg = TardisFuns::json.str_to_obj::<TardisWebsocketMgrMessage>(msg.to_string().as_str()).unwrap();
        let ori_msg = TardisFuns::json.json_to_obj::<EventMessageMgrWrap>(receive_msg.msg.clone()).unwrap();
        if receive_msg.event == Some(WS_SYSTEM_EVENT_AVATAR_ADD.to_string()) {
            return Some(Message::text(
                TardisFuns::json.obj_to_string(&receive_msg.into_req(ori_msg.msg, ori_msg.ori_from_avatar, ori_msg.ori_to_avatars)).unwrap(),
            ));
        }
        if receive_msg.event == Some("feed_add".to_string()) {
            FEED_FROM_USER_COUNTER.fetch_add(1, Ordering::SeqCst);
            return Some(Message::text(
                TardisFuns::json.obj_to_string(&receive_msg.into_req(ori_msg.msg, ori_msg.ori_from_avatar, ori_msg.ori_to_avatars)).unwrap(),
            ));
        }
        let raw_msg = TardisFuns::json.json_to_obj::<EbWebsocketMessage>(ori_msg.msg).unwrap();
        if raw_msg.content == "test xxxx" {
            assert!(1 == 2);
            TEST_LOG_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        if raw_msg.content == "feed xxxx add" {
            assert!(1 == 2);
            FEED_FROM_MGR_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        None
    })
    .await?;

    // Register user listener
    let url = add_listener(vec![TrimString("test_serv".to_string())], false, vec![], http_clients).await?;
    TardisFuns::ws_client(&url, move |msg| async move {
        let receive_msg = TardisFuns::json.str_to_obj::<TardisWebsocketMessage>(msg.to_string().as_str()).unwrap();
        let raw_msg = TardisFuns::json.json_to_obj::<EbWebsocketMessage>(receive_msg.msg).unwrap();
        if raw_msg.content == "test xxxx" {
            TEST_LOG_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        if raw_msg.content == "feed xxxx add" {
            assert!(1 == 2);
            FEED_FROM_MGR_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        if raw_msg.content == "feed yyyy add" {
            assert!(1 == 2);
            FEED_FROM_USER_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        None
    })
    .await?;

    let url = add_listener(vec![TrimString("test_serv".to_string()), TrimString("feed_serv".to_string())], false, vec![], http_clients).await?;
    TardisFuns::ws_client(&url, move |msg| async move {
        let receive_msg = TardisFuns::json.str_to_obj::<TardisWebsocketMessage>(msg.to_string().as_str()).unwrap();
        let raw_msg = TardisFuns::json.json_to_obj::<EbWebsocketMessage>(receive_msg.msg).unwrap();
        if raw_msg.content == "test xxxx" {
            TEST_LOG_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        if raw_msg.content == "feed xxxx add" {
            FEED_FROM_MGR_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        if raw_msg.content == "feed yyyy add" {
            FEED_FROM_USER_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        None
    })
    .await?;

    let url = add_listener(vec![TrimString("feed_serv".to_string())], false, vec![], http_clients).await?;
    let feed1_client = TardisFuns::ws_client(&url, move |msg| async move {
        let receive_msg = TardisFuns::json.str_to_obj::<TardisWebsocketMessage>(msg.to_string().as_str()).unwrap();
        let raw_msg = TardisFuns::json.json_to_obj::<EbWebsocketMessage>(receive_msg.msg).unwrap();
        if raw_msg.content == "test xxxx" {
            assert!(1 == 2);
            TEST_LOG_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        if raw_msg.content == "feed xxxx add" {
            FEED_FROM_MGR_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        if raw_msg.content == "feed yyyy add" {
            assert!(1 == 2);
            FEED_FROM_USER_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        None
    })
    .await?;

    let url = add_listener(vec![TrimString("others".to_string())], false, vec![], http_clients).await?;
    let feed2_client = TardisFuns::ws_client(&url, move |msg| async move {
        let receive_msg = TardisFuns::json.str_to_obj::<TardisWebsocketMessage>(msg.to_string().as_str()).unwrap();
        let raw_msg = TardisFuns::json.json_to_obj::<EbWebsocketMessage>(receive_msg.msg).unwrap();
        if raw_msg.content == "test xxxx" {
            assert!(1 == 2);
            TEST_LOG_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        if raw_msg.content == "feed xxxx add" {
            FEED_FROM_MGR_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        if raw_msg.content == "feed yyyy add" {
            FEED_FROM_USER_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        None
    })
    .await?;

    // send test log
    mgr_test_log_client
        .send_obj(&TardisWebsocketReq {
            to_avatars: Some(vec!["test_serv".to_string()]),
            event: Some("test_log_append".to_string()),
            msg: TardisFuns::json.obj_to_json(&EbWebsocketMessage { content: "test xxxx".to_string() }).unwrap(),
            ..Default::default()
        })
        .await?;

    // add new avatar
    feed2_client
        .send_obj(&TardisWebsocketReq {
            msg: json! {"feed_serv"},
            from_avatar: "others".to_string(),
            event: Some(WS_SYSTEM_EVENT_AVATAR_ADD.to_string()),
            ..Default::default()
        })
        .await?;
    sleep(Duration::from_millis(500)).await;

    // send feed from mgr
    mgr_feed_client
        .send_obj(&TardisWebsocketReq {
            to_avatars: Some(vec!["feed_serv".to_string()]),
            event: Some("feed_add".to_string()),
            msg: TardisFuns::json
                .obj_to_json(&EbWebsocketMessage {
                    content: "feed xxxx add".to_string(),
                })
                .unwrap(),
            ..Default::default()
        })
        .await?;

    // send feed from user
    feed1_client
        .send_obj(&TardisWebsocketReq {
            from_avatar: "feed_serv".to_string(),
            to_avatars: Some(vec!["feed_serv".to_string()]),
            event: Some("feed_add".to_string()),
            msg: TardisFuns::json
                .obj_to_json(&EbWebsocketMessage {
                    content: "feed yyyy add".to_string(),
                })
                .unwrap(),
            ..Default::default()
        })
        .await?;

    sleep(Duration::from_millis(500)).await;
    assert_eq!(TEST_LOG_COUNTER.load(Ordering::SeqCst), 2);
    assert_eq!(FEED_FROM_MGR_COUNTER.load(Ordering::SeqCst), 3);
    assert_eq!(FEED_FROM_USER_COUNTER.load(Ordering::SeqCst), 3);
    Ok(())
}

async fn prepare(http_clients: &[&TestHttpClient]) -> TardisResult<()> {
    let idx = rand::random::<usize>() % http_clients.len();
    let http_client = &http_clients[idx];
    let _: String = http_client
        .post(
            "/topic",
            &EventTopicAddOrModifyReq {
                code: TrimString("eb".to_string()),
                name: TrimString("事件总线".to_string()),
                save_message: true,
                need_mgr: true,
                queue_size: 1024,
                use_sk: Some("ut001".to_string()),
                mgr_sk: Some("mt001".to_string()),
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
                topic_code: TrimString("eb".to_string()),
                topic_sk: if mgr { Some("mt001".to_string()) } else { Some("ut001".to_string()) },
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
