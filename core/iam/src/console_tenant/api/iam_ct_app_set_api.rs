use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use bios_basic::rbum::dto::rbum_filer_dto::RbumSetTreeFilterReq;
use bios_basic::rbum::dto::rbum_set_dto::RbumSetTreeResp;
use bios_basic::rbum::dto::rbum_set_item_dto::RbumSetItemDetailResp;

use crate::basic::dto::iam_set_dto::{IamSetCateAddReq, IamSetCateModifyReq, IamSetItemAddReq, IamSetItemWithDefaultSetAddReq};
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_constants;
use crate::iam_enumeration::IamSetKind;

pub struct IamCtAppSetApi;

/// Tenant Console App Set API
///
#[poem_openapi::OpenApi(prefix_path = "/ct/apps", tag = "crate::iam_enumeration::Tag::Tenant")]
impl IamCtAppSetApi {
    /// Add App Set Cate
    #[oai(path = "/cate", method = "post")]
    async fn add_cate(&self, add_req: Json<IamSetCateAddReq>, ctx: TardisContextExtractor) -> TardisApiResult<String> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Apps, &funs, &ctx.0).await?;
        let result = IamSetServ::add_set_cate(&set_id, &add_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(result)
    }

    /// Modify App Set Cate By App Cate Id
    #[oai(path = "/cate/:id", method = "put")]
    async fn modify_set_cate(&self, id: Path<String>, modify_req: Json<IamSetCateModifyReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSetServ::modify_set_cate(&id.0, &modify_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Find App Tree By Current Tenant
    #[oai(path = "/tree", method = "get")]
    async fn get_tree(&self, parent_cate_id: Query<Option<String>>, only_related: Query<Option<bool>>, ctx: TardisContextExtractor) -> TardisApiResult<Vec<RbumSetTreeResp>> {
        let funs = iam_constants::get_tardis_inst();
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Apps, &funs, &ctx.0).await?;
        let only_related = only_related.0.unwrap_or(false);
        let result = IamSetServ::get_tree_with_adv_filter(
            &set_id,
            parent_cate_id.0,
            &mut RbumSetTreeFilterReq {
                fetch_cate_item: true,
                hide_cate_with_empty_item: only_related,
                filter_cate_item_ids: if only_related { Some(vec![ctx.0.owner.clone()]) } else { None },
                ..Default::default()
            },
            &funs,
            &ctx.0,
        )
        .await?;
        TardisResp::ok(result)
    }

    /// Delete App Set Cate By Org Cate Id
    #[oai(path = "/cate/:id", method = "delete")]
    async fn delete_cate(&self, id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSetServ::delete_set_cate(&id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Add App Set Item (App Or Account)
    #[oai(path = "/item", method = "put")]
    async fn add_set_item(&self, add_req: Json<IamSetItemWithDefaultSetAddReq>, ctx: TardisContextExtractor) -> TardisApiResult<String> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Apps, &funs, &ctx.0).await?;
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

    /// Find App Set Items (App Or Account)
    #[oai(path = "/item", method = "get")]
    async fn find_items(&self, cate_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<Vec<RbumSetItemDetailResp>> {
        let funs = iam_constants::get_tardis_inst();
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Apps, &funs, &ctx.0).await?;
        let result = IamSetServ::find_set_items(Some(set_id), cate_id.0, None, false, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }

    /// Delete App Set Item (App Or Account) By App Set Item Id
    #[oai(path = "/item/:id", method = "delete")]
    async fn delete_item(&self, id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSetServ::delete_set_item(&id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Find App Set Items (App Or Account)
    #[oai(path = "/scope", method = "get")]
    async fn check_scope(&self, app_id: Query<String>, account_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<bool> {
        let funs = iam_constants::get_tardis_inst();
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Apps, &funs, &ctx.0).await?;
        let result = IamSetServ::check_scope(&app_id.0, &account_id.0.unwrap_or_else(|| ctx.0.owner.clone()), &set_id, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }
}
