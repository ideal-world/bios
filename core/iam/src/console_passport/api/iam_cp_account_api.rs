use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{param::Query, payload::Json, OpenApi};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use bios_basic::rbum::dto::rbum_rel_agg_dto::RbumRelAggResp;
use bios_basic::rbum::dto::rbum_set_dto::RbumSetPathResp;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_basic::rbum::serv::rbum_set_serv::RbumSetItemServ;

use crate::basic::dto::iam_account_dto::{IamAccountDetailResp, IamAccountSelfModifyReq};
use crate::basic::dto::iam_filer_dto::IamAccountFilterReq;
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_constants;
use crate::iam_enumeration::IamRelKind;

pub struct IamCpAccountApi;

/// Passport Console Account API
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
    async fn find_rel_roles(&self, desc_by_create: Query<Option<bool>>, desc_by_update: Query<Option<bool>>, cxt: TardisContextExtractor) -> TardisApiResult<Vec<RbumRelAggResp>> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamAccountServ::find_rel_roles(&cxt.0.owner, false, desc_by_create.0, desc_by_update.0, &funs, &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Find Rel Set By Current Account
    #[oai(path = "/set-paths", method = "get")]
    async fn find_rel_set_paths(&self, sys_org: Query<Option<bool>>, cxt: TardisContextExtractor) -> TardisApiResult<Vec<Vec<RbumSetPathResp>>> {
        let funs = iam_constants::get_tardis_inst();
        let set_id = if sys_org.0.unwrap_or(false) {
            IamSetServ::get_set_id_by_code(&IamSetServ::get_default_org_code_by_own_paths(""), &funs, &cxt.0).await?
        } else {
            IamSetServ::get_default_set_id_by_cxt(true, &funs, &cxt.0).await?
        };
        let result = RbumSetItemServ::find_set_paths(&cxt.0.owner, &set_id, &funs, &cxt.0).await?;
        TardisResp::ok(result)
    }
}
