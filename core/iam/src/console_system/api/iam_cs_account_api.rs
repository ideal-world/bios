use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json, OpenApi};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};
use tardis::TardisFuns;

use bios_basic::rbum::dto::filer_dto::RbumItemFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_account_dto::{IamAccountDetailResp, IamAccountSummaryResp};
use crate::basic::serv::iam_account_serv::IamAccountCrudServ;
use crate::console_system::dto::iam_cs_account_dto::IamCsAccountModifyReq;
use crate::console_system::serv::iam_cs_account_serv::IamCsAccountServ;

pub struct IamCsAccountApi;

/// System Console Account API
#[OpenApi(prefix_path = "/cs/account", tag = "bios_basic::Components::Iam")]
impl IamCsAccountApi {
    /// Modify Account
    #[oai(path = "/:id", method = "put")]
    async fn modify(&self, id: Path<String>, mut modify_req: Json<IamCsAccountModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut tx = TardisFuns::reldb().conn();
        tx.begin().await?;
        IamCsAccountServ::modify_account(&id.0, &mut modify_req.0, &tx, &cxt.0).await?;
        tx.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Get Account By Id
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<IamAccountDetailResp> {
        let result = IamAccountCrudServ::get_item(&id.0, &RbumItemFilterReq::default(), &TardisFuns::reldb().conn(), &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Find Accounts
    #[oai(path = "/", method = "get")]
    async fn paginate(
        &self,
        name: Query<Option<String>>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        page_number: Query<u64>,
        page_size: Query<u64>,
        cxt: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<IamAccountSummaryResp>> {
        let result = IamAccountCrudServ::paginate_items(
            &RbumItemFilterReq {
                name: name.0,
                ..Default::default()
            },
            page_number.0,
            page_size.0,
            desc_by_create.0,
            desc_by_update.0,
            &TardisFuns::reldb().conn(),
            &cxt.0,
        )
        .await?;
        TardisResp::ok(result)
    }
}
