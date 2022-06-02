use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{param::Path, param::Query, OpenApi};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp};

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemRelFilterReq};
use bios_basic::rbum::dto::rbum_rel_dto::RbumRelBoneResp;
use bios_basic::rbum::dto::rbum_set_dto::RbumSetPathResp;
use bios_basic::rbum::rbum_enumeration::RbumRelFromKind;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_account_dto::{IamAccountDetailResp, IamAccountSummaryResp};
use crate::basic::dto::iam_filer_dto::IamAccountFilterReq;
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_app_serv::IamAppServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_constants;
use crate::iam_enumeration::IamRelKind;

pub struct IamCaAccountApi;

/// App Console Account API
#[OpenApi(prefix_path = "/ca/account", tag = "crate::iam_enumeration::Tag::App")]
impl IamCaAccountApi {
    /// Get Account By Account Id
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<IamAccountDetailResp> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamAccountServ::get_item(&id.0, &IamAccountFilterReq::default(), &funs, &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Find Accounts
    #[oai(path = "/", method = "get")]
    async fn paginate(
        &self,
        id: Query<Option<String>>,
        name: Query<Option<String>>,
        role_id: Query<Option<String>>,
        page_number: Query<u64>,
        page_size: Query<u64>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        cxt: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<IamAccountSummaryResp>> {
        let funs = iam_constants::get_tardis_inst();
        let rel2 = role_id.0.map(|role_id| RbumItemRelFilterReq {
            rel_by_from: true,
            tag: Some(IamRelKind::IamAccountRole.to_string()),
            from_rbum_kind: Some(RbumRelFromKind::Item),
            rel_item_id: Some(role_id),
        });
        let result = IamAccountServ::paginate_items(
            &IamAccountFilterReq {
                basic: RbumBasicFilterReq {
                    ids: id.0.map(|id| vec![id]),
                    name: name.0,
                    ..Default::default()
                },
                rel: IamAppServ::with_app_rel_filter(&cxt.0, &funs)?,
                rel2,
                ..Default::default()
            },
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

    /// Find Rel Roles By Account Id
    #[oai(path = "/:id/role", method = "get")]
    async fn find_rel_roles(
        &self,
        id: Path<String>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        cxt: TardisContextExtractor,
    ) -> TardisApiResult<Vec<RbumRelBoneResp>> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamAccountServ::find_simple_rel_roles(&id.0, true, desc_by_create.0, desc_by_update.0, &funs, &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Find Rel Set By Account Id
    #[oai(path = "/:id/set-path", method = "get")]
    async fn find_rel_set_paths(&self, id: Path<String>, sys_org: Query<Option<bool>>, cxt: TardisContextExtractor) -> TardisApiResult<Vec<Vec<RbumSetPathResp>>> {
        let funs = iam_constants::get_tardis_inst();
        let set_id = if sys_org.0.unwrap_or(false) {
            IamSetServ::get_set_id_by_code(&IamSetServ::get_default_org_code_by_own_paths(""), true, &funs, &cxt.0).await?
        } else {
            IamSetServ::get_default_set_id_by_cxt(true, &funs, &cxt.0).await?
        };
        let result = IamSetServ::find_set_paths(&id.0, &set_id, &funs, &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Count Accounts By Current Tenant
    #[oai(path = "/total", method = "get")]
    async fn count(&self, cxt: TardisContextExtractor) -> TardisApiResult<u64> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamAccountServ::count_items(&IamAccountFilterReq::default(), &funs, &cxt.0).await?;
        TardisResp::ok(result)
    }
}
