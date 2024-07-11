use bios_basic::TardisFunInstExtractor;
use tardis::basic::error::TardisError;
use tardis::chrono::{self, Utc};
use tardis::log::info;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use crate::dto::schedule_job_dto::{ScheduleJob, ScheduleJobInfoResp, ScheduleTaskInfoResp};
use crate::serv::{schedule_job_serv, schedule_job_serv_v2};

#[derive(Clone)]
pub struct ScheduleCiJobApi;

/// Interface Console schedule API
/// 接口控制台调度API
#[poem_openapi::OpenApi(prefix_path = "/ci/schedule", tag = "bios_basic::ApiTag::Interface")]
impl ScheduleCiJobApi {
    /// Add or modify schedule job Api
    /// 添加或修改调度任务
    #[oai(path = "/jobs", method = "put")]
    async fn add_or_modify(&self, add_or_modify_req: Json<ScheduleJob>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        let req = add_or_modify_req.0;
        if req.cron.iter().any(|s| s.trim().starts_with('*')) {
            return TardisResp::err(TardisError::bad_request("cron that start with * is forbidden", "schedule-bad-cron"));
        }
        schedule_job_serv_v2::add_or_modify(req, funs, ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Delete schedule job Api
    /// 删除调度任务
    #[oai(path = "/jobs/:code", method = "delete")]
    async fn delete(&self, code: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        schedule_job_serv_v2::delete(&code.0, funs, ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// find schedule job Api page
    /// 查询调度任务分页
    #[oai(path = "/jobs", method = "get")]
    async fn find_job(
        &self,
        code: Query<Option<String>>,
        page_number: Query<u32>,
        page_size: Query<u16>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<TardisPage<ScheduleJobInfoResp>> {
        let funs = request.tardis_fun_inst();
        let resp = schedule_job_serv::find_job(code.0, page_number.0, page_size.0, &funs, &ctx.0).await?;
        TardisResp::ok(resp)
    }

    /// find schedule task Api page
    /// 查询调度任务分页
    #[oai(path = "/task", method = "get")]
    async fn find_task(
        &self,
        job_code: Query<String>,
        ts_start: Query<Option<chrono::DateTime<Utc>>>,
        ts_end: Query<Option<chrono::DateTime<Utc>>>,
        page_number: Query<u32>,
        page_size: Query<u16>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<TardisPage<ScheduleTaskInfoResp>> {
        let funs = request.tardis_fun_inst();
        let resp = schedule_job_serv::find_task(&job_code.0, ts_start.0, ts_end.0, page_number.0, page_size.0, &funs, &ctx.0).await?;
        TardisResp::ok(resp)
    }

    /// get job test
    /// 测试接口
    #[oai(path = "/test/exec/:msg", method = "get")]
    async fn test_exec(&self, msg: Path<String>, _ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        for (k, v) in request.headers() {
            info!("{}: {}", k, v.to_str().expect("not as str"));
        }
        TardisResp::ok(msg.0)
    }
}
