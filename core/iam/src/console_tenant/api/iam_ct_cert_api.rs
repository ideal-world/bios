use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{param::Query, OpenApi};
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use bios_basic::rbum::dto::rbum_cert_dto::RbumCertSummaryResp;
use bios_basic::rbum::dto::rbum_filer_dto::RbumCertFilterReq;
use bios_basic::rbum::serv::rbum_cert_serv::RbumCertServ;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;

use crate::iam_constants;

pub struct IamCtCertApi;

/// Tenant Console Cert API
#[OpenApi(prefix_path = "/ct/cert", tag = "crate::iam_enumeration::Tag::Tenant")]
impl IamCtCertApi {
    /// Find Certs
    #[oai(path = "/", method = "get")]
    async fn find_certs(&self, account_id: Query<String>, cxt: TardisContextExtractor) -> TardisApiResult<Vec<RbumCertSummaryResp>> {
        let funs = iam_constants::get_tardis_inst();
        let rbum_certs = RbumCertServ::find_rbums(
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
