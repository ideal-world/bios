use tardis::TardisFuns;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{OpenApi, param::Path, param::Query, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use crate::basic::dto::iam_app_dto::{IamAppDetailResp, IamAppSummaryResp};
use crate::console_tenant::dto::iam_ct_app_dto::{IamCtAppAddReq, IamCtAppModifyReq};
use crate::console_tenant::serv::iam_ct_app_serv::IamCtAppServ;

pub struct IamCtAppApi;

/// Tenant Console App API
#[OpenApi(prefix_path = "/ct/app", tag = "bios_basic::Components::Iam")]
impl IamCtAppApi {
    /// Add App
    #[oai(path = "/", method = "post")]
    async fn add(&self, mut add_req: Json<IamCtAppAddReq>, cxt: TardisContextExtractor) -> TardisApiResult<String> {
        let mut tx = TardisFuns::reldb().conn();
        tx.begin().await?;
        let result = IamCtAppServ::add_app(&mut add_req.0, &tx, &cxt.0).await?;
        tx.commit().await?;
        TardisResp::ok(result)
    }

    /// Modify App By Id
    #[oai(path = "/:id", method = "put")]
    async fn modify(&self, id: Path<String>, mut modify_req: Json<IamCtAppModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut tx = TardisFuns::reldb().conn();
        tx.begin().await?;
        IamCtAppServ::modify_app(&id.0, &mut modify_req.0, &tx, &cxt.0).await?;
        tx.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Get App By Id
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<IamAppDetailResp> {
        let result = IamCtAppServ::get_app(&id.0, &TardisFuns::reldb().conn(), &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Find Apps
    #[oai(path = "/", method = "get")]
    async fn paginate(
        &self,
        name: Query<Option<String>>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        page_number: Query<u64>,
        page_size: Query<u64>,
        cxt: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<IamAppSummaryResp>> {
        let result = IamCtAppServ::paginate_apps(name.0, page_number.0, page_size.0, desc_by_create.0, desc_by_update.0, &TardisFuns::reldb().conn(), &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Delete App By Id
    #[oai(path = "/:id", method = "delete")]
    async fn delete(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut tx = TardisFuns::reldb().conn();
        tx.begin().await?;
        IamCtAppServ::delete_app(&id.0, &tx, &cxt.0).await?;
        tx.commit().await?;
        TardisResp::ok(Void {})
    }
}
