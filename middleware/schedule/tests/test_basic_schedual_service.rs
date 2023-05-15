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
    rand::random,
    testcontainers,
    tokio::{self},
    web::{
        poem_openapi::{self},
        web_resp::{TardisApiResult, TardisResp, Void},
    },
    TardisFuns, log::error,
};

pub struct TestEnv {
    pub counter: Arc<AtomicUsize>,
}
#[tokio::test]
async fn test_basic_schedual_service() -> TardisResult<()> {
    // std::env::set_current_dir("middleware/schedule").unwrap();
    std::env::set_var("RUST_LOG", "info,sqlx=off,sea_orm=INFO");
    let docker = testcontainers::clients::Cli::default();
    let container_hold = init_rbum_test_container::init(&docker, None).await?;
    init_tardis().await?;
    let counter = mock_webserver().await?;
    let test_env = TestEnv { counter };
    let config = ScheduleConfig::default();

    test_add_delete(&config, &test_env).await;
    test_random_add_delete(&config, &test_env).await;
    drop(container_hold);
    Ok(())
}

async fn test_add_delete(config: &ScheduleConfig, test_env: &TestEnv) {
    let code = "print-hello";
    ScheduleTaskServ::add(
        "https://127.0.0.1:8080/spi-log",
        ScheduleJobAddOrModifyReq {
            code: code.into(),
            // do every 2 seconds
            cron: "1/2 * * * * *".into(),
            callback_url: "https://localhost:8080/callback/inc".into(),
        },
        config,
    )
    .await
    .expect("fail to add schedule task");
    tokio::time::sleep(Duration::from_secs(5)).await;
    assert!(test_env.counter.load(Ordering::SeqCst) > 0);
    ScheduleTaskServ::delete(code).await.expect("fail to delete schedule task");
}

async fn test_random_add_delete(config: &ScheduleConfig, test_env: &TestEnv) {
    test_env.counter.store(0, Ordering::SeqCst);
    fn random_task(range_size: u8) -> ScheduleJobAddOrModifyReq {
        // let period = random::<u8>() % 5 + 1;
        ScheduleJobAddOrModifyReq {
            code: format!("task-{:02x}", random::<u8>() % range_size).into(),
            cron: format!("1/{period} * * * * *", period = 2),
            callback_url: "https://localhost:8080/callback/inc".into(),
        }
    }
    fn random_delete(range_size: u8) -> String {
        format!("task-{:02x}", random::<u8>() % range_size)
    }
    const RANGE_SIZE: u8 = 32;
    // let mut join_set = tokio::task::JoinSet::new();
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(1));
    let mut counter = 0;
    // loop {
    //     if counter >= 50 {
    //         break;
    //     }
    //     // 1/3 chance to add a new task
    //     let is_add = random::<u8>() % 3 == 0;
    //     if is_add {
    //         let config = config.clone();
    //         join_set.spawn(async move { 
    //             let result = ScheduleTaskServ::add("https://127.0.0.1:8080/spi-log", random_task(RANGE_SIZE), &config).await ;
    //             if let Err(e) = &result {
    //                 error!("add error {e}")
    //             }
    //             result
    //         });
    //     } else {
    //         join_set.spawn(async move { ScheduleTaskServ::delete(&random_delete(RANGE_SIZE)).await });
    //     }
    //     interval.tick().await;
    //     counter += 1;
    // }
    // while let Some(Ok(_res)) = join_set.join_next().await {
    //     //
    // }
    // tokio::time::sleep(Duration::from_secs(5)).await;
}

#[derive(Default)]
struct CallbackApi {
    counter: Arc<AtomicUsize>,
}

#[poem_openapi::OpenApi(prefix_path = "/callback")]
impl CallbackApi {
    #[oai(path = "/inc", method = "get")]
    pub async fn inc(&self) -> TardisApiResult<Void> {
        let counter = self.counter.fetch_add(1, Ordering::SeqCst) + 1;
        tardis::log::info!("callback: inc to {counter}");
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
