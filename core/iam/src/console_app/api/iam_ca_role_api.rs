use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{OpenApi, param::Path, param::Query, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use bios_basic::rbum::dto::rbum_rel_agg_dto::RbumRelAggResp;

use crate::basic::dto::iam_role_dto::{IamRoleDetailResp, IamRoleSummaryResp};
use crate::console_app::dto::iam_ca_role_dto::{IamCaRoleAddReq, IamCaRoleModifyReq};
use crate::console_app::serv::iam_ca_role_serv::IamCaRoleServ;
use crate::iam_constants;

pub struct IamCtRoleApi;

/// App Console Role API
#[OpenApi(prefix_path = "/ca/role", tag = "crate::iam_enumeration::Tag::App")]
impl IamCtRoleApi {
    /// Add Role
    #[oai(path = "/", method = "post")]
    async fn add(&self, mut add_req: Json<IamCaRoleAddReq>, cxt: TardisContextExtractor) -> TardisApiResult<String> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamCaRoleServ::add_role(&mut add_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(result)
    }

    /// Modify Role By Id
    #[oai(path = "/:id", method = "put")]
    async fn modify(&self, id: Path<String>, mut modify_req: Json<IamCaRoleModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCaRoleServ::modify_role(&id.0, &mut modify_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Get Role By Id
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<IamRoleDetailResp> {
        let result = IamCaRoleServ::get_role(&id.0, &iam_constants::get_tardis_inst(), &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Find Roles
    #[oai(path = "/", method = "get")]
    async fn paginate(
        &self,
        q_id: Query<Option<String>>,
        q_name: Query<Option<String>>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        page_number: Query<u64>,
        page_size: Query<u64>,
        cxt: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<IamRoleSummaryResp>> {
        let result = IamCaRoleServ::paginate_roles(
            q_id.0,
            q_name.0,
            page_number.0,
            page_size.0,
            desc_by_create.0,
            desc_by_update.0,
            &iam_constants::get_tardis_inst(),
            &cxt.0,
        )
        .await?;
        TardisResp::ok(result)
    }

    /// Delete Role By Id
    #[oai(path = "/:id", method = "delete")]
    async fn delete(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCaRoleServ::delete_role(&id.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Add Rel Account By Id
    #[oai(path = "/:id/account/:account_id", method = "put")]
    async fn add_rel_account(&self, id: Path<String>, account_id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCaRoleServ::add_rel_account(&id.0, &account_id.0, None, None, &funs, &cxt.0).await?;
        funs.commit().await?;
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
        let result = IamCaRoleServ::paginate_rel_accounts(
            &id.0,
            page_number.0,
            page_size.0,
            desc_by_create.0,
            desc_by_update.0,
            &iam_constants::get_tardis_inst(),
            &cxt.0,
        )
        .await?;
        TardisResp::ok(result)
    }

    /// Add Rel Http Res By Id
    #[oai(path = "/:id/http-res/:http_res_id", method = "put")]
    async fn add_rel_http_res(&self, id: Path<String>, http_res_id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCaRoleServ::add_rel_http_res(&id.0, &http_res_id.0, None, None, &funs, &cxt.0).await?;
        funs.commit().await?;
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
        let result = IamCaRoleServ::paginate_rel_http_res(
            &id.0,
            page_number.0,
            page_size.0,
            desc_by_create.0,
            desc_by_update.0,
            &iam_constants::get_tardis_inst(),
            &cxt.0,
        )
        .await?;
        TardisResp::ok(result)
    }

    /// Delete Rel By Id
    #[oai(path = "/:_/rel/:id", method = "delete")]
    async fn delete_rel_account(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCaRoleServ::delete_rel(&id.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }
}
