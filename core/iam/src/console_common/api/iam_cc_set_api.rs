use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json, OpenApi};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use bios_basic::rbum::dto::rbum_set_cate_dto::RbumSetTreeResp;
use bios_basic::rbum::dto::rbum_set_item_dto::{RbumSetItemModifyReq, RbumSetItemSummaryResp};

use crate::basic::dto::iam_set_dto::{IamSetCateAddReq, IamSetCateModifyReq, IamSetItemAddReq};
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_constants;

pub struct IamCcSetApi;

/// Common Console Set API
#[OpenApi(prefix_path = "/cc/set", tag = "crate::iam_enumeration::Tag::Common")]
impl IamCcSetApi {
    /// Get Default Set Id By Context
    #[oai(path = "/default", method = "get")]
    async fn get_default_set_id_by_cxt(&self, is_org: Query<bool>, cxt: TardisContextExtractor) -> TardisApiResult<String> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamSetServ::get_default_set_id_by_cxt(is_org.0, &funs, &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Add Set Category
    #[oai(path = "/:set_id/cate", method = "post")]
    async fn add_set_cate(&self, set_id: Path<String>, add_req: Json<IamSetCateAddReq>, cxt: TardisContextExtractor) -> TardisApiResult<String> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamSetServ::add_set_cate(&set_id.0, &add_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(result)
    }

    /// Modify Set Category By Id
    #[oai(path = "/:_/cate/:id", method = "put")]
    async fn modify_set_cate(&self, id: Path<String>, modify_req: Json<IamSetCateModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSetServ::modify_set_cate(&id.0, &modify_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Find Set Categories
    #[oai(path = "/:set_id/cates", method = "get")]
    async fn find_cates(&self, set_id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<Vec<RbumSetTreeResp>> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamSetServ::find_set_cates(&set_id.0, &funs, &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Delete Set Category By Id
    #[oai(path = "/:_/cate/:id", method = "delete")]
    async fn delete_cate(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSetServ::delete_set_cate(&id.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Add Set Item
    #[oai(path = "/item", method = "post")]
    async fn add_set_item(&self, add_req: Json<IamSetItemAddReq>, cxt: TardisContextExtractor) -> TardisApiResult<String> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamSetServ::add_set_item(&add_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(result)
    }

    /// Modify Set Item By Id
    #[oai(path = "/item/:id", method = "put")]
    async fn modify_set_item(&self, id: Path<String>, mut modify_req: Json<RbumSetItemModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSetServ::modify_set_item(&id.0, &mut modify_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Find Set Items
    #[oai(path = "/items", method = "get")]
    async fn find_items(&self, set_id: Query<Option<String>>, cate_id: Query<Option<String>>, cxt: TardisContextExtractor) -> TardisApiResult<Vec<RbumSetItemSummaryResp>> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamSetServ::find_set_items(set_id.0, cate_id.0, &funs, &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Delete Set Item By Id
    #[oai(path = "/item/:id", method = "delete")]
    async fn delete_item(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSetServ::delete_set_item(&id.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }
}
