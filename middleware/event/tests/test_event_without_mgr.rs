use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use bios_basic::test::test_http_client::TestHttpClient;
use bios_mw_event::dto::event_dto::{EventListenerRegisterReq, EventListenerRegisterResp, EventTopicAddOrModifyReq};
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::serde_json::json;
use tardis::tokio::time::sleep;
use tardis::web::ws_processor::{TardisWebsocketMessage, TardisWebsocketReq};
use tardis::TardisFuns;

pub async fn test(http_client: &TestHttpClient) -> TardisResult<()> {
    static NOTIFY_COUNTER: AtomicUsize = AtomicUsize::new(0);

    prepare(http_client).await?;

    // Register user listener
    let url = add_listener(vec![TrimString("doc001".to_string())], http_client).await?;
    TardisFuns::ws_client(&url, move |msg| async move {
        let receive_msg = TardisFuns::json.str_to_obj::<TardisWebsocketMessage>(&msg).unwrap();
        if receive_msg.msg.get("block001").is_some() && receive_msg.msg.get("block001").unwrap().as_str().unwrap() == "xnfeonfd" {
            NOTIFY_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        if receive_msg.msg.get("block002").is_some() && receive_msg.msg.get("block002").unwrap().as_str().unwrap() == "哈哈" {
            NOTIFY_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        None
    })
    .await?;

    let url = add_listener(vec![TrimString("doc001".to_string())], http_client).await?;
    TardisFuns::ws_client(&url, move |msg| async move {
        let receive_msg = TardisFuns::json.str_to_obj::<TardisWebsocketMessage>(&msg).unwrap();
        if receive_msg.msg.get("block001").is_some() && receive_msg.msg.get("block001").unwrap().as_str().unwrap() == "xnfeonfd" {
            NOTIFY_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        if receive_msg.msg.get("block002").is_some() && receive_msg.msg.get("block002").unwrap().as_str().unwrap() == "哈哈" {
            assert!(1 == 2);
            NOTIFY_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        Some(
            TardisFuns::json
                .obj_to_string(&TardisWebsocketReq {
                    msg: json!({
                        "block002": "哈哈"
                    }),
                    from_avatar: "doc001".to_string(),
                    ..Default::default()
                })
                .unwrap(),
        )
    })
    .await?;

    let url = add_listener(vec![TrimString("doc001".to_string())], http_client).await?;
    let doc01_client = TardisFuns::ws_client(&url, move |msg| async move {
        let receive_msg = TardisFuns::json.str_to_obj::<TardisWebsocketMessage>(&msg).unwrap();
        if receive_msg.msg.get("block001").is_some() && receive_msg.msg.get("block001").unwrap().as_str().unwrap() == "xnfeonfd" {
            assert!(1 == 2);
            NOTIFY_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        if receive_msg.msg.get("block002").is_some() && receive_msg.msg.get("block002").unwrap().as_str().unwrap() == "哈哈" {
            NOTIFY_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        None
    })
    .await?;

    // notify
    doc01_client
        .send_obj(&TardisWebsocketReq {
            msg: json!({
                "block001": "xnfeonfd"
            }),
            from_avatar: "doc001".to_string(),
            ..Default::default()
        })
        .await?;

    sleep(Duration::from_millis(500)).await;
    assert_eq!(NOTIFY_COUNTER.load(Ordering::SeqCst), 4);
    Ok(())
}

async fn prepare(http_client: &TestHttpClient) -> TardisResult<()> {
    let _: String = http_client
        .post(
            "/topic",
            &EventTopicAddOrModifyReq {
                code: TrimString("coo".to_string()),
                name: TrimString("协作".to_string()),
                save_message: false,
                need_mgr: false,
                queue_size: 1024,
                use_sk: Some("ut001".to_string()),
                mgr_sk: None,
            },
        )
        .await;
    Ok(())
}

async fn add_listener(avatars: Vec<TrimString>, http_client: &TestHttpClient) -> TardisResult<String> {
    let resp: EventListenerRegisterResp = http_client
        .post(
            "/listener",
            &EventListenerRegisterReq {
                topic_code: TrimString("coo".to_string()),
                topic_sk: Some("ut001".to_string()),
                events: None,
                avatars: avatars,
                subscribe_mode: true,
            },
        )
        .await;
    Ok(resp.ws_addr)
}
