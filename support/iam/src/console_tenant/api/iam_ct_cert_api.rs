use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::{param::Query, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use bios_basic::rbum::dto::rbum_cert_dto::{RbumCertSummaryResp, RbumCertSummaryWithSkResp};
use bios_basic::rbum::dto::rbum_filer_dto::RbumCertFilterReq;
use bios_basic::rbum::helper::rbum_scope_helper::get_max_level_id_by_context;

use crate::basic::dto::iam_cert_dto::IamCertUserPwdRestReq;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use crate::iam_constants;
use crate::iam_enumeration::IamCertKernelKind;
#[derive(Clone, Default)]
pub struct IamCtCertApi;

/// Tenant Console Cert API
#[poem_openapi::OpenApi(prefix_path = "/ct/cert", tag = "bios_basic::ApiTag::Tenant")]
impl IamCtCertApi {
    /// Rest Password By Account Id
    #[oai(path = "/user-pwd", method = "put")]
    async fn rest_password(
        &self,
        account_id: Query<String>,
        app_id: Query<Option<String>>,
        modify_req: Json<IamCertUserPwdRestReq>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<Void> {
        let ctx = IamCertServ::try_use_app_ctx(ctx.0, app_id.0)?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_kind(IamCertKernelKind::UserPwd.to_string().as_str(), get_max_level_id_by_context(&ctx), &funs).await?;
        IamCertUserPwdServ::reset_sk(&modify_req.0, &account_id.0, &rbum_cert_conf_id, &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Find Certs By Account Id
    #[oai(path = "/", method = "get")]
    async fn find_certs(&self, account_id: Query<String>, app_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<Vec<RbumCertSummaryResp>> {
        let ctx = IamCertServ::try_use_app_ctx(ctx.0, app_id.0)?;
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
        ctx.execute_task().await?;
        TardisResp::ok(rbum_certs)
    }

    /// Find third-kind Certs By Account Id
    #[oai(path = "/third-kind", method = "get")]
    async fn get_third_cert(
        &self,
        account_id: Query<String>,
        cert_supplier: Query<String>,
        tenant_id: Query<Option<String>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<RbumCertSummaryWithSkResp> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        let funs = iam_constants::get_tardis_inst();
        let rbum_cert = IamCertServ::get_3th_kind_cert_by_rel_rubm_id(&account_id.0, vec![cert_supplier.0], &funs, &ctx).await?;
        ctx.execute_task().await?;
        TardisResp::ok(rbum_cert)
    }
}
