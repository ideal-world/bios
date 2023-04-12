use crate::basic::dto::iam_app_dto::{IamAppAggAddReq, IamAppAggModifyReq};
use crate::basic::serv::iam_app_serv::IamAppServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::iam_constants::{self, RBUM_SCOPE_LEVEL_APP};
use bios_basic::process::task_processor::TaskProcessor;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Path;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

pub struct IamCiAppApi;

/// # Interface Console Manage Cert API
///
/// Allow Management Of aksk (an authentication method between applications)
#[poem_openapi::OpenApi(prefix_path = "/ci/app", tag = "bios_basic::ApiTag::Interface")]
impl IamCiAppApi {
    /// Add App
    #[oai(path = "/app", method = "post")]
    async fn add(&self, add_req: Json<IamAppAggAddReq>, ctx: TardisContextExtractor) -> TardisApiResult<String> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamAppServ::add_app_agg(&add_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(result)
    }

    /// Modify Current App
    ///
    /// When code = 202, the return value is the asynchronous task id
    #[oai(path = "/", method = "put")]
    async fn modify(&self, modify_req: Json<IamAppAggModifyReq>, ctx: TardisContextExtractor) -> TardisApiResult<Option<String>> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAppServ::modify_app_agg(&IamAppServ::get_id_by_ctx(&ctx.0, &funs)?, &modify_req, &funs, &ctx.0).await?;
        funs.commit().await?;
        if let Some(task_id) = TaskProcessor::get_task_id_with_ctx(&ctx.0)? {
            TardisResp::accepted(Some(task_id))
        } else {
            TardisResp::ok(None)
        }
    }
}
