use bios_basic::rbum::dto::rbum_rel_agg_dto::RbumRelAggResp;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json, OpenApi};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use crate::basic::dto::iam_filer_dto::IamResFilterReq;
use crate::basic::dto::iam_res_dto::{IamResAggAddReq, IamResDetailResp, IamResModifyReq};
use bios_basic::rbum::dto::rbum_set_cate_dto::RbumSetTreeResp;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_set_dto::{IamSetCateAddReq, IamSetCateModifyReq};
use crate::basic::serv::iam_res_serv::IamResServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_constants;

pub struct IamCsResApi;

/// System Console Res API
///
/// Note: the current res only supports sys level.
#[OpenApi(prefix_path = "/cs/res", tag = "crate::iam_enumeration::Tag::System")]
impl IamCsResApi {
    /// Add Res Cate
    #[oai(path = "/cate", method = "post")]
    async fn add_cate(&self, add_req: Json<IamSetCateAddReq>, cxt: TardisContextExtractor) -> TardisApiResult<String> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let set_id = IamSetServ::get_default_set_id_by_cxt(false, &funs, &cxt.0).await?;
        let result = IamSetServ::add_set_cate(&set_id, &add_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(result)
    }

    /// Modify Res Cate By Res Cate Id
    #[oai(path = "/cate/:id", method = "put")]
    async fn modify_set_cate(&self, id: Path<String>, modify_req: Json<IamSetCateModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSetServ::modify_set_cate(&id.0, &modify_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Find Res Cates
    #[oai(path = "/cate", method = "get")]
    async fn find_cates(&self, cxt: TardisContextExtractor) -> TardisApiResult<Vec<RbumSetTreeResp>> {
        let funs = iam_constants::get_tardis_inst();
        let set_id = IamSetServ::get_default_set_id_by_cxt(false, &funs, &cxt.0).await?;
        let result = IamSetServ::find_set_cates(&set_id, &funs, &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Delete Res Cate By Res Cate Id
    #[oai(path = "/cate/:id", method = "delete")]
    async fn delete_cate(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSetServ::delete_set_cate(&id.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Add Res
    #[oai(path = "/", method = "post")]
    async fn add(&self, mut add_req: Json<IamResAggAddReq>, cxt: TardisContextExtractor) -> TardisApiResult<String> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let set_id = IamSetServ::get_default_set_id_by_cxt(false, &funs, &cxt.0).await?;
        let result = IamResServ::add_agg_res(&mut add_req.0, &set_id, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(result)
    }

    /// Modify Res By Res Id
    #[oai(path = "/:id", method = "put")]
    async fn modify(&self, id: Path<String>, mut modify_req: Json<IamResModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamResServ::modify_item(&id.0, &mut modify_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Get Res By Res Id
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<IamResDetailResp> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamResServ::get_item(&id.0, &IamResFilterReq::default(), &funs, &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Delete Res By Res Id
    #[oai(path = "/:id", method = "delete")]
    async fn delete(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamResServ::delete_item_with_all_rels(&id.0, &funs, &cxt.0).await?;
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
        cxt: TardisContextExtractor,
    ) -> TardisApiResult<Vec<RbumRelAggResp>> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamResServ::find_rel_roles(&id.0, false, desc_by_create.0, desc_by_update.0, &funs, &cxt.0).await?;
        TardisResp::ok(result)
    }
}
