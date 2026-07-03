use bios_basic::helper::request_helper::try_set_real_ip_from_req_to_ctx;
use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use crate::basic::dto::iam_filer_dto::IamPublishSystemFilterReq;
use crate::basic::dto::iam_publish_system_dto::{
    IamPublishSystemAddReq, IamPublishSystemDeleteResp, IamPublishSystemDetailResp, IamPublishSystemModifyReq, IamPublishSystemSummaryResp,
};
use crate::basic::serv::iam_publish_system_serv::IamPublishSystemServ;
use crate::iam_constants;

#[derive(Clone, Default)]
pub struct IamCcPublishSystemApi;

/// Common Console Publish System API
/// 通用控制台发布系统API
#[poem_openapi::OpenApi(prefix_path = "/cc/publish_system", tag = "bios_basic::ApiTag::Common")]
impl IamCcPublishSystemApi {
    /// Add Publish System
    /// 添加发布系统
    #[oai(path = "/", method = "post")]
    async fn add(&self, mut add_req: Json<IamPublishSystemAddReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamPublishSystemServ::add_item(&mut add_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Modify Publish System
    /// 修改发布系统
    #[oai(path = "/:id", method = "put")]
    async fn modify(&self, id: Path<String>, mut modify_req: Json<IamPublishSystemModifyReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamPublishSystemServ::modify_item(&id.0, &mut modify_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Get Publish System By Id
    /// 根据ID获取发布系统
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamPublishSystemDetailResp> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamPublishSystemServ::get_item(
            &id.0,
            &IamPublishSystemFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            &funs,
            &ctx.0,
        )
        .await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Delete Publish System By Id
    /// 删除发布系统；若存在关联产品则拒绝删除并返回关联产品 ID
    #[oai(path = "/:id", method = "delete")]
    async fn delete(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamPublishSystemDeleteResp> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamPublishSystemServ::delete_checked(&id.0, &funs, &ctx.0).await?;
        if result.deleted {
            funs.commit().await?;
        }
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Paginate Publish Systems
    /// 分页查询发布系统
    #[oai(path = "/", method = "get")]
    async fn paginate(
        &self,
        id: Query<Option<String>>,
        ids: Query<Option<String>>,
        name: Query<Option<String>>,
        sys_ident: Query<Option<String>>,
        rel_tenant_id: Query<Option<String>>,
        tenant_id: Query<Option<String>>,
        scope_level: Query<Option<bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind>>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        with_sub: Query<Option<bool>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<TardisPage<IamPublishSystemSummaryResp>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let ids = id.0.map(|id| vec![id]).or_else(|| ids.0.map(|s| s.split(',').map(str::to_string).collect::<Vec<String>>()));
        let mut filter = IamPublishSystemFilterReq {
            basic: RbumBasicFilterReq {
                ids,
                name: name.0,
                scope_level: scope_level.0,
                with_sub_own_paths: with_sub.0.unwrap_or(false),
                ..Default::default()
            },
            sys_ident: sys_ident.0,
            ..Default::default()
        };
        if let Some(tenant_id) = tenant_id.0.or(rel_tenant_id.0) {
            filter.basic.own_paths = Some(tenant_id.clone());
            filter.rel = Some(IamPublishSystemServ::with_tenant_rel_filter(&tenant_id));
            // filter.basic.ignore_scope = true;
        }
        let result = IamPublishSystemServ::paginate_items(&filter, page_number.0, page_size.0, desc_by_create.0, desc_by_update.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }
}
