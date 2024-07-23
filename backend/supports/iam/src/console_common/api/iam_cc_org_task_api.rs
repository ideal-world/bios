use bios_basic::{helper::request_helper::try_set_real_ip_from_req_to_ctx, process::task_processor::TaskProcessor};
use tardis::web::{
    context_extractor::TardisContextExtractor,
    poem::Request,
    poem_openapi,
    web_resp::{TardisApiResult, TardisResp},
};

use crate::{console_common::serv::iam_cc_org_task_serv::IamCcOrgTaskServ, iam_constants};

#[derive(Clone, Default)]
pub struct IamCcOrgTaskApi;

/// Common Console Org task API
#[poem_openapi::OpenApi(prefix_path = "/cc/org/task", tag = "bios_basic::ApiTag::Common")]
impl IamCcOrgTaskApi {
    #[oai(path = "/", method = "get")]
    async fn execute_org_task(&self, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Option<String>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        IamCcOrgTaskServ::execute_org_task(&funs, &ctx.0).await?;
        if let Some(task_id) = TaskProcessor::get_task_id_with_ctx(&ctx.0).await? {
            TardisResp::accepted(Some(task_id))
        } else {
            TardisResp::ok(None)
        }
    }
}
