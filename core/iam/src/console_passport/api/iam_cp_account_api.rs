use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{param::Query, payload::Json, OpenApi};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use bios_basic::rbum::dto::rbum_rel_agg_dto::RbumRelAggResp;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_account_dto::{IamAccountDetailResp, IamAccountSelfModifyReq};
use crate::basic::dto::iam_filer_dto::IamAccountFilterReq;
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::iam_constants;
use crate::iam_enumeration::IamRelKind;

pub struct IamCpAccountApi;

/// Personal Console Account API
#[OpenApi(prefix_path = "/cp/account", tag = "crate::iam_enumeration::Tag::Passport")]
impl IamCpAccountApi {
    /// Modify Current Account
    #[oai(path = "/", method = "put")]
    async fn modify(&self, mut modify_req: Json<IamAccountSelfModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAccountServ::self_modify_account(&mut modify_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Get Current Account
    #[oai(path = "/", method = "get")]
    async fn get(&self, cxt: TardisContextExtractor) -> TardisApiResult<IamAccountDetailResp> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamAccountServ::get_item(&cxt.0.owner, &IamAccountFilterReq::default(), &funs, &cxt.0).await?;
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
        let funs = iam_constants::get_tardis_inst();
        let result = IamRelServ::paginate_from_rels(
            IamRelKind::IamAccountRole,
            false,
            &cxt.0.owner,
            page_number.0,
            page_size.0,
            desc_by_create.0,
            desc_by_update.0,
            &funs,
            &cxt.0,
        )
        .await?;
        TardisResp::ok(result)
    }
}
