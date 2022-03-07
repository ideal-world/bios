use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json, OpenApi};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};
use tardis::TardisFuns;

use crate::console_system::dto::iam_cs_tenant_dto::{IamCsTenantAddReq, IamCsTenantDetailResp, IamCsTenantModifyReq, IamCsTenantSummaryResp};
use crate::console_system::serv::iam_cs_tenant_serv;

pub struct IamCsTenantApi;

/// System Console Tenant API
#[OpenApi(prefix_path = "/cs/tenant", tag = "bios_basic::Components::IAM")]
impl IamCsTenantApi {
    /// Add Tenant
    #[oai(path = "/", method = "post")]
    async fn add(&self, add_req: Json<IamCsTenantAddReq>, cxt: TardisContextExtractor) -> TardisApiResult<String> {
        let mut tx = TardisFuns::reldb().conn();
        tx.begin().await?;
        let result = iam_cs_tenant_serv::add_iam_tenant(&add_req, &tx, &cxt.0).await?;
        tx.commit().await?;
        TardisResp::ok(result)
    }

    /// Modify Tenant
    #[oai(path = "/:id", method = "put")]
    async fn modify(&self, id: Path<String>, modify_req: Json<IamCsTenantModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut tx = TardisFuns::reldb().conn();
        tx.begin().await?;
        iam_cs_tenant_serv::modify_iam_tenant(&id.0, &modify_req, &tx, &cxt.0).await?;
        tx.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Delete Tenant
    #[oai(path = "/:id", method = "delete")]
    async fn delete(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut tx = TardisFuns::reldb().conn();
        tx.begin().await?;
        iam_cs_tenant_serv::delete_iam_tenant(&id.0, &tx, &cxt.0).await?;
        tx.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Get Tenant Summary By Id
    #[oai(path = "/:id/summary", method = "get")]
    async fn get_summary(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<IamCsTenantSummaryResp> {
        let result = iam_cs_tenant_serv::peek_iam_tenant(&id.0, &TardisFuns::reldb().conn(), &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Get Tenant Detail By Id
    #[oai(path = "/:id/detail", method = "get")]
    async fn get_detail(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<IamCsTenantDetailResp> {
        let result = iam_cs_tenant_serv::get_iam_tenant(&id.0, &TardisFuns::reldb().conn(), &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Find Tenants
    #[oai(path = "/", method = "get")]
    async fn find(&self, page_number: Query<u64>, page_size: Query<u64>, cxt: TardisContextExtractor) -> TardisApiResult<TardisPage<IamCsTenantDetailResp>> {
        let result = iam_cs_tenant_serv::find_iam_tenants(page_number.0, page_size.0, None, &TardisFuns::reldb().conn(), &cxt.0).await?;
        TardisResp::ok(result)
    }
}
