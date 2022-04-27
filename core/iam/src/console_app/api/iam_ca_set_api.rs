use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json, OpenApi};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use bios_basic::rbum::dto::rbum_set_cate_dto::RbumSetTreeResp;
use bios_basic::rbum::dto::rbum_set_item_dto::{RbumSetItemModifyReq, RbumSetItemSummaryResp};

use crate::basic::dto::iam_set_dto::{IamSetCateAddReq, IamSetCateModifyReq, IamSetItemAddReq};
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_constants;
use crate::iam_constants::RBUM_SCOPE_LEVEL_APP;

pub struct IamCtSetApi;

/// App Console Set API
#[OpenApi(prefix_path = "/ca/set", tag = "crate::iam_enumeration::Tag::App")]
impl IamCtSetApi {
    /// Add Set Category
    #[oai(path = "/cate", method = "post")]
    async fn add_set_cate(&self, add_req: Json<IamSetCateAddReq>, is_org: Query<bool>, cxt: TardisContextExtractor) -> TardisApiResult<String> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamSetServ::add_set_cate(&add_req.0, is_org.0, RBUM_SCOPE_LEVEL_APP, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(result)
    }

    /// Modify Set Category By Id
    #[oai(path = "/cate/:id", method = "put")]
    async fn modify_set_cate(&self, id: Path<String>, modify_req: Json<IamSetCateModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSetServ::modify_set_cate(&id.0, &modify_req.0, None, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Find Set Categories
    #[oai(path = "/cate", method = "get")]
    async fn find_cates(&self, is_org: Query<bool>, cxt: TardisContextExtractor) -> TardisApiResult<Vec<RbumSetTreeResp>> {
        let result = IamSetServ::find_set_cates(is_org.0, &iam_constants::get_tardis_inst(), &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Delete Set Category By Id
    #[oai(path = "/cate/:id", method = "delete")]
    async fn delete_cate(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSetServ::delete_set_cate(&id.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Add Set Item
    #[oai(path = "/cate/:cate_id/item", method = "post")]
    async fn add_set_item(&self, cate_id: Path<String>, add_req: Json<IamSetItemAddReq>, is_org: Query<bool>, cxt: TardisContextExtractor) -> TardisApiResult<String> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamSetServ::add_set_item(&cate_id.0, &add_req.0, is_org.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(result)
    }

    /// Modify Set Item By Id
    #[oai(path = "/cate/:_/item/:id", method = "put")]
    async fn modify_set_item(&self, id: Path<String>, mut modify_req: Json<RbumSetItemModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSetServ::modify_set_item(&id.0, &mut modify_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Find Set Items
    #[oai(path = "/cate/:cate_id/item", method = "get")]
    async fn find_items(&self, cate_id: Path<String>, is_org: Query<bool>, cxt: TardisContextExtractor) -> TardisApiResult<Vec<RbumSetItemSummaryResp>> {
        let result = IamSetServ::find_set_items(&cate_id.0, is_org.0, &iam_constants::get_tardis_inst(), &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Delete Set Item By Id
    #[oai(path = "/cate/:_/item/:id", method = "delete")]
    async fn delete_item(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSetServ::delete_set_item(&id.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }
}
