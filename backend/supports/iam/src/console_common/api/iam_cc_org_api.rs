use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_constants;
use bios_basic::helper::request_helper::try_set_real_ip_from_req_to_ctx;
use bios_basic::rbum::dto::rbum_filer_dto::RbumSetTreeFilterReq;
use bios_basic::rbum::dto::rbum_set_dto::{RbumSetTreeCateNodeResp, RbumSetTreeResp};
use bios_basic::rbum::rbum_enumeration::RbumSetCateLevelQueryKind;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

#[derive(Clone, Default)]
pub struct IamCcOrgApi;

/// Common Console Org API
/// 通用控制台组织API
///
/// Note: the current org only supports tenant level.
/// 注意：当前组织仅支持租户级别
#[poem_openapi::OpenApi(prefix_path = "/cc/org", tag = "bios_basic::ApiTag::Common")]
impl IamCcOrgApi {
    /// Find Org Tree By Current Tenant
    /// 查找组织树
    ///
    /// * Without parameters: Query the whole tree
    /// * ``parent_sys_code=true`` : query only the next level. This can be used to query level by level when the tree is too large
    /// * 无参数：查询整个树
    /// * ``parent_sys_code=true``：仅查询下一级，当树太大时可以用来逐级查询
    #[oai(path = "/tree", method = "get")]
    async fn get_tree(
        &self,
        parent_sys_code: Query<Option<String>>,
        tenant_id: Query<Option<String>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<RbumSetTreeResp> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        let code = if ctx.own_paths.is_empty() {
            IamSetServ::get_default_org_code_by_system()
        } else {
            IamSetServ::get_default_org_code_by_tenant(&funs, &ctx)?
        };
        let set_id = IamSetServ::get_set_id_by_code(&code, true, &funs, &ctx).await?;
        let result = IamSetServ::get_tree(
            &set_id,
            &mut RbumSetTreeFilterReq {
                fetch_cate_item: true,
                hide_item_with_disabled: true,
                sys_codes: parent_sys_code.0.map(|parent_sys_code| vec![parent_sys_code]),
                sys_code_query_kind: Some(RbumSetCateLevelQueryKind::CurrentAndSub),
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

    /// Find Org Tree By Current Tenant-Directory to Tree
    /// 查找组织树-目录转成树
    ///
    /// * Without parameters: Query the whole tree
    /// * ``parent_sys_code=true`` : query only the next level. This can be used to query level by level when the tree is too large
    /// * 无参数：查询整个树
    /// * ``parent_sys_code=true``：仅查询下一级，当树太大时可以用来逐级查询
    #[oai(path = "/tree/cate", method = "get")]
    async fn get_tree_cate(
        &self,
        parent_sys_code: Query<Option<String>>,
        tenant_id: Query<Option<String>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<RbumSetTreeCateNodeResp>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        let code = if ctx.own_paths.is_empty() {
            IamSetServ::get_default_org_code_by_system()
        } else {
            IamSetServ::get_default_org_code_by_tenant(&funs, &ctx)?
        };
        let set_id = IamSetServ::get_set_id_by_code(&code, true, &funs, &ctx).await?;
        let result = IamSetServ::get_tree(
            &set_id,
            &mut RbumSetTreeFilterReq {
                fetch_cate_item: true,
                hide_item_with_disabled: true,
                sys_codes: parent_sys_code.0.map(|parent_sys_code| vec![parent_sys_code]),
                sys_code_query_kind: Some(RbumSetCateLevelQueryKind::CurrentAndSub),
                sys_code_query_depth: Some(1),
                ..Default::default()
            },
            &funs,
            &ctx,
        )
        .await?;
        ctx.execute_task().await?;
        TardisResp::ok(result.to_trees().cate_tree)
    }

    /// Find Org Cate Name By Cate Ids
    ///
    /// Return format: ["<id>,<name>"]
    #[oai(path = "/cate_name", method = "get")]
    async fn find_set_cate_name_by_cate_ids(
        &self,
        // Cate Ids, multiple ids separated by ,
        ids: Query<String>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<String>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let ids = ids.0.split(',').map(|s| s.to_string()).collect();
        let result = IamSetServ::find_set_cate_name_by_cate_ids(ids, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }
}
