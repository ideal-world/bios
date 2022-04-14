use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json, OpenApi};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use crate::basic::dto::iam_account_dto::{IamAccountDetailResp, IamAccountSummaryResp};
use crate::console_system::dto::iam_cs_account_dto::IamCsAccountModifyReq;
use crate::console_system::serv::iam_cs_account_serv::IamCsAccountServ;
use crate::iam_constants;

pub struct IamCsAccountApi;

/// System Console Account API
#[OpenApi(prefix_path = "/cs/account", tag = "crate::iam_enumeration::Tag::System")]
impl IamCsAccountApi {
    /// Modify Account By Id
    #[oai(path = "/:id", method = "put")]
    async fn modify(&self, id: Path<String>, mut modify_req: Json<IamCsAccountModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCsAccountServ::modify_account(&id.0, &mut modify_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Get Account By Id
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<IamAccountDetailResp> {
        let result = IamCsAccountServ::get_account(&id.0, &iam_constants::get_tardis_inst(), &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Find Accounts
    #[oai(path = "/", method = "get")]
    async fn paginate(
        &self,
        iam_tenant_id: Query<String>,
        name: Query<Option<String>>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        page_number: Query<u64>,
        page_size: Query<u64>,
        cxt: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<IamAccountSummaryResp>> {
        let result = IamCsAccountServ::paginate_accounts(
            iam_tenant_id.0,
            name.0,
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
