use std::env;
use std::time::Duration;

use bios_basic::test::init_test_container;

use bios_mw_schedule::schedule_constants::DOMAIN_CODE;
use bios_mw_schedule::schedule_initializer;
use bios_spi_kv::kv_initializer;
use bios_spi_log::log_initializer;

use tardis::basic::result::TardisResult;
use tardis::tokio::time::sleep;

use tardis::{testcontainers, tokio, TardisFuns};

use crate::test_common::init_spi;
mod test_common;
mod test_schedule_item;
#[tokio::test]
async fn test_log() -> TardisResult<()> {
    // for debug
    // env::set_current_dir("middlewares/schedule").unwrap();
    let docker = testcontainers::clients::Cli::default();
    let container_hold = init_test_container::init(&docker, None).await?;
    env::set_var("RUST_LOG", "debug,test_schedual=trace,sqlx::query=off,bios_mw_schedule=trace,bios_spi_kv=trace");

    init_data().await?;

    drop(container_hold);
    Ok(())
}

async fn init_data() -> TardisResult<()> {
    use bios_basic::rbum::rbum_config::RbumConfig;
    bios_basic::rbum::rbum_initializer::init(DOMAIN_CODE, RbumConfig::default()).await?;
    let web_server = TardisFuns::web_server();
    // Initialize SPI shedule
    schedule_initializer::init(&web_server).await?;
    log_initializer::init(&web_server).await?;
    kv_initializer::init(&web_server).await?;

    tokio::spawn(async move {
        web_server.start().await.unwrap();
    });

    sleep(Duration::from_millis(1000)).await;
    const LOG_DOMAIN_CODE: &str = bios_spi_log::log_constants::DOMAIN_CODE;
    const KV_DOMAIN_CODE: &str = bios_spi_kv::kv_constants::DOMAIN_CODE;
    init_spi(LOG_DOMAIN_CODE).await?;
    init_spi(KV_DOMAIN_CODE).await?;
    let mut client = test_common::init_client().await?;
    let funs = TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None);

    test_schedule_item::test(&mut client, &funs).await?;

    Ok(())
}
