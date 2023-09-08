mod test_common;
use std::sync::atomic::Ordering;

use bios_basic::test::init_rbum_test_container;
use bios_mw_schedule::{dto::schedule_job_dto::ScheduleJobAddOrModifyReq, schedule_config::ScheduleConfig};
use tardis::chrono::{Utc, self};
use tardis::rand::seq::SliceRandom;
use tardis::{basic::result::TardisResult, rand, testcontainers, tokio};
use test_common::*;

fn new_task(code: &str) -> ScheduleJobAddOrModifyReq {
    // let period = random::<u8>() % 5 + 1;
    ScheduleJobAddOrModifyReq {
        code: code.into(),
        cron: format!("1/{period} * * * * *", period = 2),
        callback_url: "https://localhost:8080/callback/inc".into(),
        enable_time: Utc::now().checked_add_signed(chrono::Duration::seconds(5)),
        disable_time: Utc::now().checked_add_signed(chrono::Duration::seconds(10)),
    }
}

#[tokio::test]
async fn test_multi() -> TardisResult<()> {
    // std::env::set_current_dir("middleware/schedule").unwrap();
    std::env::set_var("RUST_LOG", "info,sqlx=off,sea_orm=INFO,bios_mw_schedule=TRACE,tardis=off");
    let docker = testcontainers::clients::Cli::default();
    let container_hold = init_rbum_test_container::init(&docker, None).await?;
    let config = ScheduleConfig::default();

    init_tardis().await?;
    let counter = mock_webserver().await?;
    let mut serve_group = init_task_serve_group(5).await?;
    let test_env = TestEnv { counter };
    let rng = &mut rand::thread_rng();
    for idx in 0..10 {
        let code = format!("print-hello-{idx}", idx = idx);
        serve_group.shuffle(rng);
        for serv in serve_group.iter() {
            serv.add(new_task(&code), &config).await.expect("fail to add schedule task");
        }
    }
    tokio::time::sleep(tokio::time::Duration::from_secs(11)).await;
    // 10 * (3 / 2 + 1) == 20
    // if every task is executed twice and only executed by one task serv, then the counter should be 20
    assert!(test_env.counter.load(Ordering::SeqCst) == 25);
    drop(container_hold);
    Ok(())
}
