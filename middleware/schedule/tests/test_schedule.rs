use std::env;
use std::time::Duration;

use bios_basic::test::init_rbum_test_container;
use bios_basic::test::test_http_client::TestHttpClient;
use bios_mw_schedule::schedule_constants::DOMAIN_CODE;
use bios_mw_schedule::schedule_initializer;
use bios_spi_kv::kv_initializer;
use bios_spi_log::log_initializer;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::tokio::time::sleep;
use tardis::{testcontainers, tokio, TardisFuns};
mod test_schedule_item;

#[tokio::test]
async fn test_log() -> TardisResult<()> {
    // for debug
    // env::set_current_dir("middleware/schedule").unwrap();
    let docker = testcontainers::clients::Cli::default();
    let container_hold = init_rbum_test_container::init(&docker, None).await?;
    env::set_var("RUST_LOG", "debug,test_reldb=trace,sqlx::query=off");

    init_data().await?;

    drop(container_hold);
    Ok(())
}

async fn init_data() -> TardisResult<()> {
    use bios_basic::rbum::{rbum_config::RbumConfig, rbum_initializer};
    rbum_initializer::init("schedule", RbumConfig::default()).await?;
    let web_server = TardisFuns::web_server();
    // Initialize SPI shedule
    schedule_initializer::init(web_server).await.unwrap();
    log_initializer::init(web_server).await?;
    kv_initializer::init(web_server).await?;

    tokio::spawn(async move {
        web_server.start().await.unwrap();
    });

    sleep(Duration::from_millis(500)).await;

    let funs = TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None);
    let ctx = TardisContext {
        own_paths: "".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "".to_string(),
        ..Default::default()
    };

    let mut client = TestHttpClient::new(format!("https://localhost:8080/{}", DOMAIN_CODE));

    client.set_auth(&ctx)?;

    test_schedule_item::test(&mut client, &funs, &ctx).await?;

    Ok(())
}
