use crate::basic::dto::iam_cert_dto::{IamCertAkSkAddReq, IamCertAkSkResp, IamOauth2AkSkResp};
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::console_interface::serv::iam_ci_cert_aksk_serv::IamCiCertAkSkServ;
use crate::console_interface::serv::iam_ci_oauth2_token_serv::IamCiOauth2AkSkServ;
use crate::iam_constants;
use crate::iam_enumeration::Oauth2GrantType;
use bios_basic::rbum::dto::rbum_cert_dto::RbumCertSummaryWithSkResp;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

pub struct IamCiCertManageApi;
pub struct IamCiCertApi;

/// # Interface Console Manage Cert API
/// Allow management of aksk (an authentication method between applications)
#[poem_openapi::OpenApi(prefix_path = "/ci/manage", tag = "bios_basic::ApiTag::Interface")]
impl IamCiCertManageApi {
    /// add aksk cert
    #[oai(path = "/aksk", method = "put")]
    async fn add_aksk(&self, add_req: Json<IamCertAkSkAddReq>, ctx: TardisContextExtractor) -> TardisApiResult<IamCertAkSkResp> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamCiCertAkSkServ::general_cert(add_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }

    #[oai(path = "/aksk", method = "delete")]
    async fn delete_aksk(&self, id: Query<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = iam_constants::get_tardis_inst();
        IamCiCertAkSkServ::delete_cert(&id.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    #[oai(path = "/token", method = "get")]
    async fn get_token(
        &self,
        grant_type: Query<String>,
        client_id: Query<String>,
        client_secret: Query<String>,
        scope: Query<Option<String>>,
    ) -> TardisApiResult<IamOauth2AkSkResp> {
        let grant_type = Oauth2GrantType::parse(&grant_type.0)?;
        let funs = iam_constants::get_tardis_inst();
        let resp = IamCiOauth2AkSkServ::generate_token(grant_type, &client_id.0, &client_secret.0, scope.0, funs).await?;
        TardisResp::ok(resp)
    }
}

#[poem_openapi::OpenApi(prefix_path = "/ci/cert", tag = "bios_basic::ApiTag::Interface")]
impl IamCiCertApi {
    /// find cert by kind and supplier
    ///
    /// if kind is none,query default kind(UserPwd)
    #[oai(path = "/:account_id", method = "get")]
    async fn get_cert_by_kind_supplier(
        &self,
        account_id: Path<String>,
        kind: Query<Option<String>>,
        tenant_id: Query<Option<String>>,
        supplier: Query<Option<String>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<RbumCertSummaryWithSkResp> {
        let funs = iam_constants::get_tardis_inst();
        let supplier = supplier.0.unwrap_or_default();
        let kind = kind.0.unwrap_or_else(|| "UserPwd".to_string());
        let kind = if kind.is_empty() { "UserPwd".to_string() } else { kind };

        let true_tenant_id = if IamAccountServ::is_global_account(&account_id.0, &funs, &ctx.0).await? {
            None
        } else {
            tenant_id.0
        };
        let conf_id = if let Ok(conf_id) = IamCertServ::get_cert_conf_id_by_kind_supplier(&kind, &supplier.clone(), true_tenant_id.clone(), &funs).await {
            Some(conf_id)
        } else {
            None
        };

        let cert = IamCertServ::get_cert_by_relrubmid_kind_supplier(&account_id.0, &kind, vec![supplier], conf_id, &true_tenant_id.unwrap_or_default(), &funs, &ctx.0).await?;
        TardisResp::ok(cert)
    }

    ///Auto sync
    ///
    /// 定时任务触发第三方集成同步
    #[oai(path = "/sync", method = "get")]
    async fn third_integration_sync(&self, ctx: TardisContextExtractor) -> TardisApiResult<String> {
        let funs = iam_constants::get_tardis_inst();
        let msg = IamCertServ::third_integration_sync_without_config(&funs, &ctx.0).await?;

        TardisResp::ok(msg)
    }
}
