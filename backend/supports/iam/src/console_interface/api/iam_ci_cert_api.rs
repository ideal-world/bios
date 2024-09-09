use std::collections::HashMap;

use crate::basic::dto::iam_account_dto::IamAccountExtSysResp;
use crate::basic::dto::iam_cert_conf_dto::IamCertConfLdapResp;
use crate::basic::dto::iam_cert_dto::{IamCertAkSkAddReq, IamCertAkSkResp, IamCertDecodeRequest, IamOauth2AkSkResp, IamThirdPartyCertExtAddReq, IamThirdPartyCertExtModifyReq};
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_cert_ldap_serv::IamCertLdapServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::console_interface::serv::iam_ci_cert_aksk_serv::IamCiCertAkSkServ;
use crate::console_interface::serv::iam_ci_oauth2_token_serv::IamCiOauth2AkSkServ;
use crate::iam_constants;
use crate::iam_enumeration::Oauth2GrantType;
use bios_basic::helper::request_helper::try_set_real_ip_from_req_to_ctx;
use bios_basic::rbum::dto::rbum_cert_dto::RbumCertSummaryWithSkResp;
use bios_basic::rbum::dto::rbum_filer_dto::RbumCertFilterReq;
use bios_basic::rbum::helper::rbum_event_helper;
use bios_basic::rbum::helper::rbum_scope_helper::check_without_owner_and_unsafe_fill_ctx;
use bios_basic::rbum::serv::rbum_cert_serv::RbumCertServ;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::log::info;
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
/// 接口控制台管理证书API
///
/// Allow Management Of aksk (an authentication method between applications)
/// 允许管理aksk（应用之间的一种认证方式）
#[poem_openapi::OpenApi(prefix_path = "/private/ci/manage", tag = "bios_basic::ApiTag::Interface")]
impl IamCiCertManageApi {
    /// Add aksk Cert
    /// 添加aksk证书
    #[oai(path = "/aksk", method = "post")]
    async fn add_aksk(&self, add_req: Json<IamCertAkSkAddReq>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamCertAkSkResp> {
        let mut funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, Some(add_req.tenant_id.clone()))?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        funs.begin().await?;
        let result = IamCiCertAkSkServ::general_cert(add_req.0, &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Delete aksk Cert
    /// 删除aksk证书
    #[oai(path = "/aksk", method = "delete")]
    async fn delete_aksk(&self, id: Query<String>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        funs.begin().await?;
        IamCiCertAkSkServ::delete_cert(&id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Get token
    /// 获取token
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

/// # Interface Console Cert API
/// 接口控制台证书API
#[poem_openapi::OpenApi(prefix_path = "/ci/cert", tag = "bios_basic::ApiTag::Interface")]
impl IamCiCertApi {
    /// Get Cert By Id
    /// 根据id获取证书
    #[oai(path = "/get/:id", method = "get")]
    async fn get_cert_by_id(&self, id: Path<String>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamCertAkSkResp> {
        let funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let ak = RbumCertServ::find_one_detail_rbum(
            &RbumCertFilterReq {
                id: Some(id.0.clone()),
                ..Default::default()
            },
            &funs,
            &ctx.0,
        )
        .await?
        .ok_or_else(|| funs.err().internal_error("iam_ci_cert", "get_cert_by_id", "cert is not found", "401-iam-cert-code-not-exist"))?
        .ak;
        let sk = RbumCertServ::show_sk(&id.0, &RbumCertFilterReq::default(), &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(IamCertAkSkResp { id: id.clone(), ak, sk })
    }
    /// Find Cert By Kind And Supplier
    ///
    /// if kind is none,query default kind(UserPwd)
    /// - `supplier` is only used when kind is `Ldap`
    /// - `ldap_origin` is only used when kind is `Ldap` and default is false.
    /// when true,return ak will be original DN
    ///
    /// 根据kind和supplier查询证书
    ///
    /// 如果kind为空，查询默认kind(UserPwd)
    /// - `supplier`仅在kind为`Ldap`时使用
    /// - `ldap_origin`仅在kind为`Ldap`时使用，默认为false
    /// 当为true时，返回的ak将是原始DN
    #[oai(path = "/:account_id", method = "get")]
    async fn get_cert_by_kind_supplier(
        &self,
        account_id: Path<String>,
        kind: Query<Option<String>>,
        tenant_id: Query<Option<String>>,
        supplier: Query<Option<String>>,
        ldap_origin: Query<Option<bool>>,
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<RbumCertSummaryWithSkResp> {
        let funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        info!("tenant_id: {:?}, ctx.own_paths: {}", tenant_id.0, ctx.0.own_paths);
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0.clone())?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let supplier = supplier.0.unwrap_or_default();
        let kind = kind.0.unwrap_or_else(|| "UserPwd".to_string());
        let kind = if kind.is_empty() { "UserPwd".to_string() } else { kind };

        let true_tenant_id = if IamAccountServ::is_global_account(&account_id.0, &funs, &ctx).await? {
            None
        } else {
            tenant_id.0
        };
        let conf_id = if let Ok(conf_id) = IamCertServ::get_cert_conf_id_by_kind_supplier(&kind, &supplier.clone(), true_tenant_id.clone(), &funs).await {
            Some(conf_id)
        } else {
            None
        };
        let ldap_dn = ldap_origin.0.unwrap_or_default();
        let cert =
            IamCertServ::get_cert_by_rel_rbum_id_kind_supplier(&account_id.0, &kind, vec![supplier], conf_id, &true_tenant_id.unwrap_or_default(), ldap_dn, &funs, &ctx).await?;
        ctx.execute_task().await?;
        TardisResp::ok(cert)
    }

    /// Add Third-kind Cert
    ///
    /// 添加第三方证书
    #[oai(path = "/third-kind", method = "post")]
    async fn add_third_cert(&self, mut add_req: Json<IamThirdPartyCertExtAddReq>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        funs.begin().await?;
        IamCertServ::add_3th_kind_cert(&mut add_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// modify Third-kind Cert
    ///
    /// 修改第三方证书
    #[oai(path = "/third-kind", method = "put")]
    async fn modify_third_cert(&self, mut modify_req: Json<IamThirdPartyCertExtModifyReq>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        funs.begin().await?;
        IamCertServ::modify_3th_kind_cert(&mut modify_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// find Third-kind Certs By Account Id
    ///
    /// 根据账号id获取第三方证书,如果account_id为空，则查询所有
    #[oai(path = "/find/third-kind", method = "get")]
    async fn find_third_cert(
        &self,
        rel_rbum_id: Query<Option<String>>,
        supplier: Query<String>,
        ext: Query<Option<String>>,
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<RbumCertSummaryWithSkResp>> {
        let funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let rbum_cert = IamCertServ::find_3th_kind_cert(rel_rbum_id.0, Some(vec![supplier.0]), false, ext.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(rbum_cert)
    }

    /// Get Third-kind Certs By Account Id
    ///
    /// 根据账号id获取第三方证书
    #[oai(path = "/third-kind", method = "get")]
    async fn get_third_cert(
        &self,
        account_id: Query<Option<String>>,
        supplier: Query<Option<String>>,
        ext: Query<Option<String>>,
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<RbumCertSummaryWithSkResp> {
        let funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let rbum_cert = IamCertServ::get_3th_kind_cert_by_rel_rbum_id(account_id.0, supplier.0.map(|supplier| vec![supplier]), true, ext.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(rbum_cert)
    }

    /// Auto Sync
    ///
    /// 定时任务触发第三方集成同步
    #[oai(path = "/sync", method = "get")]
    async fn third_integration_sync(&self, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        let funs = iam_constants::get_tardis_inst();
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let msg = IamCertServ::third_integration_sync_without_config(&funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        if let Some(notify_events) = rbum_event_helper::get_notify_event_with_ctx(&ctx.0).await? {
            rbum_event_helper::try_notifies(notify_events, &iam_constants::get_tardis_inst(), &ctx.0).await?;
        }
        TardisResp::ok(msg)
    }

    /// Decode cert
    ///
    /// 解码证书
    #[oai(path = "/decode", method = "post")]
    async fn decode_certs(&self, body: Json<IamCertDecodeRequest>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<HashMap<String, String>> {
        let mut funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        let ctx = IamCertServ::use_sys_or_tenant_ctx_unsafe(ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        funs.begin().await?;
        let doceded = IamCertServ::batch_decode_cert(body.0.codes, &funs, &ctx).await?;
        funs.commit().await?;
        TardisResp::ok(doceded)
    }
}

/// # Interface Console Ldap Cert API
/// 接口控制台Ldap证书API
#[poem_openapi::OpenApi(prefix_path = "/ci/ldap", tag = "bios_basic::ApiTag::Interface")]
impl IamCiLdapCertApi {
    /// Query the corresponding displayName according to ldap cn
    /// 根据ldap cn查询对应的displayName
    #[oai(path = "/cert/cn/:cn", method = "get")]
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

    /// Get Ldap Cert Conf
    /// 获取ldap证书配置
    #[oai(path = "/conf", method = "get")]
    async fn get_ldap_cert(
        &self,
        supplier: Query<String>,
        tenant_id: Query<Option<String>>,
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<IamCertConfLdapResp> {
        let mut funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0.clone())?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        funs.begin().await?;
        let conf_id = if let Ok(conf_id) = IamCertServ::get_cert_conf_id_by_kind_supplier("Ldap", &supplier.0, tenant_id.0, &funs).await {
            conf_id
        } else {
            return TardisResp::err(TardisError::bad_request("ldap config not found", ""));
        };
        let resp = IamCertLdapServ::get_cert_conf(&conf_id, &funs, &ctx).await?;
        funs.commit().await?;
        TardisResp::ok(resp)
    }
}
