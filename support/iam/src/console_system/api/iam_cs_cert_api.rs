use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Path;
use tardis::web::poem_openapi::{param::Query, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use crate::basic::dto::iam_cert_conf_dto::{IamCertConfLdapAddOrModifyReq, IamCertConfLdapResp};
use bios_basic::rbum::dto::rbum_cert_dto::{RbumCertSummaryResp, RbumCertSummaryWithSkResp};
use bios_basic::rbum::dto::rbum_filer_dto::RbumCertFilterReq;
use bios_basic::rbum::helper::rbum_scope_helper::get_max_level_id_by_context;

use crate::basic::dto::iam_cert_dto::{IamCertExtAddReq, IamCertUserPwdRestReq};
use crate::basic::serv::iam_cert_ldap_serv::IamCertLdapServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use crate::iam_constants;
use crate::iam_enumeration::IamCertKernelKind;

pub struct IamCsCertApi;

/// System Console Cert API
#[poem_openapi::OpenApi(prefix_path = "/cs/cert", tag = "bios_basic::ApiTag::System")]
impl IamCsCertApi {
    /// Rest Password By Account Id
    #[oai(path = "/user-pwd", method = "put")]
    async fn rest_password(
        &self,
        account_id: Query<String>,
        tenant_id: Query<Option<String>>,
        modify_req: Json<IamCertUserPwdRestReq>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<Void> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_kind(IamCertKernelKind::UserPwd.to_string().as_str(), get_max_level_id_by_context(&ctx), &funs).await?;
        IamCertUserPwdServ::reset_sk(&modify_req.0, &account_id.0, &rbum_cert_conf_id, &funs, &ctx).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Find Certs By Account Id
    #[oai(path = "/", method = "get")]
    async fn find_certs(&self, account_id: Query<String>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<Vec<RbumCertSummaryResp>> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        let funs = iam_constants::get_tardis_inst();
        let rbum_certs = IamCertServ::find_certs(
            &RbumCertFilterReq {
                rel_rbum_id: Some(account_id.0.to_string()),
                ..Default::default()
            },
            None,
            None,
            &funs,
            &ctx,
        )
        .await?;
        TardisResp::ok(rbum_certs)
    }

    /// TODO 移动至 ci 并且名称修改
    /// Add Gitlab Cert
    #[oai(path = "/gitlab", method = "put")]
    async fn add_gitlab_cert(
        &self,
        account_id: Query<String>,
        tenant_id: Query<Option<String>>,
        mut add_req: Json<IamCertExtAddReq>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<Void> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCertServ::add_3th_kind_cert(&mut add_req.0, &account_id.0, "gitlab", &funs, &ctx).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Get Gitlab Certs By Account Id
    #[oai(path = "/gitlab", method = "get")]
    async fn get_gitlab_cert(&self, account_id: Query<String>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<RbumCertSummaryWithSkResp> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        let funs = iam_constants::get_tardis_inst();
        let rbum_cert = IamCertServ::get_3th_kind_cert_by_rel_rubm_id(&account_id.0, vec!["gitlab".to_string()], &funs, &ctx).await?;
        TardisResp::ok(rbum_cert)
    }

    /// Get UserPwd Certs By Account Id
    #[oai(path = "/userpwd", method = "get")]
    async fn get_user_pwd_cert(&self, account_id: Query<String>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<RbumCertSummaryWithSkResp> {
        let funs = iam_constants::get_tardis_inst();
        let resp = IamCertServ::get_kernel_cert(&account_id.0, &IamCertKernelKind::UserPwd, &funs, &ctx.0).await;
        let rbum_cert = if resp.is_ok() {
            resp.unwrap()
        } else {
            let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
            IamCertServ::get_kernel_cert(&account_id.0, &IamCertKernelKind::UserPwd, &funs, &ctx).await?
        };
        TardisResp::ok(rbum_cert)
    }
}

struct IamCsCertConfigLdapApi;
/// System Console Cert Config LDAP API
#[cfg(feature = "ldap_client")]
#[poem_openapi::OpenApi(prefix_path = "/cs/ldap", tag = "bios_basic::ApiTag::System")]
impl IamCsCertConfigLdapApi {
    /// add ldap cert
    #[oai(path = "/", method = "post")]
    async fn add_ldap_cert(&self, add_req: Json<IamCertConfLdapAddOrModifyReq>, ctx: TardisContextExtractor) -> TardisApiResult<String> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let resp = IamCertLdapServ::add_cert_conf(&add_req.0, None, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(resp)
    }
    /// modify ldap cert
    #[oai(path = "/:id", method = "put")]
    async fn modify_ldap_cert(&self, id: Path<String>, modify_req: Json<IamCertConfLdapAddOrModifyReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCertLdapServ::modify_cert_conf(&id.0, &modify_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }
    /// get ldap cert
    #[oai(path = "/", method = "get")]
    async fn get_ldap_cert(&self, ctx: TardisContextExtractor) -> TardisApiResult<Option<IamCertConfLdapResp>> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let resp = IamCertLdapServ::get_cert_conf_by_ctx(&funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(resp)
    }
}
