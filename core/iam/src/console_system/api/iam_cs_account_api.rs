use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::web::Path;
use tardis::web::poem_openapi::{param::Query, OpenApi};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::dto::rbum_rel_agg_dto::RbumRelAggResp;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_filer_dto::IamAccountFilterReq;
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::iam_constants;

pub struct IamCsAccountApi;

/// System Console Tenant API
#[OpenApi(prefix_path = "/cs/account", tag = "crate::iam_enumeration::Tag::System")]
impl IamCsAccountApi {
    /// Count Accounts
    #[oai(path = "/total", method = "get")]
    async fn count(&self, tenant_id: Query<String>, cxt: TardisContextExtractor) -> TardisApiResult<u64> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamAccountServ::count_items(
            &IamAccountFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some(tenant_id.0.clone()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
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
