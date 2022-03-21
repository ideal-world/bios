use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json, OpenApi};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};
use tardis::TardisFuns;

use bios_basic::rbum::dto::filer_dto::RbumItemFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::console_system::dto::iam_cs_tenant_dto::{IamCsTenantAddReq, IamCsTenantDetailResp, IamCsTenantModifyReq, IamCsTenantSummaryResp};
use crate::console_system::serv::iam_cs_tenant_serv::IamCsTenantServ;

pub struct IamCsTenantApi;

/// System Console Tenant API
#[OpenApi(prefix_path = "/cs/tenant", tag = "bios_basic::Components::Iam")]
impl IamCsTenantApi {
    /// Add Tenant
    #[oai(path = "/", method = "post")]
    async fn add(&self, mut add_req: Json<IamCsTenantAddReq>, cxt: TardisContextExtractor) -> TardisApiResult<String> {
        let mut tx = TardisFuns::reldb().conn();
        tx.begin().await?;
        let result = IamCsTenantServ::add_item(&mut add_req.0, &tx, &cxt.0).await?;
        tx.commit().await?;
        TardisResp::ok(result)
    }

    /// Modify Tenant
    #[oai(path = "/:id", method = "put")]
    async fn modify(&self, id: Path<String>, mut modify_req: Json<IamCsTenantModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut tx = TardisFuns::reldb().conn();
        tx.begin().await?;
        IamCsTenantServ::modify_item(&id.0, &mut modify_req.0, &tx, &cxt.0).await?;
        tx.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Delete Tenant
    #[oai(path = "/:id", method = "delete")]
    async fn delete(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut tx = TardisFuns::reldb().conn();
        tx.begin().await?;
        IamCsTenantServ::delete_item(&id.0, &tx, &cxt.0).await?;
        tx.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Get Tenant Detail By Id
    #[oai(path = "/:id/detail", method = "get")]
    async fn get_detail(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<IamCsTenantDetailResp> {
        let result = IamCsTenantServ::get_item(&id.0, &RbumItemFilterReq::default(), &TardisFuns::reldb().conn(), &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Find Tenants
    #[oai(path = "/", method = "get")]
    async fn paginate(
        &self,
        name: Query<Option<String>>,
        page_number: Query<u64>,
        page_size: Query<u64>,
        cxt: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<IamCsTenantSummaryResp>> {
        let result = IamCsTenantServ::paginate_items(
            &RbumItemFilterReq {
                name: name.0,
                ..Default::default()
            },
            page_number.0,
            page_size.0,
            None,
            &TardisFuns::reldb().conn(),
            &cxt.0,
        )
        .await?;
        TardisResp::ok(result)
    }
}
