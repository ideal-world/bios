use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{OpenApi, param::Path, param::Query, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use bios_basic::rbum::dto::rbum_rel_agg_dto::RbumRelAggResp;

use crate::basic::dto::iam_account_dto::{IamAccountDetailResp, IamAccountSummaryResp};
use crate::console_tenant::dto::iam_ct_account_dto::{IamCtAccountAddReq, IamCtAccountModifyReq};
use crate::console_tenant::serv::iam_ct_account_serv::IamCtAccountServ;
use crate::iam_constants;

pub struct IamCtAccountApi;

/// Tenant Console Account API
#[OpenApi(prefix_path = "/ct/account", tag = "crate::iam_enumeration::Tag::Tenant")]
impl IamCtAccountApi {
    /// Add Account
    #[oai(path = "/", method = "post")]
    async fn add(&self, mut add_req: Json<IamCtAccountAddReq>, cxt: TardisContextExtractor) -> TardisApiResult<String> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamCtAccountServ::add_account(&mut add_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(result)
    }

    /// Modify Account By Id
    #[oai(path = "/:id", method = "put")]
    async fn modify(&self, id: Path<String>, mut modify_req: Json<IamCtAccountModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCtAccountServ::modify_account(&id.0, &mut modify_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Get Account By Id
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<IamAccountDetailResp> {
        let result = IamCtAccountServ::get_account(&id.0, &iam_constants::get_tardis_inst(), &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Find Accounts
    #[oai(path = "/", method = "get")]
    async fn paginate(
        &self,
        q_id: Query<Option<String>>,
        q_name: Query<Option<String>>,
        page_number: Query<u64>,
        page_size: Query<u64>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        cxt: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<IamAccountSummaryResp>> {
        let result = IamCtAccountServ::paginate_accounts(
            q_id.0,
            q_name.0,
            page_number.0,
            page_size.0,
            desc_by_create.0,
            desc_by_update.0,
            &iam_constants::get_tardis_inst(),
            &cxt.0,
        )
        .await?;
        TardisResp::ok(result)
    }

    /// Delete Account By Id
    #[oai(path = "/:id", method = "delete")]
    async fn delete(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCtAccountServ::delete_account(&id.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Find Rel Roles By Account Id
    #[oai(path = "/:id/roles", method = "get")]
    async fn paginate_rel_roles(
        &self,
        id: Path<String>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        page_number: Query<u64>,
        page_size: Query<u64>,
        cxt: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<RbumRelAggResp>> {
        let result = IamCtAccountServ::paginate_rel_roles(
            &id.0,
            page_number.0,
            page_size.0,
            desc_by_create.0,
            desc_by_update.0,
            &iam_constants::get_tardis_inst(),
            &cxt.0,
        )
        .await?;
        TardisResp::ok(result)
    }
}
