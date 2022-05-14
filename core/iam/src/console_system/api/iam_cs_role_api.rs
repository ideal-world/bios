use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json, OpenApi};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::dto::rbum_rel_agg_dto::RbumRelAggResp;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_filer_dto::IamRoleFilterReq;
use crate::basic::dto::iam_role_dto::{IamRoleAddReq, IamRoleDetailResp, IamRoleModifyReq, IamRoleSummaryResp};
use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::iam_constants;

pub struct IamCsRoleApi;

/// System Console Role API
#[OpenApi(prefix_path = "/cs/role", tag = "crate::iam_enumeration::Tag::System")]
impl IamCsRoleApi {
    /// Add Role
    #[oai(path = "/", method = "post")]
    async fn add(&self, mut add_req: Json<IamRoleAddReq>, cxt: TardisContextExtractor) -> TardisApiResult<String> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamRoleServ::add_item(&mut add_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(result)
    }

    /// Modify Role By Id
    #[oai(path = "/:id", method = "put")]
    async fn modify(&self, id: Path<String>, mut modify_req: Json<IamRoleModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamRoleServ::modify_item(&id.0, &mut modify_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Get Role By Id
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<IamRoleDetailResp> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamRoleServ::get_item(&id.0, &IamRoleFilterReq::default(), &funs, &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Find Roles
    #[oai(path = "/", method = "get")]
    async fn paginate(
        &self,
        id: Query<Option<String>>,
        name: Query<Option<String>>,
        with_sub: Query<Option<bool>>,
        page_number: Query<u64>,
        page_size: Query<u64>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        cxt: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<IamRoleSummaryResp>> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamRoleServ::paginate_items(
            &IamRoleFilterReq {
                basic: RbumBasicFilterReq {
                    ids: id.0.map(|id| vec![id]),
                    name: name.0,
                    own_paths: Some(cxt.0.own_paths.clone()),
                    with_sub_own_paths: with_sub.0.unwrap_or(false),
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

    /// Delete Role By Id
    #[oai(path = "/:id", method = "delete")]
    async fn delete(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamRoleServ::delete_item(&id.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Add Rel Account By Id
    #[oai(path = "/:id/account/:account_id", method = "put")]
    async fn add_rel_account(&self, id: Path<String>, account_id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamRoleServ::add_rel_account(&id.0, &account_id.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Count Rel Accounts By Role Id
    #[oai(path = "/:id/account/total", method = "get")]
    async fn count_rel_accounts(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<u64> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamRoleServ::count_rel_accounts(&id.0, &funs, &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Find Rel Accounts By Role Id
    #[oai(path = "/:id/account", method = "get")]
    async fn paginate_rel_accounts(
        &self,
        id: Path<String>,
        page_number: Query<u64>,
        page_size: Query<u64>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        cxt: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<RbumRelAggResp>> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamRoleServ::paginate_rel_accounts(&id.0, page_number.0, page_size.0, desc_by_create.0, desc_by_update.0, &funs, &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Add Rel Res By Id
    #[oai(path = "/:id/res/:res_id", method = "put")]
    async fn add_rel_res(&self, id: Path<String>, res_id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamRoleServ::add_rel_res(&id.0, &res_id.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Count Rel Res By Role Id
    #[oai(path = "/:id/res/total", method = "get")]
    async fn count_rel_res(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<u64> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamRoleServ::count_rel_res(&id.0, &funs, &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Find Rel Res By Role Id
    #[oai(path = "/:id/res", method = "get")]
    async fn paginate_rel_res(
        &self,
        id: Path<String>,
        page_number: Query<u64>,
        page_size: Query<u64>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        cxt: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<RbumRelAggResp>> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamRoleServ::paginate_rel_res(&id.0, page_number.0, page_size.0, desc_by_create.0, desc_by_update.0, &funs, &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Delete Rel By Rel Id
    #[oai(path = "/:_/rel/:id", method = "delete")]
    async fn delete_rel(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamRelServ::delete_rel(&id.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }
}
