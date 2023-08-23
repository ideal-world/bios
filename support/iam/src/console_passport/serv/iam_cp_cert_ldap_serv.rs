use crate::basic::dto::iam_account_dto::{IamAccountInfoResp, IamAccountInfoWithUserPwdAkResp, IamCpUserPwdBindResp};
use crate::basic::serv::iam_cert_ldap_serv::IamCertLdapServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::console_passport::dto::iam_cp_cert_dto::{IamCpLdapLoginReq, IamCpUserPwdBindWithLdapReq, IamCpUserPwdCheckReq};
use crate::iam_enumeration::{IamCertKernelKind, IamCertTokenKind};
use std::collections::HashMap;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::TardisFunsInst;

pub struct IamCpCertLdapServ;

impl IamCpCertLdapServ {
    pub async fn login_or_register(login_req: &IamCpLdapLoginReq, ip: Option<String>, funs: &TardisFunsInst) -> TardisResult<IamAccountInfoWithUserPwdAkResp> {
        let ldap_info = IamCertLdapServ::get_account_with_verify(
            login_req.name.as_ref(),
            login_req.password.as_ref(),
            login_req.tenant_id.clone(),
            login_req.code.as_ref(),
            funs,
        )
        .await?;
        let mock_ctx = IamCertLdapServ::generate_default_mock_ctx(login_req.code.as_ref(), login_req.tenant_id.clone(), funs).await;
        let resp = if let Some((account_id, access_token)) = ldap_info {
            let (ak, status) = Self::get_pwd_cert_name(&account_id, funs, &mock_ctx).await?;
            let iam_account_info_resp = IamCertServ::package_tardis_context_and_resp(
                login_req.tenant_id.clone(),
                &account_id,
                Some(IamCertTokenKind::TokenDefault.to_string()),
                Some(access_token),
                ip,
                funs,
            )
            .await?;
            IamAccountInfoWithUserPwdAkResp {
                iam_account_info_resp,
                ak,
                status,
            }
        } else {
            let iam_account_info_resp = IamAccountInfoResp {
                account_id: "".to_string(),
                account_name: "".to_string(),
                token: "".to_string(),
                access_token: None,
                roles: HashMap::new(),
                groups: HashMap::new(),
                apps: vec![],
            };
            IamAccountInfoWithUserPwdAkResp {
                iam_account_info_resp,
                ak: "".into(),
                status: "".into(),
            }
        };

        Ok(resp)
    }

    pub async fn check_user_pwd_is_bind(check_req: &IamCpUserPwdCheckReq, funs: &TardisFunsInst) -> TardisResult<IamCpUserPwdBindResp> {
        let is_bind = IamCertLdapServ::check_user_pwd_is_bind(check_req.ak.to_string().as_ref(), check_req.code.to_string().as_ref(), check_req.tenant_id.clone(), funs).await?;
        Ok(IamCpUserPwdBindResp { is_bind })
    }

    pub async fn bind_or_create_user_pwd_by_ldap(
        login_req: &IamCpUserPwdBindWithLdapReq,
        ip: Option<String>,
        funs: &TardisFunsInst,
    ) -> TardisResult<IamAccountInfoWithUserPwdAkResp> {
        let (account_id, access_token) = IamCertLdapServ::bind_or_create_user_pwd_by_ldap(login_req, funs).await?;

        let iam_account_info_resp = IamCertServ::package_tardis_context_and_resp(
            login_req.tenant_id.clone(),
            &account_id,
            Some(IamCertTokenKind::TokenDefault.to_string()),
            Some(access_token.clone()),
            ip,
            funs,
        )
        .await?;
        let mock_ctx = IamCertLdapServ::generate_default_mock_ctx(login_req.ldap_login.code.as_ref(), login_req.tenant_id.clone(), funs).await;
        let (ak, status) = Self::get_pwd_cert_name(&account_id, funs, &mock_ctx).await?;
        let resp = IamAccountInfoWithUserPwdAkResp {
            iam_account_info_resp,
            ak,
            status,
        };
        Ok(resp)
    }
    /// return String or "" empty String
    async fn get_pwd_cert_name(account_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<(String, String)> {
        let resp = IamCertServ::get_kernel_cert(account_id, &IamCertKernelKind::UserPwd, funs, ctx).await;
        if let Ok(with_sk_resp) = resp {
            Ok((with_sk_resp.ak, with_sk_resp.status.to_string()))
        } else {
            Ok(("".into(), "".into()))
        }
    }
}
