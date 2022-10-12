use std::collections::HashMap;
use crate::basic::dto::iam_account_dto::{IamAccountInfoResp, IamCpUserPwdBindResp};
use crate::basic::serv::iam_cert_ldap_serv::IamCertLdapServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::console_passport::dto::iam_cp_cert_dto::{IamCpLdapLoginReq, IamCpUserPwdBindReq, IamCpUserPwdBindWithLdapReq, IamCpUserPwdCheckReq};
use crate::iam_enumeration::IamCertTokenKind;
use tardis::basic::result::TardisResult;
use tardis::db::sea_orm::sea_query::ColumnSpec::Default;
use tardis::TardisFunsInst;
use tardis::web::web_resp::TardisApiResult;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;

pub struct IamCpCertLdapServ;

impl IamCpCertLdapServ {
    pub async fn login_or_register(login_req: &IamCpLdapLoginReq, funs: &TardisFunsInst) -> TardisResult<IamAccountInfoResp> {
        let ldap_info = IamCertLdapServ::get_account_with_verify(
            login_req.name.as_ref(),
            login_req.password.as_ref(),
            login_req.tenant_id.as_ref(),
            login_req.code.as_ref(),
            funs,
        )
        .await?;
        if let Some((account_id, access_token)) = ldap_info {
            IamCertServ::package_tardis_context_and_resp(
                Some(login_req.tenant_id.clone()),
                &account_id,
                Some(IamCertTokenKind::TokenDefault.to_string()),
                Some(access_token),
                funs,
            ).await
        } else {
            Ok(IamAccountInfoResp {
                account_id: "".to_string(),
                account_name: "".to_string(),
                token: "".to_string(),
                access_token: None,
                roles: HashMap::new(),
                groups: HashMap::new(),
                apps: vec![],
            })
        }
    }

    pub async fn check_user_pwd_is_bind(check_req: &IamCpUserPwdCheckReq, funs: &TardisFunsInst) -> TardisResult<IamCpUserPwdBindResp> {
        let is_bind = IamCertLdapServ::check_user_pwd_is_bind(check_req.ak.to_string().as_ref(), check_req.code.to_string().as_ref(), check_req.tenant_id.as_ref(), funs).await?;
        Ok(IamCpUserPwdBindResp { is_bind })
    }

    pub async fn bind_or_create_user_pwd_by_ldap(login_req: &IamCpUserPwdBindWithLdapReq, funs: &TardisFunsInst) -> TardisResult<IamAccountInfoResp> {
        let mut account_id: String = String::from("");
        let mut access_token: String = String::from("");
        (account_id, access_token)=IamCertLdapServ::bind_or_create_user_pwd_by_ldap(login_req,funs).await?;

        IamCertServ::package_tardis_context_and_resp(
            Some(login_req.tenant_id.clone()),
            &account_id,
            Some(IamCertTokenKind::TokenDefault.to_string()),
            Some(access_token.clone()),
            funs,
        ).await

    }
}
