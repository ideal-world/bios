use std::{collections::VecDeque, sync::atomic::Ordering, time::Duration};

use bios_basic::test::init_rbum_test_container;
use bios_mw_schedule::{dto::schedule_job_dto::ScheduleJobAddOrModifyReq, schedule_config::ScheduleConfig, serv::schedule_job_serv::ScheduleTaskServ};

use tardis::{basic::result::TardisResult, rand::random, testcontainers, tokio};

mod test_common;
use test_common::*;

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
    test_random_ops(&config, &test_env).await;
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

async fn test_random_ops(config: &ScheduleConfig, test_env: &TestEnv) {
    test_env.counter.store(0, Ordering::SeqCst);
    let mut tasks = VecDeque::<String>::new();
    const RANGE_SIZE: u8 = 32;

    let random_code = || -> String { format!("task-{:02x}", random::<u8>() % RANGE_SIZE) };
    let mut join_set = tokio::task::JoinSet::new();
    let new_task = |code: &String| -> ScheduleJobAddOrModifyReq {
        // let period = random::<u8>() % 5 + 1;
        ScheduleJobAddOrModifyReq {
            code: code.clone().into(),
            cron: format!("1/{period} * * * * *", period = 2),
            callback_url: "https://localhost:8080/callback/inc".into(),
        }
    };
    let mut counter = 100;
    while counter > 0 {
        let is_delete = random::<u8>() % 3 == 0;
        if is_delete {
            tasks.pop_front().map(|code| join_set.spawn(async move { ScheduleTaskServ::delete(&code).await }));
        } else {
            if tasks.len() > (RANGE_SIZE as usize) / 2 {
                continue;
            }
            let code = 'gen_code: loop {
                let code = random_code();
                if tasks.contains(&code) {
                    continue 'gen_code;
                }
                break 'gen_code code;
            };
            let cfg: ScheduleConfig = config.clone();
            tasks.push_back(code.clone());
            join_set.spawn(async move { ScheduleTaskServ::add("https://127.0.0.1:8080/spi-log", new_task(&code), &cfg).await });
        }
        counter -= 1;
        if counter == 0 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    tokio::time::sleep(Duration::from_secs(2)).await;
}
