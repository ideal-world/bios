use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use bios_basic::test::init_rbum_test_container;
use bios_mw_schedule::{dto::schedule_job_dto::ScheduleJobAddOrModifyReq, schedule_config::ScheduleConfig, schedule_initializer, serv::schedule_job_serv::ScheduleTaskServ};
use bios_spi_kv::kv_initializer;
use bios_spi_log::log_initializer;
use tardis::{
    basic::result::TardisResult,
    testcontainers,
    tokio::{self},
    web::{
        poem_openapi::{self},
        web_resp::{TardisApiResult, TardisResp, Void},
    },
    TardisFuns,
};

#[tokio::test]
async fn test_basic_schedual_service() -> TardisResult<()> {
    // std::env::set_current_dir("middleware/schedule").unwrap();
    std::env::set_var("RUST_LOG", "info,sqlx=off,sea_orm=INFO");
    let docker = testcontainers::clients::Cli::default();
    let container_hold = init_rbum_test_container::init(&docker, None).await?;
    init_tardis().await?;
    let counter = mock_webserver().await?;
    let config = ScheduleConfig::default();
    ScheduleTaskServ::add(
        "https://127.0.0.1:8080/spi-log",
        ScheduleJobAddOrModifyReq {
            code: "print-hello".into(),
            // do every 2 seconds
            cron: "1/2 * * * * *".into(),
            callback_url: "https://localhost:8080/callback/inc".into(),
        },
        &config,
    )
    .await
    .expect("fail to add schedule task");
    tokio::time::sleep(Duration::from_secs(5)).await;
    assert!(counter.load(Ordering::SeqCst) > 0);
    drop(container_hold);
    Ok(())
}

#[derive(Default)]
struct CallbackApi {
    counter: Arc<AtomicUsize>,
}

#[poem_openapi::OpenApi(prefix_path = "/callback")]
impl CallbackApi {
    #[oai(path = "/inc", method = "get")]
    pub async fn inc(&self) -> TardisApiResult<Void> {
        tardis::log::info!("callback: inc");
        self.counter.fetch_add(1, Ordering::SeqCst);
        TardisResp::ok(Void {})
    }
}

async fn init_tardis() -> TardisResult<()> {
    TardisFuns::init(Some("tests/config")).await?;
    // rbum_initializer::init("", RbumConfig::default()).await?;
    let web_server = TardisFuns::web_server();
    // cache_initializer::init(web_server).await?;
    log_initializer::init(web_server).await?;
    kv_initializer::init(web_server).await?;
    schedule_initializer::init(web_server).await?;
    Ok(())
}

async fn mock_webserver() -> TardisResult<Arc<AtomicUsize>> {
    println!("mock logger started");
    let cb_api = CallbackApi::default();
    let cb_counter = Arc::clone(&cb_api.counter);
    tokio::spawn(TardisFuns::web_server().add_route(cb_api).await.start());
    Ok(cb_counter)
}
