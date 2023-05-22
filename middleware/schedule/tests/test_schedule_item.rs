use bios_basic::test::test_http_client::TestHttpClient;
use bios_mw_schedule::dto::schedule_job_dto::ScheduleJobAddOrModifyReq;
use bios_mw_schedule::schedule_config::ScheduleConfig;
use bios_mw_schedule::serv::schedule_job_serv::ScheduleTaskServ;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::serde_json::json;
use tardis::web::web_resp::Void;
use tardis::TardisFunsInst;
pub async fn test(client: &mut TestHttpClient, _funs: &TardisFunsInst, _ctx: &TardisContext) -> TardisResult<()> {
    client.set_auth(&TardisContext {
        own_paths: "t1/app001".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "app001".to_string(),
        ..Default::default()
    })?;
    let req = json!({
        "code": "test",
        "cron": "0/5 * * * * ?",
        "callback_url": "http://localhost:8080/schedule/ci/schedule/test/exec/123",
    });
    let _resp = client.put::<_, Void>(
        "schedual/ci/schedule/jobs",
        &req,
    ).await;
    client.delete(
        "schedual/ci/schedule/jobs/test"
    ).await;
    Ok(())
}
