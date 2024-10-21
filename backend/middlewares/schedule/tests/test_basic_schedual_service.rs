use bios_mw_schedule::{
    dto::schedule_job_dto::ScheduleJob,
    schedule_config::ScheduleConfig,
    schedule_constants::DOMAIN_CODE,
    serv::schedule_job_serv_v2::{add_or_modify, delete},
};
use std::{collections::VecDeque, env, sync::atomic::Ordering, time::Duration};

use tardis::{
    basic::result::TardisResult,
    chrono::{self, Utc},
    rand::random,
    test::test_container::TardisTestContainer,
    tokio, TardisFuns, TardisFunsInst,
};

mod test_common;
use test_common::*;
fn funs() -> TardisFunsInst {
    TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None)
}
#[tokio::test]
async fn test_basic_schedual_service() -> TardisResult<()> {
    // std::env::set_current_dir("middlewares/schedule").unwrap();
    std::env::set_var("RUST_LOG", "info,sqlx=off,sea_orm=INFO");
    let reldb_container = TardisTestContainer::postgres_custom(None).await?;
    let port = reldb_container.get_host_port_ipv4(5432).await?;
    let url = format!("postgres://postgres:123456@127.0.0.1:{port}/test");
    env::set_var("TARDIS_FW.DB.URL", url);

    let redis_container = TardisTestContainer::redis_custom().await?;
    let port = redis_container.get_host_port_ipv4(6379).await?;
    let url = format!("redis://127.0.0.1:{port}/0");
    env::set_var("TARDIS_FW.CACHE.URL", url);

    let rabbit_container = TardisTestContainer::rabbit_custom().await?;
    let port = rabbit_container.get_host_port_ipv4(5672).await?;
    let url = format!("amqp://guest:guest@127.0.0.1:{port}/%2f");
    env::set_var("TARDIS_FW.MQ.URL", url);

    let holder = (reldb_container, redis_container, rabbit_container);
    init_tardis().await?;

    let counter = mock_webserver().await?;
    let test_env = TestEnv { counter };
    let config = ScheduleConfig::default();

    test_add_delete(&test_env).await;
    // test_random_ops(&config, &test_env).await;
    drop(holder);
    Ok(())
}

async fn test_add_delete(test_env: &TestEnv) {
    let code = "print-hello";
    let funs = TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None);
    add_or_modify(
        ScheduleJob {
            code: code.into(),
            // do every 2 seconds
            cron: vec!["1/2 * * * * *".to_string()],
            callback_url: "http://127.0.0.1:8080/callback/inc".into(),
            ..Default::default()
        },
        funs,
        Default::default(),
    )
    .await
    .expect("fail to modify");
    tokio::time::sleep(Duration::from_secs(5)).await;
    assert!(test_env.counter.load(Ordering::SeqCst) > 0);
    let funs = TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None);
    delete(code, funs, Default::default()).await.expect("fail to delete schedule task");
}

async fn test_random_ops(config: &ScheduleConfig, test_env: &TestEnv) {
    test_env.counter.store(0, Ordering::SeqCst);
    let mut tasks = VecDeque::<String>::new();
    const RANGE_SIZE: u8 = 32;

    let random_code = || -> String { format!("task-{:02x}", random::<u8>() % RANGE_SIZE) };
    let mut join_set = tokio::task::JoinSet::new();
    let new_task = |code: &String| -> ScheduleJob {
        // let period = random::<u8>() % 5 + 1;
        ScheduleJob {
            code: code.clone().into(),
            cron: vec![format!("1/{period} * * * * *", period = 2)],
            callback_url: "https://127.0.0.1:8080/callback/inc".into(),
            enable_time: Utc::now().checked_add_signed(chrono::Duration::seconds(5)),
            ..Default::default()
        }
    };
    let mut counter = 100;
    while counter > 0 {
        let is_delete = random::<u8>() % 3 == 0;
        if is_delete {
            tasks.pop_front().map(|code| join_set.spawn(async move { delete(&code, funs(), Default::default()).await }));
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
            join_set.spawn(async move { add_or_modify(new_task(&code), funs(), Default::default()).await });
        }
        counter -= 1;
        if counter == 0 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    tokio::time::sleep(Duration::from_secs(2)).await;
}
