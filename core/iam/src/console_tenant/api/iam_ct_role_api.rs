use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json, OpenApi};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};
use tardis::TardisFuns;

use bios_basic::rbum::dto::rbum_rel_agg_dto::RbumRelAggResp;

use crate::basic::dto::iam_role_dto::{IamRoleDetailResp, IamRoleSummaryResp};
use crate::console_tenant::dto::iam_ct_role_dto::{IamCtRoleAddReq, IamCtRoleModifyReq};
use crate::console_tenant::serv::iam_ct_role_serv::IamCtRoleServ;

pub struct IamCtRoleApi;

/// Tenant Console Role API
#[OpenApi(prefix_path = "/ct/role", tag = "bios_basic::Components::Iam")]
impl IamCtRoleApi {
    /// Add Role
    #[oai(path = "/", method = "post")]
    async fn add(&self, mut add_req: Json<IamCtRoleAddReq>, cxt: TardisContextExtractor) -> TardisApiResult<String> {
        let mut tx = TardisFuns::reldb().conn();
        tx.begin().await?;
        let result = IamCtRoleServ::add_role(&mut add_req.0, &tx, &cxt.0).await?;
        tx.commit().await?;
        TardisResp::ok(result)
    }

    /// Modify Role By Id
    #[oai(path = "/:id", method = "put")]
    async fn modify(&self, id: Path<String>, mut modify_req: Json<IamCtRoleModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut tx = TardisFuns::reldb().conn();
        tx.begin().await?;
        IamCtRoleServ::modify_role(&id.0, &mut modify_req.0, &tx, &cxt.0).await?;
        tx.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Get Role By Id
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<IamRoleDetailResp> {
        let result = IamCtRoleServ::get_role(&id.0, &TardisFuns::reldb().conn(), &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Find Roles
    #[oai(path = "/", method = "get")]
    async fn paginate(
        &self,
        name: Query<Option<String>>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        page_number: Query<u64>,
        page_size: Query<u64>,
        cxt: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<IamRoleSummaryResp>> {
        let result = IamCtRoleServ::paginate_roles(name.0, page_number.0, page_size.0, desc_by_create.0, desc_by_update.0, &TardisFuns::reldb().conn(), &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Delete Role By Id
    #[oai(path = "/:id", method = "delete")]
    async fn delete(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut tx = TardisFuns::reldb().conn();
        tx.begin().await?;
        IamCtRoleServ::delete_role(&id.0, &tx, &cxt.0).await?;
        tx.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Add Rel Account By Id
    #[oai(path = "/:id/account/:account_id", method = "put")]
    async fn add_rel_account(&self, id: Path<String>, account_id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut tx = TardisFuns::reldb().conn();
        tx.begin().await?;
        IamCtRoleServ::add_rel_account(&id.0, &account_id.0, None, None, &tx, &cxt.0).await?;
        tx.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Delete Rel Account By Id
    #[oai(path = "/:id/account/:account_id", method = "delete")]
    async fn delete_rel_account(&self, id: Path<String>, account_id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut tx = TardisFuns::reldb().conn();
        tx.begin().await?;
        IamCtRoleServ::delete_rel_account(&id.0, &account_id.0, &tx, &cxt.0).await?;
        tx.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Find Rel Accounts By Role Id
    #[oai(path = "/:id/account", method = "get")]
    async fn paginate_rel_accounts(
        &self,
        id: Path<String>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        page_number: Query<u64>,
        page_size: Query<u64>,
        cxt: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<RbumRelAggResp>> {
        let result = IamCtRoleServ::paginate_rel_accounts(&id.0, page_number.0, page_size.0, desc_by_create.0, desc_by_update.0, &TardisFuns::reldb().conn(), &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Add Rel Http Res By Id
    #[oai(path = "/:id/http-res/:http_res_id", method = "put")]
    async fn add_rel_http_res(&self, id: Path<String>, http_res_id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut tx = TardisFuns::reldb().conn();
        tx.begin().await?;
        IamCtRoleServ::add_rel_http_res(&id.0, &http_res_id.0, None, None, &tx, &cxt.0).await?;
        tx.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Delete Rel Http Res By Id
    #[oai(path = "/:id/http-res/:http_res_id", method = "delete")]
    async fn delete_rel_http_res(&self, id: Path<String>, http_res_id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut tx = TardisFuns::reldb().conn();
        tx.begin().await?;
        IamCtRoleServ::delete_rel_http_res(&id.0, &http_res_id.0, &tx, &cxt.0).await?;
        tx.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Find Rel Http Res By Role Id
    #[oai(path = "/:id/http-res", method = "get")]
    async fn paginate_rel_http_res(
        &self,
        id: Path<String>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        page_number: Query<u64>,
        page_size: Query<u64>,
        cxt: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<RbumRelAggResp>> {
        let result = IamCtRoleServ::paginate_rel_http_res(&id.0, page_number.0, page_size.0, desc_by_create.0, desc_by_update.0, &TardisFuns::reldb().conn(), &cxt.0).await?;
        TardisResp::ok(result)
    }
}
