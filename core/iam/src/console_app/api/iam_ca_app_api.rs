use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{param::Path, payload::Json, OpenApi};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_app_dto::{IamAppDetailResp, IamAppModifyReq};
use crate::basic::dto::iam_filer_dto::IamAppFilterReq;
use crate::basic::serv::iam_app_serv::IamAppServ;
use crate::console_app::dto::iam_ca_app_dto::IamCaAppModifyReq;
use crate::iam_constants;

pub struct IamCaAppApi;

/// App Console App API
#[OpenApi(prefix_path = "/ca/app", tag = "crate::iam_enumeration::Tag::App")]
impl IamCaAppApi {
    /// Modify Current App
    #[oai(path = "/", method = "put")]
    async fn modify(&self, modify_req: Json<IamCaAppModifyReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAppServ::modify_item(
            &IamAppServ::get_id_by_ctx(&ctx.0, &funs)?,
            &mut IamAppModifyReq {
                name: modify_req.0.name.clone(),
                icon: modify_req.0.icon.clone(),
                sort: modify_req.0.sort,
                contact_phone: modify_req.0.contact_phone.clone(),
                disabled: modify_req.0.disabled,
                scope_level: None,
            },
            &funs,
            &ctx.0,
        )
        .await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Get Current App
    #[oai(path = "/", method = "get")]
    async fn get(&self, ctx: TardisContextExtractor) -> TardisApiResult<IamAppDetailResp> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamAppServ::get_item(&IamAppServ::get_id_by_ctx(&ctx.0, &funs)?, &IamAppFilterReq::default(), &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }

    /// Add Rel Account
    #[oai(path = "/:id/account/:account_id", method = "put")]
    async fn add_rel_account(&self, id: Path<String>, account_id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAppServ::add_rel_account(&id.0, &account_id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Delete Rel Account
    #[oai(path = "/:id/account/:account_id", method = "delete")]
    async fn delete_rel_account(&self, id: Path<String>, account_id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAppServ::delete_rel_account(&id.0, &account_id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }
}
