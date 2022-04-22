use tardis::basic::dto::TardisContext;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{OpenApi, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use crate::basic::dto::iam_account_dto::AccountInfoResp;
use crate::basic::dto::iam_cert_dto::{IamContextFetchReq, IamUserPwdCertModifyReq};
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::console_passport::dto::iam_cp_cert_dto::IamCpUserPwdLoginReq;
use crate::console_passport::serv::iam_cp_cert_user_pwd_serv::IamCpCertUserPwdServ;
use crate::iam_constants;

pub struct IamCpAccountApi;

/// Personal Console Cert API
#[OpenApi(prefix_path = "/cp/cert", tag = "crate::iam_enumeration::Tag::Passport")]
impl IamCpAccountApi {
    /// Login by Username and Password
    #[oai(path = "/login", method = "put")]
    async fn login(&self, login_req: Json<IamCpUserPwdLoginReq>) -> TardisApiResult<AccountInfoResp> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let resp = IamCpCertUserPwdServ::login_by_user_pwd(&login_req.0, &funs).await?;
        funs.commit().await?;
        TardisResp::ok(resp)
    }

    /// Fetch TardisContext By Token
    #[oai(path = "/cp/context", method = "get")]
    async fn fetch_context(&self, fetch_req: Json<IamContextFetchReq>) -> TardisApiResult<TardisContext> {
        let funs = iam_constants::get_tardis_inst();
        let cxt = IamCertServ::fetch_context(&fetch_req.0, &funs).await?;
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
