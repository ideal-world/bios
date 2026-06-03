use crate::basic::dto::iam_account_dto::IamAccountInfoResp;
use crate::basic::serv::iam_cert_oauth2_serv::IamCertOAuth2Serv;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::console_passport::dto::iam_cp_cert_dto::IamCpOAuth2LoginReq;
use crate::iam_enumeration::{IamCertExtKind, IamCertOAuth2Supplier, IamCertTokenKind};
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::TardisFunsInst;

pub struct IamCpCertOAuth2Serv;

impl IamCpCertOAuth2Serv {
    pub async fn get_ak(cert_supplier: IamCertOAuth2Supplier, tenant_id: String, funs: &TardisFunsInst) -> TardisResult<String> {
        let cert_conf_id = IamCertServ::get_cert_conf_id_by_kind_supplier(&IamCertExtKind::OAuth2.to_string(), &cert_supplier.to_string(), Some(tenant_id.clone()), funs).await?;
        let mock_ctx = TardisContext {
            own_paths: tenant_id,
            ..Default::default()
        };
        let cert_conf = IamCertOAuth2Serv::get_cert_conf(&cert_conf_id, funs, &mock_ctx).await?;
        Ok(cert_conf.ak)
    }

    pub async fn login_or_register(
        cert_supplier: IamCertOAuth2Supplier,
        login_req: &IamCpOAuth2LoginReq,
        ip: Option<String>,
        funs: &TardisFunsInst,
    ) -> TardisResult<IamAccountInfoResp> {
        let oauth_info = IamCertOAuth2Serv::get_or_add_account(cert_supplier, login_req.code.as_ref(), &login_req.tenant_id.to_string(), funs).await?;
        IamCertServ::package_tardis_context_and_resp(
            Some(login_req.tenant_id.clone()),
            &oauth_info.0,
            Some(IamCertTokenKind::TokenDefault.to_string()),
            Some(oauth_info.1),
            ip,
            funs,
        )
        .await
    }

    /// 手动绑定外部 OAuth2 身份到当前登录账号
    ///
    /// 用于已登录用户首次登录时主动关联两边账号；账号取自当前登录上下文 `ctx.owner`，返回绑定的 open_id。
    pub async fn bind(cert_supplier: IamCertOAuth2Supplier, login_req: &IamCpOAuth2LoginReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        IamCertOAuth2Serv::bind_cert_account(cert_supplier, login_req.code.as_ref(), &login_req.tenant_id, &ctx.owner, funs, ctx).await
    }

    /// token 置换：用当前登录账号已缓存的 refresh_token 向 Provider 换取新的 access_token
    ///
    /// 账号取自当前登录上下文 `ctx.owner`，租户取自 `ctx.own_paths`。
    pub async fn refresh_provider_token(
        cert_supplier: IamCertOAuth2Supplier,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<crate::basic::serv::iam_cert_oauth2_serv::IamCertOAuth2TokenInfo> {
        IamCertOAuth2Serv::refresh_provider_token(cert_supplier, &ctx.owner, &ctx.own_paths, funs).await
    }

    /// 通过当前登录账号已缓存的 access_token 向 Provider 查询用户信息
    ///
    /// 账号取自当前登录上下文 `ctx.owner`，租户取自 `ctx.own_paths`，返回 Provider 原始用户信息 JSON。
    pub async fn get_provider_user_info(cert_supplier: IamCertOAuth2Supplier, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<tardis::serde_json::Value> {
        IamCertOAuth2Serv::get_provider_user_info(cert_supplier, &ctx.owner, &ctx.own_paths, funs).await
    }
}
