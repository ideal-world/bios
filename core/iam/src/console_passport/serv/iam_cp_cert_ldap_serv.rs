use crate::basic::dto::iam_account_dto::IamAccountInfoResp;
use crate::basic::serv::iam_cert_ldap_serv::IamCertLdapServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::console_passport::dto::iam_cp_cert_dto::IamCpLdapLoginReq;
use crate::iam_enumeration::IamCertTokenKind;
use tardis::basic::result::TardisResult;
use tardis::TardisFunsInst;

pub struct IamCpCertLdapServ;

impl IamCpCertLdapServ {
    pub async fn login_or_register(login_req: &IamCpLdapLoginReq, funs: &TardisFunsInst) -> TardisResult<IamAccountInfoResp> {
        let ldap_info = IamCertLdapServ::get_or_add_account_with_verify(
            login_req.name.as_ref(),
            login_req.password.as_ref(),
            login_req.tenant_id.as_ref(),
            login_req.code.as_ref(),
            funs,
        )
        .await?;
        IamCertServ::package_tardis_context_and_resp(
            Some(login_req.tenant_id.clone()),
            &ldap_info.0,
            Some(IamCertTokenKind::TokenDefault.to_string()),
            Some(ldap_info.1),
            funs,
        )
        .await
    }
}
