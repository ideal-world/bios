use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{param::Query, payload::Json, OpenApi};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use bios_basic::rbum::dto::rbum_rel_agg_dto::RbumRelAggResp;

use crate::basic::dto::iam_account_dto::IamAccountDetailResp;
use crate::console_passport::dto::iam_cp_account_dto::IamCpAccountModifyReq;
use crate::console_passport::serv::iam_cp_account_serv::IamCpAccountServ;
use crate::iam_constants;

pub struct IamCpAccountApi;

/// Personal Console Account API
#[OpenApi(prefix_path = "/cp/account", tag = "crate::iam_enumeration::Tag::Passport")]
impl IamCpAccountApi {
    /// Modify Current Account
    #[oai(path = "/", method = "put")]
    async fn modify(&self, mut modify_req: Json<IamCpAccountModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCpAccountServ::modify_account(&mut modify_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Get Current Account
    #[oai(path = "/", method = "get")]
    async fn get(&self, cxt: TardisContextExtractor) -> TardisApiResult<IamAccountDetailResp> {
        let result = IamCpAccountServ::get_account(&iam_constants::get_tardis_inst(), &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Find Rel Roles By Current Account
    #[oai(path = "/roles", method = "get")]
    async fn paginate_rel_roles(
        &self,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        page_number: Query<u64>,
        page_size: Query<u64>,
        cxt: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<RbumRelAggResp>> {
        let result = IamCpAccountServ::paginate_rel_roles(page_number.0, page_size.0, desc_by_create.0, desc_by_update.0, &iam_constants::get_tardis_inst(), &cxt.0).await?;
        TardisResp::ok(result)
    }
}
