use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{OpenApi, param::Path, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use crate::basic::dto::iam_cert_dto::IamUserPwdCertRestReq;
use crate::console_tenant::serv::iam_ct_cert_serv::IamCtCertServ;
use crate::iam_constants;

pub struct IamCtCertConfApi;

/// Tenant Console Cert API
#[OpenApi(prefix_path = "/ct/cert", tag = "crate::iam_enumeration::Tag::Tenant")]
impl IamCtCertConfApi {
    
    /// Rest Password
    #[oai(path = "/user-pwd/:account_id", method = "put")]
    async fn rest_password(&self, account_id: Path<String>, mut modify_req: Json<IamUserPwdCertRestReq>, cxt: 
    TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCtCertServ::rest_password(&account_id.0, &mut modify_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

}
