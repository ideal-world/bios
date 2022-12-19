use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::iam_constants;
use bios_basic::rbum::dto::rbum_cert_dto::RbumCertSummaryWithSkResp;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::web::Path;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

pub struct IamCcCertApi;

/// Common Console Cert API
#[poem_openapi::OpenApi(prefix_path = "/cc/cert", tag = "bios_basic::ApiTag::Common")]
impl IamCcCertApi {
    /// Find Accounts
    #[oai(path = "/:account_id", method = "get")]
    async fn get_cert_by_kind_supplier(
        &self,
        account_id: Path<String>,
        kind: Query<String>,
        tenant_id: Query<Option<String>>,
        supplier: Query<Option<String>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<RbumCertSummaryWithSkResp> {
        let funs = iam_constants::get_tardis_inst();
        let supplier = supplier.0.unwrap_or_default();
        let conf_id = if let Ok(conf_id) = IamCertServ::get_cert_conf_id_by_kind_supplier(&kind.0, &supplier.clone(), tenant_id.0, &funs).await {
            Some(conf_id)
        } else {
            None
        };
        let cert = IamCertServ::get_cert_by_relrubmid_kind_supplier(&account_id.0, &kind.0, vec![supplier], conf_id, &funs, &ctx.0).await?;
        TardisResp::ok(cert)
    }
}
