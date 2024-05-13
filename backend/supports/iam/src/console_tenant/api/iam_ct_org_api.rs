use bios_basic::rbum::dto::rbum_filer_dto::{RbumRelFilterReq, RbumSetTreeFilterReq};
use bios_basic::rbum::dto::rbum_set_dto::RbumSetTreeResp;
use itertools::Itertools;
use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::futures::future::join_all;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use bios_basic::rbum::dto::rbum_set_item_dto::RbumSetItemDetailResp;
use bios_basic::rbum::rbum_enumeration::{RbumRelFromKind, RbumSetCateLevelQueryKind};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_rel_serv::RbumRelServ;

use crate::basic::dto::iam_set_dto::{IamSetCateAddReq, IamSetCateModifyReq, IamSetItemAddReq, IamSetItemWithDefaultSetAddReq};
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_constants;
use crate::iam_enumeration::{IamRelKind, IamSetKind};
use bios_basic::helper::request_helper::try_set_real_ip_from_req_to_ctx;
use tardis::web::poem::Request;
#[derive(Clone, Default)]
pub struct IamCtOrgApi;

/// Tenant Console Org API
/// 租户控制台组织API
///
/// Note: the current org only supports tenant level.
/// Transferring to another tenant or platform's set_id will result in permission escalation
/// 注意：当前组织仅支持租户级别。
/// 转移到其他租户或平台的set_id会导致权限升级
#[poem_openapi::OpenApi(prefix_path = "/ct/org", tag = "bios_basic::ApiTag::Tenant")]
impl IamCtOrgApi {
    /// Add Org Cate
    /// 添加组织分类
    #[oai(path = "/cate", method = "post")]
    async fn add_cate(&self, add_req: Json<IamSetCateAddReq>, set_id: Query<Option<String>>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let ctx = IamSetServ::try_get_rel_ctx_by_set_id(set_id.0, &funs, ctx.0).await?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, &funs, &ctx).await?;
        let result = IamSetServ::add_set_cate(&set_id, &add_req.0, &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Modify Org Cate By Org Cate Id
    /// 修改组织分类
    #[oai(path = "/cate/:id", method = "put")]
    async fn modify_set_cate(
        &self,
        id: Path<String>,
        modify_req: Json<IamSetCateModifyReq>,
        set_id: Query<Option<String>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let ctx = IamSetServ::try_get_rel_ctx_by_set_id(set_id.0, &funs, ctx.0).await?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        IamSetServ::modify_set_cate(&id.0, &modify_req.0, &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Find Org Tree By Current Tenant
    ///
    /// * Without parameters: Query the whole tree
    /// * ``parent_sys_code=true`` : query only the next level. This can be used to query level by level when the tree is too large
    ///
    /// 根据当前租户查找组织树
    ///
    /// * 无参数：查询整个树
    /// * ``parent_sys_code=true``：仅查询下一级。当树太大时，可以逐级查询
    #[oai(path = "/tree", method = "get")]
    async fn get_tree(
        &self,
        parent_sys_code: Query<Option<String>>,
        set_id: Query<Option<String>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<RbumSetTreeResp> {
        let funs = iam_constants::get_tardis_inst();
        let ctx = IamSetServ::try_get_rel_ctx_by_set_id(set_id.0, &funs, ctx.0).await?;
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

    /// Delete Org Cate By Org Cate Id
    /// 删除组织分类
    #[oai(path = "/cate/:id", method = "delete")]
    async fn delete_cate(&self, id: Path<String>, set_id: Query<Option<String>>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let ctx = IamSetServ::try_get_rel_ctx_by_set_id(set_id.0, &funs, ctx.0).await?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        IamSetServ::delete_set_cate(&id.0, &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Current is bound by platform
    /// 当前是否被平台绑定
    #[oai(path = "/is_bound", method = "get")]
    async fn is_bond_by_platform(&self, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<bool> {
        let mock_ctx = TardisContext {
            own_paths: "".to_string(),
            ..ctx.0.clone()
        };
        try_set_real_ip_from_req_to_ctx(request, &mock_ctx).await?;
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
        mock_ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Batch Add Org Item
    /// 批量添加组织项
    #[oai(path = "/item/batch", method = "put")]
    async fn batch_add_set_item(
        &self,
        add_req: Json<IamSetItemWithDefaultSetAddReq>,
        set_id: Query<Option<String>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<String>> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let ctx = IamSetServ::try_get_rel_ctx_by_set_id(set_id.0, &funs, ctx.0).await?;
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
    /// 查询组织项
    #[oai(path = "/item", method = "get")]
    async fn find_items(
        &self,
        cate_id: Query<Option<String>>,
        set_id: Query<Option<String>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<RbumSetItemDetailResp>> {
        let funs = iam_constants::get_tardis_inst();
        let ctx = IamSetServ::try_get_rel_ctx_by_set_id(set_id.0, &funs, ctx.0).await?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, &funs, &ctx).await?;
        let result = IamSetServ::find_set_items(Some(set_id), cate_id.0, None, None, false, None, &funs, &ctx).await?;
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
        set_id: Query<Option<String>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<TardisPage<RbumSetItemDetailResp>> {
        let funs = iam_constants::get_tardis_inst();
        let ctx = IamSetServ::try_get_rel_ctx_by_set_id(set_id.0, &funs, ctx.0).await?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, &funs, &ctx).await?;
        let result = IamSetServ::paginate_set_items(Some(set_id), cate_id.0, None, None, false, None, page_number.0, page_size.0, &funs, &ctx).await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Delete Org Item By Org Item Id
    /// 删除组织项
    #[oai(path = "/item/:id", method = "delete")]
    async fn delete_item(&self, id: Path<String>, set_id: Query<Option<String>>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let ctx = IamSetServ::try_get_rel_ctx_by_set_id(set_id.0, &funs, ctx.0).await?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        IamSetServ::delete_set_item(&id.0, &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(Void {})
    }
}
