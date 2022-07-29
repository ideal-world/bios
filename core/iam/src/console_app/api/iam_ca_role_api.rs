use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemRelFilterReq};
use bios_basic::rbum::dto::rbum_rel_dto::RbumRelBoneResp;
use bios_basic::rbum::rbum_enumeration::RbumRelFromKind;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_filer_dto::{IamAccountFilterReq, IamRoleFilterReq};
use crate::basic::dto::iam_role_dto::{IamRoleAggAddReq, IamRoleAggModifyReq, IamRoleDetailResp, IamRoleSummaryResp};
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_app_serv::IamAppServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::iam_constants;
use crate::iam_constants::RBUM_SCOPE_LEVEL_APP;
use crate::iam_enumeration::IamRelKind;

pub struct IamCaRoleApi;

/// App Console Role API
#[poem_openapi::OpenApi(prefix_path = "/ca/role", tag = "crate::iam_enumeration::Tag::App")]
impl IamCaRoleApi {
    /// Add Role
    #[oai(path = "/", method = "post")]
    async fn add(&self, mut add_req: Json<IamRoleAggAddReq>, ctx: TardisContextExtractor) -> TardisApiResult<String> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamRoleServ::add_role_agg(&mut add_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(result)
    }

    /// Modify Role By Role Id
    #[oai(path = "/:id", method = "put")]
    async fn modify(&self, id: Path<String>, mut modify_req: Json<IamRoleAggModifyReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamRoleServ::modify_role_agg(&id.0, &mut modify_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Get Role By Role Id
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<IamRoleDetailResp> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamRoleServ::get_item(
            &id.0,
            &IamRoleFilterReq {
                basic: RbumBasicFilterReq {
                    // Only fetch app-level roles
                    scope_level: Some(RBUM_SCOPE_LEVEL_APP),
                    ..Default::default()
                },
                ..Default::default()
            },
            &funs,
            &ctx.0,
        )
        .await?;
        TardisResp::ok(result)
    }

    /// Find Roles
    #[oai(path = "/", method = "get")]
    async fn paginate(
        &self,
        id: Query<Option<String>>,
        name: Query<Option<String>>,
        page_number: Query<u64>,
        page_size: Query<u64>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<IamRoleSummaryResp>> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamRoleServ::paginate_items(
            &IamRoleFilterReq {
                basic: RbumBasicFilterReq {
                    // Only fetch app-level roles
                    scope_level: Some(RBUM_SCOPE_LEVEL_APP),
                    ids: id.0.map(|id| vec![id]),
                    name: name.0,
                    ..Default::default()
                },
                ..Default::default()
            },
            page_number.0,
            page_size.0,
            desc_by_create.0,
            desc_by_update.0,
            &funs,
            &ctx.0,
        )
        .await?;
        TardisResp::ok(result)
    }

    /// Delete Role By Role Id
    #[oai(path = "/:id", method = "delete")]
    async fn delete(&self, id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamRoleServ::delete_item(&id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Add Role Rel Account
    #[oai(path = "/:id/account/:account_id", method = "put")]
    async fn add_rel_account(&self, id: Path<String>, account_id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let app_id = IamAppServ::get_id_by_ctx(&ctx.0, &funs)?;
        IamAppServ::add_rel_account(&app_id, &account_id.0, true, &funs, &ctx.0).await?;
        IamRoleServ::add_rel_account(&id.0, &account_id.0, Some(RBUM_SCOPE_LEVEL_APP), &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Delete Role Rel Account
    #[oai(path = "/:id/account/:account_id", method = "delete")]
    async fn delete_rel_account(&self, id: Path<String>, account_id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamRoleServ::delete_rel_account(&id.0, &account_id.0, Some(RBUM_SCOPE_LEVEL_APP), &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Count Rel Accounts By Role Id
    #[oai(path = "/:id/account/total", method = "get")]
    async fn count_rel_accounts(&self, id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<u64> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamAccountServ::count_items(
            &IamAccountFilterReq {
                rel: IamAppServ::with_app_rel_filter(&ctx.0, &funs)?,
                rel2: Some(RbumItemRelFilterReq {
                    rel_by_from: true,
                    tag: Some(IamRelKind::IamAccountRole.to_string()),
                    from_rbum_kind: Some(RbumRelFromKind::Item),
                    rel_item_id: Some(id.0.to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            &funs,
            &ctx.0,
        )
        .await?;
        TardisResp::ok(result)
    }

    /// Add Role Rel Res
    #[oai(path = "/:id/res/:res_id", method = "put")]
    async fn add_rel_res(&self, id: Path<String>, res_id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamRoleServ::add_rel_res(&id.0, &res_id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Delete Role Rel Res
    #[oai(path = "/:id/res/:res_id", method = "delete")]
    async fn delete_rel_res(&self, id: Path<String>, res_id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamRoleServ::delete_rel_res(&id.0, &res_id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Count Rel Res By Role Id
    #[oai(path = "/:id/res/total", method = "get")]
    async fn count_rel_res(&self, id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<u64> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamRoleServ::count_rel_res(&id.0, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }

    /// Find Rel Res By Role Id
    #[oai(path = "/:id/res", method = "get")]
    async fn find_rel_res(
        &self,
        id: Path<String>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<Vec<RbumRelBoneResp>> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamRoleServ::find_simple_rel_res(&id.0, desc_by_create.0, desc_by_update.0, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }
}
