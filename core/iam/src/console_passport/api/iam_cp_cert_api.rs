use tardis::basic::dto::TardisContext;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{payload::Json, OpenApi};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use bios_basic::rbum::dto::rbum_cert_dto::RbumCertSummaryResp;
use bios_basic::rbum::dto::rbum_filer_dto::RbumCertFilterReq;

use crate::basic::dto::iam_account_dto::AccountInfoResp;
use crate::basic::dto::iam_cert_dto::{IamContextFetchReq, IamUserPwdCertModifyReq};
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_key_cache_serv::IamIdentCacheServ;
use crate::console_passport::dto::iam_cp_cert_dto::IamCpUserPwdLoginReq;
use crate::console_passport::serv::iam_cp_cert_user_pwd_serv::IamCpCertUserPwdServ;
use crate::iam_constants;

pub struct IamCpCertApi;

/// Passport Console Cert API
#[OpenApi(prefix_path = "/cp", tag = "crate::iam_enumeration::Tag::Passport")]
impl IamCpCertApi {
    /// Fetch TardisContext By Token
    #[oai(path = "/context", method = "put")]
    async fn fetch_context(&self, fetch_req: Json<IamContextFetchReq>) -> TardisApiResult<TardisContext> {
        let funs = iam_constants::get_tardis_inst();
        let cxt = IamIdentCacheServ::get_context(&fetch_req.0, &funs).await?;
        TardisResp::ok(cxt)
    }

    /// Login by Username and Password
    #[oai(path = "/login/userpwd", method = "put")]
    async fn login_by_user_pwd(&self, login_req: Json<IamCpUserPwdLoginReq>) -> TardisApiResult<AccountInfoResp> {
        let funs = iam_constants::get_tardis_inst();
        let resp = IamCpCertUserPwdServ::login_by_user_pwd(&login_req.0, &funs).await?;
        TardisResp::ok(resp)
    }

    /// Find Certs By Current Account
    #[oai(path = "/cert", method = "get")]
    async fn find_certs(&self, cxt: TardisContextExtractor) -> TardisApiResult<Vec<RbumCertSummaryResp>> {
        let funs = iam_constants::get_tardis_inst();
        let rbum_certs = IamCertServ::find_certs(
            &RbumCertFilterReq {
                rel_rbum_id: Some(cxt.0.owner.to_string()),
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

    /// Modify Password By Current Account
    #[oai(path = "/cert/userpwd", method = "put")]
    async fn modify_cert_user_pwd(&self, modify_req: Json<IamUserPwdCertModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCpCertUserPwdServ::modify_cert_user_pwd(&cxt.0.owner, &modify_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    // /// Add Mail-VCode Cert
    // #[oai(path = "/cert/mailvcode", method = "put")]
    // async fn add_mail_vcode_cert(&self, add_req: Json<IamMailVCodeCertAddReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
    //     let mut funs = iam_constants::get_tardis_inst();
    //     funs.begin().await?;
    //     IamCpCertMailVCodeServ::add_cert_mail_vocde(&add_req.0, &funs, &cxt.0).await?;
    //     funs.commit().await?;
    //     TardisResp::ok(Void {})
    // }
    //
    // /// Delete Mail-VCode Cert
    // #[oai(path = "/cert/mailvcode/:id", method = "delete")]
    // async fn delete_mail_vcode_cert(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
    //     let mut funs = iam_constants::get_tardis_inst();
    //     funs.begin().await?;
    //     IamCertServ::delete_cert(&id.0, &funs, &cxt.0).await?;
    //     funs.commit().await?;
    //     TardisResp::ok(Void {})
    // }

    // /// Resend Activation Mail
    // #[oai(path = "/cert/mailvcode/resend", method = "put")]
    // async fn resend_activation_mail(&self, req: Json<IamMailVCodeCertResendActivationReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
    //     let funs = iam_constants::get_tardis_inst();
    //     IamCertMailVCodeServ::resend_activation_mail(&cxt.0.owner, &req.0.mail, &funs, &cxt.0).await?;
    //     TardisResp::ok(Void {})
    // }

    // /// Activate Mail
    // #[oai(path = "/cert/mailvcode/activate", method = "put")]
    // async fn activate_mail(&self, req: Json<IamMailVCodeCertActivateReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
    //     let mut funs = iam_constants::get_tardis_inst();
    //     funs.begin().await?;
    //     IamCertMailVCodeServ::activate_mail(&req.0.mail, &req.0.vcode, &funs, &cxt.0).await?;
    //     funs.commit().await?;
    //     TardisResp::ok(Void {})
    // }
    //
    // /// Send Login Mail
    // #[oai(path = "/login/mailvcode/vcode", method = "post")]
    // async fn send_login_mail(&self, login_req: Json<IamCpMailVCodeLoginGenVCodeReq>) -> TardisApiResult<Void> {
    //     let mut funs = iam_constants::get_tardis_inst();
    //     funs.begin().await?;
    //     IamCertMailVCodeServ::send_login_mail(&login_req.0.mail, &login_req.0.tenant_id, &funs).await?;
    //     funs.commit().await?;
    //     TardisResp::ok(Void {})
    // }
    //
    // /// Login by Mail And Vcode
    // #[oai(path = "/login/mailvcode", method = "put")]
    // async fn login_by_mail_vocde(&self, login_req: Json<IamCpMailVCodeLoginReq>) -> TardisApiResult<AccountInfoResp> {
    //     let mut funs = iam_constants::get_tardis_inst();
    //     funs.begin().await?;
    //     let resp = IamCpCertMailVCodeServ::login_by_mail_vocde(&login_req.0, &funs).await?;
    //     funs.commit().await?;
    //     TardisResp::ok(resp)
    // }
}
