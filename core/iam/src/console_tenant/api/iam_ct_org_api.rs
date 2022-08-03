use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use bios_basic::rbum::dto::rbum_set_dto::RbumSetTreeResp;
use bios_basic::rbum::dto::rbum_set_item_dto::RbumSetItemDetailResp;

use crate::basic::dto::iam_set_dto::{IamSetCateAddReq, IamSetCateModifyReq, IamSetItemAddReq, IamSetItemWithDefaultSetAddReq};
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_constants;

pub struct IamCtOrgApi;

/// Tenant Console Org API
///
/// Note: the current org only supports tenant level.
#[poem_openapi::OpenApi(prefix_path = "/ct/org", tag = "crate::iam_enumeration::Tag::Tenant")]
impl IamCtOrgApi {
    /// Add Org Cate
    #[oai(path = "/cate", method = "post")]
    async fn add_cate(&self, add_req: Json<IamSetCateAddReq>, ctx: TardisContextExtractor) -> TardisApiResult<String> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let set_id = IamSetServ::get_default_set_id_by_ctx(true, &funs, &ctx.0).await?;
        let result = IamSetServ::add_set_cate(&set_id, &add_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(result)
    }

    /// Modify Org Cate By Org Cate Id
    #[oai(path = "/cate/:id", method = "put")]
    async fn modify_set_cate(&self, id: Path<String>, modify_req: Json<IamSetCateModifyReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSetServ::modify_set_cate(&id.0, &modify_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Find Org Tree By Current Tenant
    #[oai(path = "/tree", method = "get")]
    async fn get_tree(&self, parent_cate_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<Vec<RbumSetTreeResp>> {
        let funs = iam_constants::get_tardis_inst();
        let set_id = IamSetServ::get_default_set_id_by_ctx(true, &funs, &ctx.0).await?;
        let result = IamSetServ::get_tree(&set_id, parent_cate_id.0, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }

    /// Delete Org Cate By Org Cate Id
    #[oai(path = "/cate/:id", method = "delete")]
    async fn delete_cate(&self, id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSetServ::delete_set_cate(&id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Add Org Item
    #[oai(path = "/item", method = "put")]
    async fn add_set_item(&self, add_req: Json<IamSetItemWithDefaultSetAddReq>, ctx: TardisContextExtractor) -> TardisApiResult<String> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let set_id = IamSetServ::get_default_set_id_by_ctx(true, &funs, &ctx.0).await?;
        let result = IamSetServ::add_set_item(
            &IamSetItemAddReq {
                set_id,
                set_cate_id: add_req.set_cate_id.to_string(),
                sort: add_req.sort,
                rel_rbum_item_id: add_req.rel_rbum_item_id.to_string(),
            },
            &funs,
            &ctx.0,
        )
        .await?;
        funs.commit().await?;
        TardisResp::ok(result)
    }

    /// Find Org Items
    #[oai(path = "/item", method = "get")]
    async fn find_items(&self, cate_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<Vec<RbumSetItemDetailResp>> {
        let funs = iam_constants::get_tardis_inst();
        let set_id = IamSetServ::get_default_set_id_by_ctx(true, &funs, &ctx.0).await?;
        let result = IamSetServ::find_set_items(Some(set_id), cate_id.0, None, false, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }

    /// Delete Org Item By Org Item Id
    #[oai(path = "/item/:id", method = "delete")]
    async fn delete_item(&self, id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSetServ::delete_set_item(&id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }
}
