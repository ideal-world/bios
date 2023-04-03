use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::iam_constants;
use bios_basic::rbum::dto::rbum_cert_dto::RbumCertSummaryWithSkResp;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

pub struct IamCaCertApi;

/// Common Console App API
#[poem_openapi::OpenApi(prefix_path = "/ca/cert", tag = "bios_basic::ApiTag::Common")]
impl IamCaCertApi {
    /// Find Third-kind Certs By Current Account
    #[oai(path = "/third-kind", method = "get")]
    async fn get_third_cert(&self, tenant_id: Query<Option<String>>, supplier: Query<String>, ctx: TardisContextExtractor) -> TardisApiResult<RbumCertSummaryWithSkResp> {
        let funs = iam_constants::get_tardis_inst();
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        let rbum_cert = IamCertServ::get_3th_kind_cert_by_rel_rubm_id(&ctx.owner, vec![supplier.0], &funs, &ctx).await?;
        TardisResp::ok(rbum_cert)
    }
}
