use std::thread::sleep;
use std::time::Duration;

use crate::basic::dto::iam_set_dto::{IamSetCateAddReq, IamSetCateModifyReq, IamSetItemAddReq, IamSetItemWithDefaultSetAddReq};
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::console_system::serv::iam_cs_org_serv::IamCsOrgServ;
use crate::iam_constants;
use crate::iam_enumeration::{IamRelKind, IamSetKind};
use bios_basic::rbum::dto::rbum_filer_dto::{RbumRelFilterReq, RbumSetTreeFilterReq};
use bios_basic::rbum::dto::rbum_set_dto::RbumSetTreeResp;
use bios_basic::rbum::dto::rbum_set_item_dto::RbumSetItemDetailResp;
use bios_basic::rbum::rbum_enumeration::{RbumRelFromKind, RbumScopeLevelKind, RbumSetCateLevelQueryKind};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_rel_serv::RbumRelServ;
use tardis::tokio::{self, task};
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

pub struct IamCsOrgApi;
pub struct IamCsOrgItemApi;

/// System Console Org API
#[poem_openapi::OpenApi(prefix_path = "/cs/org", tag = "bios_basic::ApiTag::System")]
impl IamCsOrgApi {
    /// Find Org Tree
    ///
    /// * Without parameters: Query the whole tree
    /// * ``parent_sys_code=true`` : query only the next level. This can be used to query level by level when the tree is too large
    #[oai(path = "/tree", method = "get")]
    async fn get_tree(&self, parent_sys_code: Query<Option<String>>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<RbumSetTreeResp> {
        let funs = iam_constants::get_tardis_inst();
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
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
    /// Add Org Cate
    #[oai(path = "/cate", method = "post")]
    async fn add_cate(&self, mut add_req: Json<IamSetCateAddReq>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<String> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamCsOrgServ::add_set_cate(tenant_id.0, &mut add_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        let task_handle = task::spawn_blocking(move || tokio::runtime::Runtime::new().unwrap().block_on(ctx.0.execute_task()));
        let _ = task_handle.await;
        TardisResp::ok(result)
    }

    /// Modify Org Cate By Org Cate Id
    #[oai(path = "/cate/:id", method = "put")]
    async fn modify_set_cate(
        &self,
        id: Path<String>,
        tenant_id: Query<Option<String>>,
        modify_req: Json<IamSetCateModifyReq>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        IamSetServ::modify_set_cate(&id.0, &modify_req.0, &funs, &ctx).await?;
        funs.commit().await?;
        let task_handle = task::spawn_blocking(move || tokio::runtime::Runtime::new().unwrap().block_on(ctx.execute_task()));
        let _ = task_handle.await;
        TardisResp::ok(Void {})
    }

    /// Delete Org Cate By Org Cate Id
    #[oai(path = "/cate/:id", method = "delete")]
    async fn delete_cate(&self, id: Path<String>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        IamSetServ::delete_set_cate(&id.0, &funs, &ctx).await?;
        funs.commit().await?;
        let task_handle = task::spawn_blocking(move || tokio::runtime::Runtime::new().unwrap().block_on(ctx.execute_task()));
        let _ = task_handle.await;
        sleep(Duration::from_millis(100));
        TardisResp::ok(Void {})
    }

    /// Import Tenant Org
    ///
    /// id -> set_cate_id
    /// tenant_id -> tenant_id
    /// 导入租户组织,不支持换绑
    /// 如果平台绑定的节点下有其他节点，那么全部剪切到租户层，解绑的时候需要拷贝一份去平台，并且保留租户的节点
    #[oai(path = "/cate/:id/rel/tenant/:tenant_id", method = "post")]
    async fn bind_cate_with_platform(&self, id: Path<String>, tenant_id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCsOrgServ::bind_cate_with_tenant(&id.0, &tenant_id.0, &IamSetKind::Org, &funs, &ctx.0).await?;
        funs.commit().await?;
        let task_handle = task::spawn_blocking(move || tokio::runtime::Runtime::new().unwrap().block_on(ctx.0.execute_task()));
        let _ = task_handle.await;
        TardisResp::ok(Void {})
    }

    /// Unbind Tenant Org
    #[oai(path = "/cate/:id/rel", method = "delete")]
    async fn unbind_cate_with_tenant(&self, id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let old_rel = RbumRelServ::find_one_rbum(
            &RbumRelFilterReq {
                basic: Default::default(),
                tag: Some(IamRelKind::IamOrgRel.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::SetCate),
                from_rbum_id: Some(id.0.clone()),
                from_rbum_scope_levels: None,
                to_rbum_item_id: None,
                to_rbum_item_scope_levels: None,
                to_own_paths: None,
                ext_eq: None,
                ext_like: None,
            },
            &funs,
            &ctx.0,
        )
        .await?;
        if let Some(old_rel) = old_rel {
            IamCsOrgServ::unbind_cate_with_tenant(old_rel, &funs, &ctx.0).await?;
        }
        funs.commit().await?;
        let task_handle = task::spawn_blocking(move || tokio::runtime::Runtime::new().unwrap().block_on(ctx.0.execute_task()));
        let _ = task_handle.await;
        TardisResp::ok(Void {})
    }

    /// Query tenant IDs that have already been bound
    #[oai(path = "/tenant/rel", method = "get")]
    async fn find_rel_tenant_org(&self, ctx: TardisContextExtractor) -> TardisApiResult<Vec<String>> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamCsOrgServ::find_rel_tenant_org(&funs, &ctx.0).await?;
        funs.commit().await?;
        let task_handle = task::spawn_blocking(move || tokio::runtime::Runtime::new().unwrap().block_on(ctx.0.execute_task()));
        let _ = task_handle.await;
        TardisResp::ok(result)
    }
}
/// System Console Org Item API
#[poem_openapi::OpenApi(prefix_path = "/cs/org/item", tag = "bios_basic::ApiTag::System")]
impl IamCsOrgItemApi {
    /// Batch Add Org Item
    #[oai(path = "/batch", method = "put")]
    async fn batch_add_set_item(
        &self,
        add_req: Json<IamSetItemWithDefaultSetAddReq>,
        tenant_id: Query<Option<String>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<Vec<String>> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
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
    #[oai(path = "/", method = "get")]
    async fn find_items(&self, cate_id: Query<Option<String>>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<Vec<RbumSetItemDetailResp>> {
        let funs = iam_constants::get_tardis_inst();
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0.clone())?;
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, &funs, &ctx).await?;
        let scope_level = if tenant_id.is_none() || tenant_id.0.clone().unwrap().is_empty() {
            None
        } else {
            Some(RbumScopeLevelKind::Root)
        };
        let result = IamSetServ::find_set_items(Some(set_id), cate_id.0, None, scope_level, false, &funs, &ctx).await?;
        let task_handle = task::spawn_blocking(move || tokio::runtime::Runtime::new().unwrap().block_on(ctx.execute_task()));
        let _ = task_handle.await;
        TardisResp::ok(result)
    }

    /// Delete Org Item By Org Item Id
    #[oai(path = "/:id", method = "delete")]
    async fn delete_item(&self, id: Path<String>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        IamSetServ::delete_set_item(&id.0, &funs, &ctx).await?;
        funs.commit().await?;
        let task_handle = task::spawn_blocking(move || tokio::runtime::Runtime::new().unwrap().block_on(ctx.execute_task()));
        let _ = task_handle.await;
        TardisResp::ok(Void {})
    }
}
