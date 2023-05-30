use bios_basic::rbum::dto::rbum_filer_dto::{RbumRelFilterReq, RbumSetTreeFilterReq};
use bios_basic::rbum::dto::rbum_set_dto::RbumSetTreeResp;
use tardis::basic::dto::TardisContext;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use bios_basic::rbum::dto::rbum_set_item_dto::RbumSetItemDetailResp;
use bios_basic::rbum::rbum_enumeration::{RbumRelFromKind, RbumSetCateLevelQueryKind};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_rel_serv::RbumRelServ;

use crate::basic::dto::iam_set_dto::{IamSetCateAddReq, IamSetCateModifyReq, IamSetItemAddReq, IamSetItemWithDefaultSetAddReq};
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_constants;
use crate::iam_enumeration::{IamRelKind, IamSetKind};
use tardis::tokio::{self, task};
pub struct IamCtOrgApi;

/// Tenant Console Org API
///
/// Note: the current org only supports tenant level.
/// Transferring to another tenant or platform's set_id will result in permission escalation
#[poem_openapi::OpenApi(prefix_path = "/ct/org", tag = "bios_basic::ApiTag::Tenant")]
impl IamCtOrgApi {
    /// Add Org Cate
    #[oai(path = "/cate", method = "post")]
    async fn add_cate(&self, add_req: Json<IamSetCateAddReq>, set_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<String> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let ctx = IamSetServ::try_get_rel_ctx_by_set_id(set_id.0, &funs, ctx.0).await?;
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, &funs, &ctx).await?;
        let result = IamSetServ::add_set_cate(&set_id, &add_req.0, &funs, &ctx).await?;
        funs.commit().await?;
        let task_handle = task::spawn_blocking(move || tokio::runtime::Runtime::new().unwrap().block_on(ctx.execute_task()));
        let _ = task_handle.await;
        TardisResp::ok(result)
    }

    /// Modify Org Cate By Org Cate Id
    #[oai(path = "/cate/:id", method = "put")]
    async fn modify_set_cate(&self, id: Path<String>, modify_req: Json<IamSetCateModifyReq>, set_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let ctx = IamSetServ::try_get_rel_ctx_by_set_id(set_id.0, &funs, ctx.0).await?;
        IamSetServ::modify_set_cate(&id.0, &modify_req.0, &funs, &ctx).await?;
        funs.commit().await?;
        let task_handle = task::spawn_blocking(move || tokio::runtime::Runtime::new().unwrap().block_on(ctx.execute_task()));
        let _ = task_handle.await;
        TardisResp::ok(Void {})
    }

    /// Find Org Tree By Current Tenant
    ///
    /// * Without parameters: Query the whole tree
    /// * ``parent_sys_code=true`` : query only the next level. This can be used to query level by level when the tree is too large
    #[oai(path = "/tree", method = "get")]
    async fn get_tree(&self, parent_sys_code: Query<Option<String>>, set_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<RbumSetTreeResp> {
        let funs = iam_constants::get_tardis_inst();
        let ctx = IamSetServ::try_get_rel_ctx_by_set_id(set_id.0, &funs, ctx.0).await?;
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, &funs, &ctx).await?;
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
            &ctx,
        )
        .await?;
        let task_handle = task::spawn_blocking(move || tokio::runtime::Runtime::new().unwrap().block_on(ctx.execute_task()));
        let _ = task_handle.await;
        TardisResp::ok(result)
    }

    /// Delete Org Cate By Org Cate Id
    #[oai(path = "/cate/:id", method = "delete")]
    async fn delete_cate(&self, id: Path<String>, set_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let ctx = IamSetServ::try_get_rel_ctx_by_set_id(set_id.0, &funs, ctx.0).await?;
        IamSetServ::delete_set_cate(&id.0, &funs, &ctx).await?;
        funs.commit().await?;
        let task_handle = task::spawn_blocking(move || tokio::runtime::Runtime::new().unwrap().block_on(ctx.execute_task()));
        let _ = task_handle.await;
        TardisResp::ok(Void {})
    }

    /// Current is bound by platform
    #[oai(path = "/is_bound", method = "get")]
    async fn is_bond_by_platform(&self, ctx: TardisContextExtractor) -> TardisApiResult<bool> {
        let mock_ctx = TardisContext {
            own_paths: "".to_string(),
            ..ctx.0.clone()
        };
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = RbumRelServ::find_one_rbum(
            &RbumRelFilterReq {
                tag: Some(IamRelKind::IamOrgRel.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::SetCate),
                to_rbum_item_id: Some(ctx.0.own_paths.to_owned()),
                ..Default::default()
            },
            &funs,
            &mock_ctx,
        )
        .await?
        .is_some();
        funs.commit().await?;
        let task_handle = task::spawn_blocking(move || tokio::runtime::Runtime::new().unwrap().block_on(mock_ctx.execute_task()));
        let _ = task_handle.await;
        TardisResp::ok(result)
    }

    /// Batch Add Org Item
    #[oai(path = "/item/batch", method = "put")]
    async fn batch_add_set_item(&self, add_req: Json<IamSetItemWithDefaultSetAddReq>, set_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<Vec<String>> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let ctx = IamSetServ::try_get_rel_ctx_by_set_id(set_id.0, &funs, ctx.0).await?;
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, &funs, &ctx).await?;
        let split = add_req.rel_rbum_item_id.split(',').collect::<Vec<_>>();
        let mut result = vec![];
        for s in split {
            result.push(
                IamSetServ::add_set_item(
                    &IamSetItemAddReq {
                        set_id: set_id.clone(),
                        set_cate_id: add_req.set_cate_id.clone().unwrap_or_default(),
                        sort: add_req.sort,
                        rel_rbum_item_id: s.to_string(),
                    },
                    &funs,
                    &ctx,
                )
                .await?,
            );
        }
        funs.commit().await?;
        let task_handle = task::spawn_blocking(move || tokio::runtime::Runtime::new().unwrap().block_on(ctx.execute_task()));
        let _ = task_handle.await;
        TardisResp::ok(result)
    }

    /// Find Org Items
    #[oai(path = "/item", method = "get")]
    async fn find_items(&self, cate_id: Query<Option<String>>, set_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<Vec<RbumSetItemDetailResp>> {
        let funs = iam_constants::get_tardis_inst();
        let ctx = IamSetServ::try_get_rel_ctx_by_set_id(set_id.0, &funs, ctx.0).await?;
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, &funs, &ctx).await?;
        let result = IamSetServ::find_set_items(Some(set_id), cate_id.0, None, None, false, &funs, &ctx).await?;
        let task_handle = task::spawn_blocking(move || tokio::runtime::Runtime::new().unwrap().block_on(ctx.execute_task()));
        let _ = task_handle.await;
        TardisResp::ok(result)
    }

    /// Delete Org Item By Org Item Id
    #[oai(path = "/item/:id", method = "delete")]
    async fn delete_item(&self, id: Path<String>, set_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let ctx = IamSetServ::try_get_rel_ctx_by_set_id(set_id.0, &funs, ctx.0).await?;
        IamSetServ::delete_set_item(&id.0, &funs, &ctx).await?;
        funs.commit().await?;
        let task_handle = task::spawn_blocking(move || tokio::runtime::Runtime::new().unwrap().block_on(ctx.execute_task()));
        let _ = task_handle.await;
        TardisResp::ok(Void {})
    }
}
