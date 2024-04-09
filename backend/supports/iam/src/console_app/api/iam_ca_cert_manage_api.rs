use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::{param::Path, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use bios_basic::rbum::dto::rbum_cert_dto::RbumCertSummaryWithSkResp;
use bios_basic::rbum::dto::rbum_rel_dto::RbumRelBoneResp;

use crate::basic::dto::iam_cert_dto::IamCertModifyVisibilityRequest;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::iam_constants;
use bios_basic::helper::request_helper::try_set_real_ip_from_req_to_ctx;
use tardis::web::poem::Request;
#[derive(Clone, Default)]
pub struct IamCaCertManageApi;

/// Tenant Console Cert manage API
#[poem_openapi::OpenApi(prefix_path = "/ca/cert/manage", tag = "bios_basic::ApiTag::Tenant")]
impl IamCaCertManageApi {
    /// get manage cert
    #[oai(path = "/:id", method = "get")]
    async fn get_manage_cert(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<RbumCertSummaryWithSkResp> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let ctx = IamCertServ::use_sys_or_tenant_ctx_unsafe(ctx.0)?;
        let cert = IamCertServ::get_3th_kind_cert_by_id(&id.0, &funs, &ctx).await?;
        ctx.execute_task().await?;
        TardisResp::ok(cert)
    }

    /// get manage cert
    #[oai(path = "/:id", method = "put")]
    async fn modify_sk_visibility(&self, id: Path<String>, body: Json<IamCertModifyVisibilityRequest>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        let ctx = IamCertServ::use_sys_or_tenant_ctx_unsafe(ctx.0)?;
        funs.begin().await?;
        IamCertServ::modify_sk_visibility(&id.0, body.0, &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(Void)
    }

    /// Find Manage Certs By item Id
    #[oai(path = "/rel/:item_id", method = "get")]
    async fn find_certs(&self, item_id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Vec<RbumRelBoneResp>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let ctx = IamCertServ::use_sys_or_tenant_ctx_unsafe(ctx.0)?;
        let rbum_certs = IamCertServ::find_to_simple_rel_cert(&item_id.0, None, None, &funs, &ctx).await?;
        ctx.execute_task().await?;
        TardisResp::ok(rbum_certs)
    }
}
