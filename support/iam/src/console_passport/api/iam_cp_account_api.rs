use bios_basic::process::task_processor::TaskProcessor;
use bios_basic::rbum::helper::rbum_event_helper;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use crate::basic::dto::iam_account_dto::IamAccountSelfModifyReq;
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::console_passport::dto::iam_cp_account_dto::IamCpAccountInfoResp;
use crate::console_passport::serv::iam_cp_account_serv::IamCpAccountServ;
use crate::iam_constants;

pub struct IamCpAccountApi;

/// Passport Console Account API
#[poem_openapi::OpenApi(prefix_path = "/cp/account", tag = "bios_basic::ApiTag::Passport")]
impl IamCpAccountApi {
    /// Modify Current Account
    #[oai(path = "/", method = "put")]
    async fn modify(&self, mut modify_req: Json<IamAccountSelfModifyReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let ctx = IamCertServ::use_sys_or_tenant_ctx_unsafe(ctx.0)?;
        IamAccountServ::self_modify_account(&mut modify_req.0, &funs, &ctx).await?;
        IamAccountServ::async_add_or_modify_account_search(ctx.clone().owner, true, "".to_string(), &funs, ctx.clone()).await?;
        funs.commit().await?;
        if let Some(notify_events) = TaskProcessor::get_notify_event_with_ctx(&ctx).await? {
            rbum_event_helper::try_notifies(notify_events, &iam_constants::get_tardis_inst(), &ctx).await?;
        }
        TardisResp::ok(Void {})
    }

    /// Get Current Account
    #[oai(path = "/", method = "get")]
    async fn get_current_account_info(&self, ctx: TardisContextExtractor) -> TardisApiResult<IamCpAccountInfoResp> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamCpAccountServ::get_current_account_info(true, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }
}
