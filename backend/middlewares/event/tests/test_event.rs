use std::env;
use std::time::Duration;

use asteroid_mq::prelude::{Interest, Subject, TopicCode};
use asteroid_mq_sdk::model::EdgeMessage;
use asteroid_mq_sdk::ClientNode;
use bios_basic::rbum::rbum_config::RbumConfig;
use bios_basic::test::init_test_container;
use bios_basic::test::test_http_client::TestHttpClient;
use bios_mw_event::event_constants::DOMAIN_CODE;
use bios_mw_event::event_initializer;
use tardis::basic::dto::TardisContext;
use tardis::serde_json::{json, Value};
use tardis::web::web_resp::Void;
use tardis::{log as tracing, TardisFunsInst};
use tardis::{tardis_static, tokio, TardisFuns};
use tokio::io::AsyncReadExt;
#[tokio::test(flavor = "multi_thread")]
async fn test_event() -> Result<(), Box<dyn std::error::Error>> {
    // env::set_var("RUST_LOG", "debug,tardis=trace,bios_mw_event=trace,test_event=trace,sqlx::query=off,asteroid-mq=info");

    let _x = init_test_container::init(None).await?;

    init_data().await?;
    // tokio::io::stdin().read_buf(&mut Vec::new()).await?;
    test_event_topic_api().await?;
    Ok(())
}

async fn init_data() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize RBUM
    bios_basic::rbum::rbum_initializer::init(DOMAIN_CODE, RbumConfig::default()).await?;
    let mut funs = TardisFuns::inst(DOMAIN_CODE.to_string(), None);
    let web_server = TardisFuns::web_server();
    // Initialize Event
    event_initializer::init(web_server.as_ref()).await?;
    web_server.start().await?;

    let ctx = TardisContext {
        own_paths: "test".to_string(),
        ak: "test".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "test".to_string(),
        ..Default::default()
    };
    bios_sdk_invoke::clients::event_client::init_ws_client_node(None, Duration::from_secs(1), &ctx, &funs).await;
    let mut client = TestHttpClient::new(format!("http://127.0.0.1:8080/{}", DOMAIN_CODE));
    let auth = ctx.to_base64()?;
    tracing::info!(?auth, "auth");
    client.set_auth(&ctx)?;
    tokio::time::sleep(Duration::from_secs(5)).await;
    Ok(())
}

tardis_static! {
    pub test_tardis_context: TardisContext = TardisContext {
        own_paths: "test".to_string(),
        ak: "test".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "test-owner".to_string(),
        ..Default::default()
    };
}

pub async fn test_event_topic_api() -> Result<(), Box<dyn std::error::Error>> {
    use bios_sdk_invoke::clients::event_client::{EventClient, EventTopicConfig};
    let mut funs = TardisFuns::inst(DOMAIN_CODE.to_string(), None);
    const TEST_TOPIC_NAME: &str = "test-topic";
    let mut client = TestHttpClient::new(format!("http://127.0.0.1:8080/{}", DOMAIN_CODE));
    let ctx = test_tardis_context();
    let topic_id = EventClient::create_topic(
        &EventTopicConfig {
            topic_code: TEST_TOPIC_NAME.to_string(),
            overflow_policy: Some("RejectNew".to_string()),
            overflow_size: 500,
            check_auth: true,
            blocking: false,
        },
        ctx,
        &funs,
    )
    .await?;
    tracing::info!(?topic_id, "event registered");
    tokio::time::sleep(Duration::from_secs(1)).await;
    let existed = EventClient::check_topic_exist(TEST_TOPIC_NAME, ctx, &funs).await?;
    tracing::info!(?existed, "topic existed");

    let register_auth_result = EventClient::register_user(TEST_TOPIC_NAME, true, true, ctx, &funs).await;
    tracing::info!(?register_auth_result, "auth registered");
    // tracing::info!("bind context result");
    // let bind_result = client.put::<Void, Value>("/ca/register", &Void).await;
    // let node_id = bind_result["node_id"].as_str().expect("node_id is settled");

    let client_node = bios_sdk_invoke::clients::event_client::mq_client_node();
    const TOPIC_CODE: TopicCode = TopicCode::const_new(TEST_TOPIC_NAME);
    let mut ep = client_node.create_endpoint(TopicCode::const_new(TEST_TOPIC_NAME), [Interest::new("test_node")]).await?;
    tokio::spawn(async move {
        while let Some(message) = ep.next_message().await {
            tracing::info!(payload = ?message.text().unwrap(), "received message");
            let _ = message.ack_processed().await;
        }
    });
    let message = EdgeMessage::builder(TOPIC_CODE, [Subject::const_new("test_node")], "test message").build();
    let _ack = client_node.send_message(message).await?;

    tokio::time::sleep(Duration::from_secs(5)).await;

    Ok(())
}
