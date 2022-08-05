use bios_basic::rbum::dto::rbum_filer_dto::RbumSetTreeFilterReq;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use bios_basic::rbum::dto::rbum_rel_dto::RbumRelBoneResp;
use bios_basic::rbum::dto::rbum_set_dto::RbumSetTreeResp;
use bios_basic::rbum::rbum_enumeration::RbumSetCateLevelQueryKind;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_filer_dto::IamResFilterReq;
use crate::basic::dto::iam_res_dto::{IamResAggAddReq, IamResDetailResp, IamResModifyReq, IamResSummaryResp};
use crate::basic::dto::iam_set_dto::{IamSetCateAddReq, IamSetCateModifyReq};
use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::basic::serv::iam_res_serv::IamResServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_constants;
use crate::iam_enumeration::{IamRelKind, IamResKind, IamSetKind};

pub struct IamCsResApi;

/// System Console Res API
///
/// Note: the current res only supports sys level.
#[poem_openapi::OpenApi(prefix_path = "/cs/res", tag = "crate::iam_enumeration::Tag::System")]
impl IamCsResApi {
    /// Add Res Cate
    #[oai(path = "/cate", method = "post")]
    async fn add_cate(&self, add_req: Json<IamSetCateAddReq>, ctx: TardisContextExtractor) -> TardisApiResult<String> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Res, &funs, &ctx.0).await?;
        let result = IamSetServ::add_set_cate(&set_id, &add_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(result)
    }

    /// Modify Res Cate By Res Cate Id
    #[oai(path = "/cate/:id", method = "put")]
    async fn modify_set_cate(&self, id: Path<String>, modify_req: Json<IamSetCateModifyReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSetServ::modify_set_cate(&id.0, &modify_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    // Find Res
    #[oai(path = "/", method = "get")]
    async fn find(&self, desc_by_create: Query<Option<bool>>, desc_by_update: Query<Option<bool>>, ctx: TardisContextExtractor) -> TardisApiResult<Vec<IamResSummaryResp>> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamResServ::find_items(
            &IamResFilterReq {
                kind: Some(IamResKind::Api),
                ..Default::default()
            },
            desc_by_create.0,
            desc_by_update.0,
            &funs,
            &ctx.0,
        )
        .await?;
        TardisResp::ok(result)
    }

    /// Find Res Tree
    ///
    /// * Without parameters: Query the whole tree
    /// * ``parent_sys_code=true`` : query only the next level. This can be used to query level by level when the tree is too large
    #[oai(path = "/tree", method = "get")]
    async fn get_tree(&self, parent_sys_code: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<RbumSetTreeResp> {
        let funs = iam_constants::get_tardis_inst();
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Res, &funs, &ctx.0).await?;
        let result = IamSetServ::get_tree(
            &set_id,
            &mut RbumSetTreeFilterReq {
                fetch_cate_item: true,
                sys_codes: parent_sys_code.0.map(|parent_sys_code| vec![parent_sys_code]),
                sys_code_query_kind: Some(RbumSetCateLevelQueryKind::Sub),
                sys_code_query_depth: Some(1),
                ..Default::default()
            },
            &funs,
            &ctx.0,
        )
        .await?;
        TardisResp::ok(result)
    }

    /// Find Menu Tree
    #[oai(path = "/tree/menu", method = "get")]
    async fn get_menu_tree(&self, ctx: TardisContextExtractor) -> TardisApiResult<RbumSetTreeResp> {
        let funs = iam_constants::get_tardis_inst();
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Res, &funs, &ctx.0).await?;
        let result = IamSetServ::get_menu_tree(&set_id, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }

    /// Find Api Tree
    #[oai(path = "/tree/api", method = "get")]
    async fn get_api_tree(&self, ctx: TardisContextExtractor) -> TardisApiResult<RbumSetTreeResp> {
        let funs = iam_constants::get_tardis_inst();
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Res, &funs, &ctx.0).await?;
        let result = IamSetServ::get_api_tree(&set_id, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }

    /// Delete Res Cate By Res Cate Id
    #[oai(path = "/cate/:id", method = "delete")]
    async fn delete_cate(&self, id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSetServ::delete_set_cate(&id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Add Res
    #[oai(path = "/", method = "post")]
    async fn add(&self, mut add_req: Json<IamResAggAddReq>, ctx: TardisContextExtractor) -> TardisApiResult<String> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Res, &funs, &ctx.0).await?;
        let result = IamResServ::add_res_agg(&mut add_req.0, &set_id, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(result)
    }

    /// Modify Res By Res Id
    #[oai(path = "/:id", method = "put")]
    async fn modify(&self, id: Path<String>, mut modify_req: Json<IamResModifyReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamResServ::modify_item(&id.0, &mut modify_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Get Res By Res Id
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<IamResDetailResp> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamResServ::get_item(&id.0, &IamResFilterReq::default(), &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }

    /// Delete Res By Res Id
    #[oai(path = "/:id", method = "delete")]
    async fn delete(&self, id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamResServ::delete_item_with_all_rels(&id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Find Rel Roles By Res Id
    #[oai(path = "/:id/role", method = "get")]
    async fn find_rel_roles(
        &self,
        id: Path<String>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<Vec<RbumRelBoneResp>> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamResServ::find_from_simple_rel_roles(&IamRelKind::IamResRole, false, &id.0, desc_by_create.0, desc_by_update.0, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }

    /// Add Res Rel Api Res
    #[oai(path = "/:id/res/:res_api_id", method = "put")]
    async fn add_rel_res(&self, id: Path<String>, res_api_id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamRelServ::add_simple_rel(&IamRelKind::IamResApi, &res_api_id.0, &id.0, None, None, false, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Delete Res Rel Api Res
    #[oai(path = "/:id/res/:res_api_id", method = "delete")]
    async fn delete_rel_res(&self, id: Path<String>, res_api_id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamRelServ::delete_simple_rel(&IamRelKind::IamResApi, &res_api_id.0, &id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Count Rel Res By Role Id
    #[oai(path = "/:id/res/total", method = "get")]
    async fn count_rel_res(&self, id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<u64> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamRelServ::count_to_rels(&IamRelKind::IamResApi, &id.0, &funs, &ctx.0).await?;
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
        let result = IamResServ::find_to_simple_rel_roles(&IamRelKind::IamResApi, &id.0, desc_by_create.0, desc_by_update.0, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }
}
