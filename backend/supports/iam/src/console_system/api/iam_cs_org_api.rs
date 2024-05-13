use crate::basic::dto::iam_set_dto::{IamSetCateAddReq, IamSetCateModifyReq, IamSetItemAddReq, IamSetItemWithDefaultSetAddReq};
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::console_system::serv::iam_cs_org_serv::IamCsOrgServ;
use crate::iam_constants;
use crate::iam_enumeration::{IamRelKind, IamSetKind};
use bios_basic::helper::request_helper::try_set_real_ip_from_req_to_ctx;
use bios_basic::rbum::dto::rbum_filer_dto::{RbumRelFilterReq, RbumSetTreeFilterReq};
use bios_basic::rbum::dto::rbum_set_dto::RbumSetTreeResp;
use bios_basic::rbum::dto::rbum_set_item_dto::RbumSetItemDetailResp;
use bios_basic::rbum::rbum_enumeration::{RbumRelFromKind, RbumScopeLevelKind, RbumSetCateLevelQueryKind};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_rel_serv::RbumRelServ;
use itertools::Itertools;
use tardis::basic::error::TardisError;
use tardis::futures::future::join_all;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};
#[derive(Clone, Default)]
pub struct IamCsOrgApi;
#[derive(Clone, Default)]
pub struct IamCsOrgItemApi;

/// System Console Org API
/// 系统控制台组织API
#[poem_openapi::OpenApi(prefix_path = "/cs/org", tag = "bios_basic::ApiTag::System")]
impl IamCsOrgApi {
    /// Find Org Tree
    /// 查找组织树
    ///
    /// * Without parameters: Query the whole tree
    /// * ``parent_sys_code=true`` : query only the next level. This can be used to query level by level when the tree is too large
    /// * 无参数：查询整个树
    /// * ``parent_sys_code=true`` : 仅查询下一级，当树太大时可以用来逐级查询
    #[oai(path = "/tree", method = "get")]
    async fn get_tree(
        &self,
        parent_sys_code: Query<Option<String>>,
        tenant_id: Query<Option<String>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<RbumSetTreeResp> {
        let funs = iam_constants::get_tardis_inst();
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
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
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }
    /// Add Org Cate
    /// 添加组织分类
    #[oai(path = "/cate", method = "post")]
    async fn add_cate(&self, mut add_req: Json<IamSetCateAddReq>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamCsOrgServ::add_set_cate(tenant_id.0, &mut add_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Modify Org Cate By Org Cate Id
    /// 修改组织分类
    #[oai(path = "/cate/:id", method = "put")]
    async fn modify_set_cate(
        &self,
        id: Path<String>,
        tenant_id: Query<Option<String>>,
        modify_req: Json<IamSetCateModifyReq>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        IamSetServ::modify_set_cate(&id.0, &modify_req.0, &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Delete Org Cate By Org Cate Id
    /// 删除组织分类
    #[oai(path = "/cate/:id", method = "delete")]
    async fn delete_cate(&self, id: Path<String>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        IamSetServ::delete_set_cate(&id.0, &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Import Tenant Org
    ///
    /// id -> set_cate_id
    /// tenant_id -> tenant_id
    /// 导入租户组织,不支持换绑
    /// 如果平台绑定的节点下有其他节点，那么全部剪切到租户层，解绑的时候需要拷贝一份去平台，并且保留租户的节点
    #[oai(path = "/cate/:id/rel/tenant/:tenant_id", method = "post")]
    async fn bind_cate_with_platform(&self, id: Path<String>, tenant_id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCsOrgServ::bind_cate_with_tenant(&id.0, &tenant_id.0, &IamSetKind::Org, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Unbind Tenant Org
    /// 解绑租户组织
    #[oai(path = "/cate/:id/rel", method = "delete")]
    async fn unbind_cate_with_tenant(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
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
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Query tenant IDs that have already been bound
    /// 查询已经绑定的租户ID
    #[oai(path = "/tenant/rel", method = "get")]
    async fn find_rel_tenant_org(&self, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Vec<String>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamCsOrgServ::find_rel_tenant_org(&funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }
}
/// System Console Org Item API
/// 系统控制台组织项API
#[poem_openapi::OpenApi(prefix_path = "/cs/org", tag = "bios_basic::ApiTag::System")]
impl IamCsOrgItemApi {
    /// Batch Add Org Item
    /// 批量添加组织项
    #[oai(path = "/item/batch", method = "put")]
    async fn batch_add_set_item(
        &self,
        add_req: Json<IamSetItemWithDefaultSetAddReq>,
        tenant_id: Query<Option<String>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<String>> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, &funs, &ctx).await?;
        let result = join_all(
            add_req
                .rel_rbum_item_id
                .split(',')
                .map(|item_id| async {
                    IamSetServ::add_set_item(
                        &IamSetItemAddReq {
                            set_id: set_id.clone(),
                            set_cate_id: add_req.set_cate_id.clone().unwrap_or_default(),
                            sort: add_req.sort,
                            rel_rbum_item_id: item_id.to_string(),
                        },
                        &funs,
                        &ctx,
                    )
                    .await
                })
                .collect_vec(),
        )
        .await
        .into_iter()
        .collect::<Result<Vec<String>, TardisError>>()?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Org Items
    /// 查找组织项
    #[oai(path = "/item", method = "get")]
    async fn find_items(
        &self,
        cate_id: Query<Option<String>>,
        tenant_id: Query<Option<String>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<RbumSetItemDetailResp>> {
        let funs = iam_constants::get_tardis_inst();
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0.clone())?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, &funs, &ctx).await?;
        let scope_level = if tenant_id.is_none() || tenant_id.0.clone().unwrap_or_default().is_empty() {
            None
        } else {
            Some(RbumScopeLevelKind::Root)
        };
        let result = IamSetServ::find_set_items(Some(set_id), cate_id.0, None, scope_level, false, None, &funs, &ctx).await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    /// paginate Org Items
    ///
    /// 分页获取组织项
    #[oai(path = "/", method = "get")]
    async fn paginate(
        &self,
        cate_id: Query<Option<String>>,
        tenant_id: Query<Option<String>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<TardisPage<RbumSetItemDetailResp>> {
        let funs = iam_constants::get_tardis_inst();
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0.clone())?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, &funs, &ctx).await?;
        let scope_level = if tenant_id.is_none() || tenant_id.0.clone().unwrap_or_default().is_empty() {
            None
        } else {
            Some(RbumScopeLevelKind::Root)
        };
        let result = IamSetServ::paginate_set_items(Some(set_id), cate_id.0, None, scope_level, false, None, page_number.0, page_size.0, &funs, &ctx).await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Delete Org Item By Org Item Id
    /// 删除组织项
    #[oai(path = "/item/:id", method = "delete")]
    async fn delete_item(&self, id: Path<String>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        IamSetServ::delete_set_item(&id.0, &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(Void {})
    }
}
