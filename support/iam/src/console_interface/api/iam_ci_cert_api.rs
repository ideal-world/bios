use std::collections::HashMap;

use crate::basic::dto::iam_account_dto::IamAccountExtSysResp;
use crate::basic::dto::iam_cert_dto::{IamCertAkSkAddReq, IamCertAkSkResp, IamOauth2AkSkResp, IamThirdPartyCertExtAddReq, IamCertDecodeRequest};
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_cert_ldap_serv::IamCertLdapServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::console_interface::serv::iam_ci_cert_aksk_serv::IamCiCertAkSkServ;
use crate::console_interface::serv::iam_ci_oauth2_token_serv::IamCiOauth2AkSkServ;

use crate::iam_constants;
use crate::iam_enumeration::Oauth2GrantType;
use bios_basic::helper::request_helper::add_remote_ip;
use bios_basic::rbum::dto::rbum_cert_dto::RbumCertSummaryWithSkResp;
use tardis::basic::dto::TardisContext;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};
#[derive(Clone, Default)]
pub struct IamCiCertManageApi;
#[derive(Clone, Default)]
pub struct IamCiCertApi;
#[derive(Clone, Default)]
pub struct IamCiLdapCertApi;

/// # Interface Console Manage Cert API
///
/// Allow Management Of aksk (an authentication method between applications)
#[poem_openapi::OpenApi(prefix_path = "/ci/manage", tag = "bios_basic::ApiTag::Interface")]
impl IamCiCertManageApi {
    /// Add aksk Cert
    #[oai(path = "/aksk", method = "put")]
    async fn add_aksk(&self, add_req: Json<IamCertAkSkAddReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamCertAkSkResp> {
        add_remote_ip(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamCiCertAkSkServ::general_cert(add_req.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    #[oai(path = "/aksk", method = "delete")]
    async fn delete_aksk(&self, id: Query<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        add_remote_ip(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        IamCiCertAkSkServ::delete_cert(&id.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
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
    /// Find Cert By Kind And Supplier
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
        request: &Request,
    ) -> TardisApiResult<RbumCertSummaryWithSkResp> {
        add_remote_ip(request, &ctx.0).await?;
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
        ctx.0.execute_task().await?;
        TardisResp::ok(cert)
    }

    /// Add Third-kind Cert
    #[oai(path = "/third-kind", method = "put")]
    async fn add_third_cert(
        &self,
        account_id: Query<String>,
        mut add_req: Json<IamThirdPartyCertExtAddReq>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Void> {
        add_remote_ip(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCertServ::add_3th_kind_cert(&mut add_req.0, &account_id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Get Third-kind Certs By Account Id
    #[oai(path = "/third-kind", method = "get")]
    async fn get_third_cert(
        &self,
        account_id: Query<String>,
        supplier: Query<String>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<RbumCertSummaryWithSkResp> {
        add_remote_ip(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let rbum_cert = IamCertServ::get_3th_kind_cert_by_rel_rubm_id(&account_id.0, vec![supplier.0], &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(rbum_cert)
    }

    ///Auto Sync
    ///
    /// 定时任务触发第三方集成同步
    #[oai(path = "/sync", method = "get")]
    async fn third_integration_sync(&self, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        add_remote_ip(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let msg = IamCertServ::third_integration_sync_without_config(&funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(msg)
    }

    /// decode cert
    #[oai(path = "/decode", method = "post")]
    async fn decode_certs(&self, body: Json<IamCertDecodeRequest>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<HashMap<String, String>> {
        let funs = iam_constants::get_tardis_inst();
        let ctx = IamCertServ::use_sys_or_tenant_ctx_unsafe(ctx.0)?;
        add_remote_ip(request, &ctx).await?;
        let doceded = IamCertServ::batch_decode_cert(body.0.codes, &funs, &ctx).await?;
        TardisResp::ok(doceded)
    }
}

#[poem_openapi::OpenApi(prefix_path = "/ci/ldap/cert", tag = "bios_basic::ApiTag::Interface")]
impl IamCiLdapCertApi {
    /// 根据ldap cn查询对应的displayName
    #[oai(path = "/cn/:cn", method = "get")]
    async fn get_ldap_resp_by_cn(&self, cn: Path<String>) -> TardisApiResult<Vec<IamAccountExtSysResp>> {
        let funs = iam_constants::get_tardis_inst();
        let ctx = TardisContext {
            own_paths: "".to_string(),
            ak: "".to_string(),
            roles: vec![],
            groups: vec![],
            owner: "".to_string(),
            ..Default::default()
        };
        let result = IamCertLdapServ::get_ldap_resp_by_cn(&cn.0, &funs, &ctx).await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }
}
