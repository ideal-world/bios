use tardis::basic::dto::TardisContext;
use tardis::db::sea_orm::*;
use tardis::TardisFuns;
use tardis::web::poem_openapi::{Object, OpenApi, param::Path, payload::Json};
use tardis::web::web_resp::{TardisPage, TardisResp};

use crate::iam::console_system::dto::iam_cs_tenant_dto::{IamCsTenantAddReq, IamCsTenantDetailResp, IamCsTenantModifyReq, IamCsTenantSummaryResp};
use crate::iam::console_system::serv::iam_cs_tenant_serv;

pub struct IamCsTenantApi;

#[OpenApi]
impl IamCsTenantApi {
    #[oai(path = "/cs/tenant", method = "post")]
    async fn add(&self, add_req: Json<IamCsTenantAddReq>) -> TardisResp<String> {
        // Todo
        let tx = TardisFuns::reldb().conn().begin().await.unwrap();
        // Todo
        let cxt = TardisContext {
            ..Default::default()
        };
        let result = iam_cs_tenant_serv::add_iam_tenant(&add_req, &tx, &cxt).await.unwrap();
        tx.commit().await.unwrap();
        TardisResp::ok(result)
    }

    #[oai(path = "/cs/tenant/:id", method = "put")]
    async fn modify(&self, id: Path<String>, modify_req: Json<IamCsTenantModifyReq>) ->
                                                                               TardisResp<()> {
        // Todo
        let tx = TardisFuns::reldb().conn().begin().await.unwrap();
        // Todo
        let cxt = TardisContext {
            ..Default::default()
        };
        iam_cs_tenant_serv::modify_iam_tenant(&id.0,&modify_req, &tx, &cxt).await
            .unwrap();
        tx.commit().await.unwrap();
        TardisResp::ok(())
    }

    #[oai(path = "/cs/tenant/:id", method = "delete")]
    async fn delete(&self, id: Path<i32>) -> TardisResp<()> {
        // Todo
        let tx = TardisFuns::reldb().conn().begin().await.unwrap();
        // Todo
        let cxt = TardisContext {
            ..Default::default()
        };
        iam_cs_tenant_serv::delete_iam_tenant(&id.0,&tx, &cxt).await
            .unwrap();
        tx.commit().await.unwrap();
        TardisResp::ok(())
    }

    #[oai(path = "/cs/tenant/:id/summary", method = "get")]
    async fn get_summary(&self, id: Path<i32>) -> TardisResp<IamCsTenantSummaryResp> {
        // Todo
        let cxt = TardisContext {
            ..Default::default()
        };
        let result =  iam_cs_tenant_serv::peek_iam_tenant(&id.0,TardisFuns::reldb().conn(), &cxt).await
            .unwrap();
        TardisResp::ok(result)
    }

    #[oai(path = "/cs/tenant/:id/detail", method = "get")]
    async fn get_detail(&self, id: Path<i32>) -> TardisResp<IamCsTenantDetailResp> {
        // Todo
        let cxt = TardisContext {
            ..Default::default()
        };
        let result =  iam_cs_tenant_serv::get_iam_tenant(&id.0,TardisFuns::reldb().conn(), &cxt).await
            .unwrap();
        TardisResp::ok(result)
    }

    #[oai(path = "/cs/tenant", method = "get")]
    async fn find(&self, page_number: Query<u64>, page_size: Query<u64>) ->
                                                                          TardisResp<TardisPage<IamCsTenantDetailResp>> {
        // Todo
        let cxt = TardisContext {
            ..Default::default()
        };
        let result =  iam_cs_tenant_serv::find_iam_tenants(TardisFuns::reldb().conn(), &cxt).await
            .unwrap();
        TardisResp::ok(result)
    }




}
