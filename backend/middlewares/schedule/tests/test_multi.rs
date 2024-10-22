mod test_common;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use bios_basic::test::init_test_container;
use bios_mw_schedule::dto::schedule_job_dto::ScheduleJob;
use bios_mw_schedule::schedule_constants::DOMAIN_CODE;
use bios_mw_schedule::serv::schedule_job_serv_v2::event::{EventComponent, SpiLog};
use bios_mw_schedule::serv::schedule_job_serv_v2::repo::{Repository, SpiKv};
use tardis::basic::dto::TardisContext;
use tardis::chrono::{self, Utc};
use tardis::rand::seq::SliceRandom;
use tardis::{basic::result::TardisResult, rand, testcontainers, tokio};
use tardis::{TardisFuns, TardisFunsInst};
use test_common::*;

fn new_task(code: &str) -> ScheduleJob {
    // let period = random::<u8>() % 5 + 1;
    ScheduleJob {
        code: code.into(),
        cron: vec![format!("1/{period} * * * * *", period = 2)],
        callback_url: "http://127.0.0.1:8080/callback/inc".into(),
        enable_time: Utc::now().checked_add_signed(chrono::Duration::seconds(5)),
        disable_time: Utc::now().checked_add_signed(chrono::Duration::seconds(10)),
        ..Default::default()
    }
}
fn funs() -> TardisFunsInst {
    TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None)
}
#[tokio::test]
async fn test_multi() -> TardisResult<()> {
    // std::env::set_current_dir("middlewares/schedule").unwrap();
    std::env::set_var("RUST_LOG", "info,sqlx=off,sea_orm=INFO,bios_mw_schedule=TRACE,tardis=off");
    let container_hold = init_test_container::init(None).await?;

    init_tardis().await?;
    let counter = mock_webserver().await?;
    let mut serve_group = init_task_serve_group(5).await?;
    let test_env = TestEnv { counter };
    let rng = &mut rand::thread_rng();
    let ctx = Arc::new(TardisContext::default());
    let funs = Arc::new(funs());
    for idx in 0..10 {
        let code = format!("print-hello-{idx}", idx = idx);
        serve_group.shuffle(rng);
        for serv in serve_group.iter() {
            serv.set_job(
                new_task(&code),
                SpiKv::from_context(funs.clone(), ctx.clone()),
                SpiLog::from_context(funs.clone(), ctx.clone()),
            )
            .await
            .expect("fail to add schedule task");
        }
    }
    tokio::time::sleep(tokio::time::Duration::from_secs(11)).await;
    // 10 * (3 / 2 + 1) == 20
    // if every task is executed twice and only executed by one task serv, then the counter should be 20
    assert!(test_env.counter.load(Ordering::SeqCst) == 25);
    drop(container_hold);
    Ok(())
}
