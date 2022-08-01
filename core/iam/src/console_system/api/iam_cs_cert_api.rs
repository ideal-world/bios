use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::{param::Query, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use bios_basic::rbum::dto::rbum_cert_dto::{RbumCertSummaryResp, RbumCertSummaryWithSkResp};
use bios_basic::rbum::dto::rbum_filer_dto::RbumCertFilterReq;
use bios_basic::rbum::helper::rbum_scope_helper::get_max_level_id_by_context;

use crate::basic::dto::iam_cert_dto::{IamExtCertAddReq, IamUserPwdCertRestReq};
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use crate::iam_constants;
use crate::iam_enumeration::{IamCertExtKind, IamCertKernelKind};

pub struct IamCsCertApi;

/// System Console Cert API
#[poem_openapi::OpenApi(prefix_path = "/cs/cert", tag = "crate::iam_enumeration::Tag::System")]
impl IamCsCertApi {
    /// Rest Password By Account Id
    #[oai(path = "/user-pwd", method = "put")]
    async fn rest_password(
        &self,
        account_id: Query<String>,
        tenant_id: Query<Option<String>>,
        modify_req: Json<IamUserPwdCertRestReq>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<Void> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_code(IamCertKernelKind::UserPwd.to_string().as_str(), get_max_level_id_by_context(&ctx), &funs).await?;
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

    /// Add Gitlab Cert
    #[oai(path = "/gitlab", method = "put")]
    async fn add_gitlab_cert(
        &self,
        account_id: Query<String>,
        tenant_id: Query<Option<String>>,
        mut add_req: Json<IamExtCertAddReq>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<Void> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCertServ::add_ext_cert(&mut add_req.0, &account_id.0, &IamCertExtKind::Gitlab, &funs, &ctx).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Get Gitlab Certs By Account Id
    #[oai(path = "/gitlab", method = "get")]
    async fn get_gitlab_cert(&self, account_id: Query<String>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<RbumCertSummaryWithSkResp> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        let funs = iam_constants::get_tardis_inst();
        let rbum_cert = IamCertServ::get_ext_cert(&account_id.0, &IamCertExtKind::Gitlab, &funs, &ctx).await?;
        TardisResp::ok(rbum_cert)
    }
}
