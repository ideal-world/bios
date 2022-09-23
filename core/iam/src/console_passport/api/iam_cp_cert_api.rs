use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::{param::Path, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};
use tardis::TardisFuns;

use bios_basic::rbum::dto::rbum_cert_dto::RbumCertSummaryResp;
use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumCertFilterReq};

use crate::basic::dto::iam_account_dto::IamAccountInfoResp;
use crate::basic::dto::iam_cert_dto::{IamContextFetchReq, IamPwdNewReq, IamUserPwdCertModifyReq};
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_cert_token_serv::IamCertTokenServ;
use crate::basic::serv::iam_key_cache_serv::IamIdentCacheServ;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::console_passport::dto::iam_cp_cert_dto::{IamCpOAuth2LoginReq, IamCpUserPwdLoginReq};
use crate::console_passport::serv::iam_cp_cert_oauth2_serv::IamCpCertOAuth2Serv;
use crate::console_passport::serv::iam_cp_cert_user_pwd_serv::IamCpCertUserPwdServ;
use crate::iam_constants;

pub struct IamCpCertApi;

/// Passport Console Cert API
#[poem_openapi::OpenApi(prefix_path = "/cp", tag = "crate::iam_enumeration::Tag::Passport")]
impl IamCpCertApi {
    /// Fetch TardisContext By Token
    ///
    /// This api is for testing only!
    ///
    /// First access `PUT /cp/login/userpwd` api to get `token` .
    /// This api input  `token` and return base64 encoded `tardis context` ,
    /// set `tardis context` to the `Tardis-Context` request header.
    #[oai(path = "/context", method = "put")]
    async fn fetch_context(&self, fetch_req: Json<IamContextFetchReq>) -> TardisApiResult<String> {
        let funs = iam_constants::get_tardis_inst();
        let ctx = IamIdentCacheServ::get_context(&fetch_req.0, &funs).await?;
        let ctx = TardisFuns::crypto.base64.encode(&TardisFuns::json.obj_to_string(&ctx)?);
        TardisResp::ok(ctx)
    }

    /// Login by Username and Password
    #[oai(path = "/login/userpwd", method = "put")]
    async fn login_by_user_pwd(&self, login_req: Json<IamCpUserPwdLoginReq>) -> TardisApiResult<IamAccountInfoResp> {
        let funs = iam_constants::get_tardis_inst();
        let resp = IamCpCertUserPwdServ::login_by_user_pwd(&login_req.0, &funs).await?;
        TardisResp::ok(resp)
    }

    /// Logout By Token
    #[oai(path = "/logout/:token", method = "delete")]
    async fn logout(&self, token: Path<String>) -> TardisApiResult<Void> {
        let funs = iam_constants::get_tardis_inst();
        IamCertTokenServ::delete_cert(&token.0, &funs).await?;
        TardisResp::ok(Void {})
    }

    /// Find Certs By Current Account
    #[oai(path = "/cert", method = "get")]
    async fn find_certs(&self, ctx: TardisContextExtractor) -> TardisApiResult<Vec<RbumCertSummaryResp>> {
        let funs = iam_constants::get_tardis_inst();
        let own_paths = if ctx.0.own_paths.is_empty() {
            None
        } else {
            Some(IamTenantServ::get_id_by_ctx(&ctx.0, &funs)?)
        };
        let rbum_certs = IamCertServ::find_certs(
            &RbumCertFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths,
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                rel_rbum_id: Some(ctx.0.owner.to_string()),
                ..Default::default()
            },
            None,
            None,
            &funs,
            &ctx.0,
        )
        .await?;
        TardisResp::ok(rbum_certs)
    }

    /// Set New Password
    #[oai(path = "/cert/userpwd/new", method = "put")]
    async fn new_pwd_without_login(&self, pwd_new_req: Json<IamPwdNewReq>) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCpCertUserPwdServ::new_pwd_without_login(&pwd_new_req.0, &funs).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Modify Password By Current Account
    #[oai(path = "/cert/userpwd", method = "put")]
    async fn modify_cert_user_pwd(&self, modify_req: Json<IamUserPwdCertModifyReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let ctx = IamCertServ::use_sys_or_tenant_ctx_unsafe(ctx.0)?;
        IamCpCertUserPwdServ::modify_cert_user_pwd(&ctx.owner, &modify_req.0, &funs, &ctx).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Get AppId by Wechat MP
    #[oai(path = "/ak/wechat-mp/:tenant_id", method = "get")]
    async fn get_ak_by_wechat_mp(&self, tenant_id: Path<String>) -> TardisApiResult<String> {
        let funs = iam_constants::get_tardis_inst();
        let resp = IamCpCertOAuth2Serv::get_ak(crate::iam_enumeration::IamCertExtKind::WechatMp, tenant_id.0, &funs).await?;
        TardisResp::ok(resp)
    }

    /// Login by Wechat MP
    #[oai(path = "/login/wechat-mp", method = "put")]
    async fn login_or_register_by_wechat_mp(&self, login_req: Json<IamCpOAuth2LoginReq>) -> TardisApiResult<IamAccountInfoResp> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let resp = IamCpCertOAuth2Serv::login_or_register(crate::iam_enumeration::IamCertExtKind::WechatMp, &login_req.0, &funs).await?;
        funs.commit().await?;
        TardisResp::ok(resp)
    }

    // /// Add Mail-VCode Cert
    // #[oai(path = "/cert/mailvcode", method = "put")]
    // async fn add_mail_vcode_cert(&self, add_req: Json<IamMailVCodeCertAddReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
    //     let mut funs = iam_constants::get_tardis_inst();
    //     funs.begin().await?;
    //     IamCpCertMailVCodeServ::add_cert_mail_vocde(&add_req.0, &funs, &ctx.0).await?;
    //     funs.commit().await?;
    //     TardisResp::ok(Void {})
    // }
    //
    // /// Delete Mail-VCode Cert
    // #[oai(path = "/cert/mailvcode/:id", method = "delete")]
    // async fn delete_mail_vcode_cert(&self, id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
    //     let mut funs = iam_constants::get_tardis_inst();
    //     funs.begin().await?;
    //     IamCertServ::delete_cert(&id.0, &funs, &ctx.0).await?;
    //     funs.commit().await?;
    //     TardisResp::ok(Void {})
    // }

    // /// Resend Activation Mail
    // #[oai(path = "/cert/mailvcode/resend", method = "put")]
    // async fn resend_activation_mail(&self, req: Json<IamMailVCodeCertResendActivationReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
    //     let funs = iam_constants::get_tardis_inst();
    //     IamCertMailVCodeServ::resend_activation_mail(&ctx.0.owner, &req.0.mail, &funs, &ctx.0).await?;
    //     TardisResp::ok(Void {})
    // }

    // /// Activate Mail
    // #[oai(path = "/cert/mailvcode/activate", method = "put")]
    // async fn activate_mail(&self, req: Json<IamMailVCodeCertActivateReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
    //     let mut funs = iam_constants::get_tardis_inst();
    //     funs.begin().await?;
    //     IamCertMailVCodeServ::activate_mail(&req.0.mail, &req.0.vcode, &funs, &ctx.0).await?;
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
