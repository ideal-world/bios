use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{param::Query, OpenApi};
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use bios_basic::rbum::dto::rbum_cert_dto::RbumCertSummaryResp;
use bios_basic::rbum::dto::rbum_filer_dto::RbumCertFilterReq;

use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::iam_constants;

pub struct IamCaCertApi;

/// App Console Cert API
#[OpenApi(prefix_path = "/ca/cert", tag = "crate::iam_enumeration::Tag::App")]
impl IamCaCertApi {
    /// Find Certs By Account Id
    #[oai(path = "/", method = "get")]
    async fn find_certs(&self, account_id: Query<String>, cxt: TardisContextExtractor) -> TardisApiResult<Vec<RbumCertSummaryResp>> {
        let funs = iam_constants::get_tardis_inst();
        let rbum_certs = IamCertServ::find_certs(
            &RbumCertFilterReq {
                rel_rbum_id: Some(account_id.0.to_string()),
                ..Default::default()
            },
            None,
            None,
            &funs,
            &cxt.0,
        )
        .await?;
        TardisResp::ok(rbum_certs)
    }
}
