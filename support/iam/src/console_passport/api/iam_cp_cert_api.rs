use tardis::basic::dto::TardisContext;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::poem_openapi::{param::Path, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};
use tardis::TardisFuns;

use bios_basic::rbum::dto::rbum_cert_dto::{RbumCertSummaryResp, RbumCertSummaryWithSkResp};
use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumCertFilterReq};
use bios_basic::rbum::helper::rbum_scope_helper::get_max_level_id_by_context;

use crate::basic::dto::iam_account_dto::IamAccountInfoResp;
use crate::basic::dto::iam_cert_dto::{
    IamCertGenericValidateSkReq, IamCertMailVCodeActivateReq, IamCertMailVCodeAddReq, IamCertPhoneVCodeAddReq, IamCertPhoneVCodeBindReq, IamCertPwdNewReq, IamCertUserNameNewReq,
    IamCertUserPwdModifyReq, IamCertUserPwdRestReq, IamContextFetchReq,
};
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_cert_mail_vcode_serv::IamCertMailVCodeServ;
use crate::basic::serv::iam_cert_phone_vcode_serv::IamCertPhoneVCodeServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_cert_token_serv::IamCertTokenServ;
use crate::basic::serv::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use crate::basic::serv::iam_key_cache_serv::IamIdentCacheServ;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::console_passport::dto::iam_cp_cert_dto::{
    IamCpExistMailVCodeReq, IamCpExistPhoneVCodeReq, IamCpMailVCodeLoginGenVCodeReq, IamCpMailVCodeLoginReq, IamCpOAuth2LoginReq, IamCpPhoneVCodeLoginGenVCodeReq,
    IamCpPhoneVCodeLoginSendVCodeReq, IamCpUserPwdLoginReq,
};
use crate::console_passport::serv::iam_cp_cert_mail_vcode_serv::IamCpCertMailVCodeServ;
use crate::console_passport::serv::iam_cp_cert_oauth2_serv::IamCpCertOAuth2Serv;
use crate::console_passport::serv::iam_cp_cert_phone_vcode_serv::IamCpCertPhoneVCodeServ;
use crate::console_passport::serv::iam_cp_cert_user_pwd_serv::IamCpCertUserPwdServ;
use crate::iam_constants;
use crate::iam_enumeration::{IamCertKernelKind, IamCertOAuth2Supplier};
use bios_basic::helper::request_helper::add_remote_ip;
use tardis::web::poem::Request;
#[derive(Clone, Default)]
pub struct IamCpCertApi;

/// Passport Console Cert API
#[poem_openapi::OpenApi(prefix_path = "/cp", tag = "bios_basic::ApiTag::Passport")]
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

    #[oai(path = "/login/pwd/status", method = "get")]
    async fn login_status(&self, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        add_remote_ip(&request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let status = IamCertServ::get_kernel_cert(&ctx.0.owner, &IamCertKernelKind::UserPwd, &funs, &ctx.0).await?.status;
        ctx.0.execute_task().await?;
        TardisResp::ok(status.to_string())
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
    async fn find_certs(&self, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Vec<RbumCertSummaryResp>> {
        add_remote_ip(&request, &ctx.0).await?;
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
        ctx.0.execute_task().await?;
        TardisResp::ok(rbum_certs)
    }

    /// Find Third-kind Certs By Current Account
    #[oai(path = "/cert/third-kind", method = "get")]
    async fn get_third_cert(&self, supplier: Query<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<RbumCertSummaryWithSkResp> {
        add_remote_ip(&request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        // let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        let rbum_cert = IamCertServ::get_3th_kind_cert_by_rel_rubm_id(&ctx.0.owner, vec![supplier.0], &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(rbum_cert)
    }

    /// Set New Username(cert_conf kind usrpwd)
    #[oai(path = "/cert/username/new", method = "put")]
    async fn new_user_name(&self, pwd_new_req: Json<IamCertUserNameNewReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        add_remote_ip(&request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCpCertUserPwdServ::new_user_name(&pwd_new_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Set New Password
    #[oai(path = "/cert/userpwd/new", method = "put")]
    async fn new_pwd_without_login(&self, pwd_new_req: Json<IamCertPwdNewReq>) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCpCertUserPwdServ::new_pwd_without_login(&pwd_new_req.0, &funs).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// new userpwd-cert password by account_id
    ///
    /// only for user is global account
    #[oai(path = "/cert/userpwd/reset", method = "put")]
    async fn new_password_for_pending_status(&self, modify_req: Json<IamCertUserPwdRestReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        add_remote_ip(&request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        let account_id = &ctx.0.owner.clone();
        let ctx = IamCertServ::use_global_account_ctx(ctx.0, account_id, &funs).await?;
        funs.begin().await?;
        let rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_kind(IamCertKernelKind::UserPwd.to_string().as_str(), get_max_level_id_by_context(&ctx), &funs).await?;
        IamCertUserPwdServ::reset_sk_to_enable_status(&modify_req.0, &ctx.owner, &rbum_cert_conf_id, &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Modify Password By Current Account
    #[oai(path = "/cert/userpwd", method = "put")]
    async fn modify_cert_user_pwd(&self, modify_req: Json<IamCertUserPwdModifyReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        add_remote_ip(&request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let ctx = IamCertServ::use_sys_or_tenant_ctx_unsafe(ctx.0)?;
        IamCpCertUserPwdServ::modify_cert_user_pwd(&ctx.owner, &modify_req.0, &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Login by general oauth2
    #[oai(path = "/login/oauth2/:supplier", method = "put")]
    async fn login_or_register_by_oauth2(&self, supplier: Path<String>, login_req: Json<IamCpOAuth2LoginReq>) -> TardisApiResult<IamAccountInfoResp> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let resp = IamCpCertOAuth2Serv::login_or_register(IamCertOAuth2Supplier::parse(&supplier.0)?, &login_req.0, &funs).await?;
        funs.commit().await?;
        TardisResp::ok(resp)
    }

    /// Validate userpwd By Current Account and ignore expired
    #[oai(path = "/validate/userpwd/ignore/expired", method = "put")]
    async fn validate_by_user_pwd_ignore_expired(&self, req: Json<IamCertGenericValidateSkReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        add_remote_ip(&request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        IamCpCertUserPwdServ::validate_by_user_pwd(&req.0.sk, true, &funs, &IamAccountServ::new_context_if_account_is_global(&ctx.0, &funs).await?).await?;
        TardisResp::ok(Void {})
    }

    // /// Add Mail-VCode Cert
    // /// Send Activation Mail
    // #[oai(path = "/cert/mailvcode/send", method = "put")]
    // async fn resend_activation_mail(&self, req: Json<IamCertMailVCodeResendActivationReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
    //     let funs = iam_constants::get_tardis_inst();
    //     IamCertMailVCodeServ::resend_activation_mail(&ctx.0.owner, &req.0.mail, &funs, &ctx.0).await?;
    //     TardisResp::ok(Void {})
    // }
    //
    // /// Activate Mail
    // #[oai(path = "/cert/mailvcode/activate", method = "put")]
    // async fn activate_mail(&self, req: Json<IamCertMailVCodeActivateReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
    //     let mut funs = iam_constants::get_tardis_inst();
    //     funs.begin().await?;
    //     IamCertMailVCodeServ::activate_mail(&req.0.mail, &req.0.vcode, &funs, &ctx.0).await?;
    //     funs.commit().await?;
    //     TardisResp::ok(Void {})
    // }

    /// exist Mail
    #[oai(path = "/exist/mailvcode", method = "put")]
    async fn exist_mail(&self, exist_req: Json<IamCpExistMailVCodeReq>) -> TardisApiResult<bool> {
        let funs = iam_constants::get_tardis_inst();
        let own_paths = if let Some(tenant_id) = exist_req.0.tenant_id { tenant_id } else { "".to_string() };
        let mock_ctx = TardisContext { own_paths, ..Default::default() };
        let count = IamCertServ::count_cert_ak_by_kind(&IamCertKernelKind::MailVCode.to_string(), &exist_req.0.mail, &funs, &mock_ctx).await?;
        TardisResp::ok(count > 0)
    }

    /// Send bind Mail
    #[oai(path = "/cert/mailvcode/send", method = "put")]
    async fn send_bind_mail(&self, req: Json<IamCertMailVCodeAddReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        add_remote_ip(&request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCertMailVCodeServ::send_bind_mail(&req.0.mail, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Bind Mail
    #[oai(path = "/cert/mailvcode/bind", method = "put")]
    async fn bind_mail(&self, req: Json<IamCertMailVCodeActivateReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        add_remote_ip(&request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCertMailVCodeServ::bind_mail(&req.0.mail, &req.0.vcode, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Send Login Mail
    #[oai(path = "/login/mailvcode/vcode", method = "post")]
    async fn send_login_mail(&self, login_req: Json<IamCpMailVCodeLoginGenVCodeReq>) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCertMailVCodeServ::send_login_mail(&login_req.0.mail, &login_req.0.tenant_id.unwrap_or("".to_string()), &funs).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Login by Mail And Vcode
    #[oai(path = "/login/mailvcode", method = "put")]
    async fn login_by_mail_vocde(&self, login_req: Json<IamCpMailVCodeLoginReq>) -> TardisApiResult<IamAccountInfoResp> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let resp = IamCpCertMailVCodeServ::login_by_mail_vocde(&login_req.0, &funs).await?;
        funs.commit().await?;
        TardisResp::ok(resp)
    }

    /// exist phone
    #[oai(path = "/exist/phonevcode", method = "put")]
    async fn exist_phone(&self, exist_req: Json<IamCpExistPhoneVCodeReq>) -> TardisApiResult<bool> {
        let funs = iam_constants::get_tardis_inst();
        let own_paths = if let Some(tenant_id) = exist_req.0.tenant_id { tenant_id } else { "".to_string() };
        let mock_ctx = TardisContext { own_paths, ..Default::default() };
        let count = IamCertServ::count_cert_ak_by_kind(&IamCertKernelKind::PhoneVCode.to_string(), &exist_req.0.phone, &funs, &mock_ctx).await?;
        TardisResp::ok(count > 0)
    }

    /// Send bind phone
    #[oai(path = "/cert/phonevcode/send", method = "put")]
    async fn send_bind_phone(&self, req: Json<IamCertPhoneVCodeAddReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        add_remote_ip(&request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCertPhoneVCodeServ::send_bind_phone(&req.0.phone, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Bind phone
    #[oai(path = "/cert/phonevcode/bind", method = "put")]
    async fn bind_phone(&self, req: Json<IamCertPhoneVCodeBindReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        add_remote_ip(&request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCertPhoneVCodeServ::bind_phone(&req.0.phone.to_string(), &req.0.vcode, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Send Login Phone
    #[oai(path = "/login/phonecode/vcode", method = "post")]
    async fn send_login_phone(&self, login_req: Json<IamCpPhoneVCodeLoginGenVCodeReq>) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCertPhoneVCodeServ::send_login_phone(&login_req.0.phone, &login_req.0.tenant_id.unwrap_or("".to_string()), &funs).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Login by Phone And Vcode
    #[oai(path = "/login/phonevcode", method = "put")]
    async fn login_by_phone_vocde(&self, login_req: Json<IamCpPhoneVCodeLoginSendVCodeReq>) -> TardisApiResult<IamAccountInfoResp> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let resp = IamCpCertPhoneVCodeServ::login_by_phone_vocde(&login_req.0, &funs).await?;
        funs.commit().await?;
        TardisResp::ok(resp)
    }
}
