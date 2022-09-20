use bios_basic::process::task_processor::TaskProcessor;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::{param::Path, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_app_dto::{IamAppAggModifyReq, IamAppDetailResp};
use crate::basic::dto::iam_filer_dto::IamAppFilterReq;
use crate::basic::serv::iam_app_serv::IamAppServ;
use crate::iam_constants;

pub struct IamCaAppApi;

/// App Console App API
#[poem_openapi::OpenApi(prefix_path = "/ca/app", tag = "crate::iam_enumeration::Tag::App")]
impl IamCaAppApi {
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

    /// Get Current App
    #[oai(path = "/", method = "get")]
    async fn get(&self, ctx: TardisContextExtractor) -> TardisApiResult<IamAppDetailResp> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamAppServ::get_item(&IamAppServ::get_id_by_ctx(&ctx.0, &funs)?, &IamAppFilterReq::default(), &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }

    /// Add App Rel Account
    #[oai(path = "/:id/account/:account_id", method = "put")]
    async fn add_rel_account(&self, id: Path<String>, account_id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAppServ::add_rel_account(&id.0, &account_id.0, false, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Delete App Rel Account
    #[oai(path = "/:id/account/:account_id", method = "delete")]
    async fn delete_rel_account(&self, id: Path<String>, account_id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAppServ::delete_rel_account(&id.0, &account_id.0, &funs, &ctx.0).await?;
        // todo delete app rel account
        funs.commit().await?;
        TardisResp::ok(Void {})
    }
}
