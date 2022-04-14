use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{payload::Json, OpenApi};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use crate::basic::dto::iam_cert_dto::IamUserPwdCertModifyReq;
use crate::console_passport::dto::iam_cp_cert_dto::{IamCpUserPwdLoginReq, LoginResp};
use crate::console_passport::serv::iam_cp_cert_user_pwd_serv::IamCpCertUserPwdServ;
use crate::iam_constants;

pub struct IamCpAccountApi;

/// Personal Console Cert API
#[OpenApi(prefix_path = "/cp/cert", tag = "crate::iam_enumeration::Tag::Passport")]
impl IamCpAccountApi {
    /// Login by Username and Password
    #[oai(path = "/login", method = "put")]
    async fn login(&self, mut login_req: Json<IamCpUserPwdLoginReq>) -> TardisApiResult<LoginResp> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let cxt = IamCpCertUserPwdServ::login_by_user_pwd(&mut login_req.0, &funs).await?;
        funs.commit().await?;
        TardisResp::ok(cxt)
    }

    /// Modify Password
    #[oai(path = "/", method = "put")]
    async fn modify_password(&self, mut modify_req: Json<IamUserPwdCertModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCpCertUserPwdServ::modify_cert_user_pwd(&mut modify_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }
}
