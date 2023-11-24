use std::env;
use std::time::Duration;

use bios_basic::rbum::rbum_config::RbumConfig;
use bios_basic::test::init_rbum_test_container;
use bios_basic::test::test_http_client::TestHttpClient;
use bios_mw_event::event_constants::DOMAIN_CODE;
use bios_mw_event::event_initializer;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::tokio::time::sleep;
use tardis::{testcontainers, tokio, TardisFuns};
mod test_event_with_event_code;
mod test_event_with_im;
mod test_event_without_mgr;

#[tokio::test(flavor = "multi_thread")]
async fn test_event() -> TardisResult<()> {
    env::set_var("RUST_LOG", "debug,tardis=trace,bios_mw_event=trace,test_event=trace,sqlx::query=off");

    let docker = testcontainers::clients::Cli::default();
    let _x = init_rbum_test_container::init(&docker, None).await?;

    init_data().await?;

    Ok(())
}

async fn init_data() -> TardisResult<()> {
    // Initialize RBUM
    bios_basic::rbum::rbum_initializer::init(DOMAIN_CODE, RbumConfig::default()).await?;

    let web_server = TardisFuns::web_server();
    // Initialize Event
    event_initializer::init(web_server.as_ref()).await.unwrap();
    tokio::spawn(async move {
        web_server.start().await.unwrap();
    });

    sleep(Duration::from_millis(500)).await;

    let ctx = TardisContext {
        own_paths: "".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "".to_string(),
        ..Default::default()
    };

    let mut client = TestHttpClient::new(format!("http://localhost:8080/{}", DOMAIN_CODE));

    client.set_auth(&ctx)?;

    test_event_without_mgr::test(&[&client]).await?;
    test_event_with_event_code::test(&[&client]).await?;
    test_event_with_im::test(&[&client]).await?;

    Ok(())
}
