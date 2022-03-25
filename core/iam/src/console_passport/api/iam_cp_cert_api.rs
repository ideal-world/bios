use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{payload::Json, OpenApi};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};
use tardis::TardisFuns;

use crate::basic::dto::iam_cert_dto::IamUserPwdCertModifyReq;
use crate::basic::serv::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;

pub struct IamCpAccountApi;

/// Personal Console Cert API
#[OpenApi(prefix_path = "/cp/cert", tag = "bios_basic::Components::Iam")]
impl IamCpAccountApi {
    /// Modify Password
    #[oai(path = "/", method = "put")]
    async fn modify_password(&self, mut modify_req: Json<IamUserPwdCertModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut tx = TardisFuns::reldb().conn();
        tx.begin().await?;
        IamCertUserPwdServ::modify_cert(&mut modify_req.0, &cxt.0.account_id, &IamTenantServ::get_id_by_cxt(&cxt.0)?, &tx, &cxt.0).await?;
        tx.commit().await?;
        TardisResp::ok(Void {})
    }
}
