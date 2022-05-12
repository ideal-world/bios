use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::web::Path;
use tardis::web::poem_openapi::{param::Query, payload::Json, OpenApi};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::dto::rbum_rel_agg_dto::RbumRelAggResp;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_account_dto::{IamAccountAddReq, IamAccountDetailResp, IamAccountModifyReq, IamAccountSummaryResp};
use crate::basic::dto::iam_filer_dto::IamAccountFilterReq;
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::iam_constants;

pub struct IamCcAccountApi;

/// Common Console Account API
#[OpenApi(prefix_path = "/cc/account", tag = "crate::iam_enumeration::Tag::Common")]
impl IamCcAccountApi {
    /// Add Account
    #[oai(path = "/", method = "post")]
    async fn add(&self, mut add_req: Json<IamAccountAddReq>, cxt: TardisContextExtractor) -> TardisApiResult<String> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamAccountServ::add_item(&mut add_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(result)
    }

    /// Modify Account By Id
    #[oai(path = "/:id", method = "put")]
    async fn modify(&self, id: Path<String>, mut modify_req: Json<IamAccountModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAccountServ::modify_item(&id.0, &mut modify_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Get Account By Id
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
        page_number: Query<u64>,
        page_size: Query<u64>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        cxt: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<IamAccountSummaryResp>> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamAccountServ::paginate_items(
            &IamAccountFilterReq {
                basic: RbumBasicFilterReq {
                    ids: id.0.map(|id| vec![id]),
                    name: name.0,
                    own_paths: Some(cxt.0.own_paths.clone()),
                    ..Default::default()
                },
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

    /// Delete Account By Id
    #[oai(path = "/:id", method = "delete")]
    async fn delete(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAccountServ::delete_item(&id.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Find Rel Roles By Account Id
    #[oai(path = "/:id/roles", method = "get")]
    async fn paginate_rel_roles(
        &self,
        id: Path<String>,
        page_number: Query<u64>,
        page_size: Query<u64>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        cxt: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<RbumRelAggResp>> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamAccountServ::paginate_rel_roles(&id.0, page_number.0, page_size.0, desc_by_create.0, desc_by_update.0, &funs, &cxt.0).await?;
        TardisResp::ok(result)
    }
}
