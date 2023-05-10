use bios_basic::test::test_http_client::TestHttpClient;
use bios_mw_schedule::dto::schedule_job_dto::ScheduleJobAddOrModifyReq;
use bios_mw_schedule::schedule_config::ScheduleConfig;
use bios_mw_schedule::serv::schedule_job_serv::ScheduleTaskServ;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::TardisFunsInst;
pub async fn test(client: &mut TestHttpClient, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    client.set_auth(&TardisContext {
        own_paths: "t1/app001".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "app001".to_string(),
        ..Default::default()
    })?;
    ScheduleTaskServ::add(
        "",
        ScheduleJobAddOrModifyReq {
            code: TrimString("test".to_string()),
            cron: "0/5 * * * * ? ".to_string(),
            callback_url: "http://localhost:8080/schedule/ci/schedule/test/exec/123".to_string(),
        },
        &ScheduleConfig {
            rbum: RbumConfig,
            app_key: AppKeyConfig,
            spi_app_id: String,
            kv_url: String,
            log_url: String,
            cache_key_job_changed_info: String,
            cache_key_job_changed_timer_sec: u32,
        }
    )
    .await?;
    ScheduleTaskServ::delete("test").await?;
    Ok(())
}
