use bios_basic::process::task_processor::TaskProcessor;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp};

use crate::basic::dto::iam_filer_dto::IamTenantFilterReq;
use crate::basic::dto::iam_tenant_dto::{IamTenantAggAddReq, IamTenantAggDetailResp, IamTenantAggModifyReq, IamTenantSummaryResp};
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::iam_constants;
use bios_basic::helper::request_helper::add_remote_ip;
use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use tardis::web::poem::Request;
#[derive(Clone, Default)]
pub struct IamCsTenantApi;

/// System Console Tenant API
#[poem_openapi::OpenApi(prefix_path = "/cs/tenant", tag = "bios_basic::ApiTag::System")]
impl IamCsTenantApi {
    /// Add Tenant
    #[oai(path = "/", method = "post")]
    async fn add(&self, add_req: Json<IamTenantAggAddReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        add_remote_ip(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamTenantServ::add_tenant_agg(&add_req.0, &funs, &ctx.0).await?.0;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Modify Tenant By Tenant Id
    ///
    /// When code = 202, the return value is the asynchronous task id
    #[oai(path = "/:id", method = "put")]
    async fn modify(
        &self,
        id: Path<String>,
        tenant_id: Query<Option<String>>,
        modify_req: Json<IamTenantAggModifyReq>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Option<String>> {
        let mut funs = iam_constants::get_tardis_inst();
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0.clone())?;
        add_remote_ip(request, &ctx).await?;
        funs.begin().await?;
        IamTenantServ::modify_tenant_agg(&id.0, &modify_req.0, &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        if let Some(task_id) = TaskProcessor::get_task_id_with_ctx(&ctx).await? {
            TardisResp::accepted(Some(task_id))
        } else {
            TardisResp::ok(None)
        }
    }

    /// Get Tenant By Tenant Id
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamTenantAggDetailResp> {
        let funs = iam_constants::get_tardis_inst();
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0.clone())?;
        add_remote_ip(request, &ctx).await?;
        let result = IamTenantServ::get_tenant_agg(
            &id.0,
            &IamTenantFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            &funs,
            &ctx,
        )
        .await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Tenants
    #[oai(path = "/", method = "get")]
    async fn paginate(
        &self,
        id: Query<Option<String>>,
        name: Query<Option<String>>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<TardisPage<IamTenantSummaryResp>> {
        add_remote_ip(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamTenantServ::paginate_items(
            &IamTenantFilterReq {
                basic: RbumBasicFilterReq {
                    ids: id.0.map(|id| vec![id]),
                    name: name.0,
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            page_number.0,
            page_size.0,
            desc_by_create.0,
            desc_by_update.0,
            &funs,
            &ctx.0,
        )
        .await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }
}
