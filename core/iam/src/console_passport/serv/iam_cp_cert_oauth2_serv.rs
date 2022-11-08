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

    pub async fn login_or_register(cert_supplier: IamCertOAuth2Supplier, login_req: &IamCpOAuth2LoginReq, funs: &TardisFunsInst) -> TardisResult<IamAccountInfoResp> {
        let oauth_info = IamCertOAuth2Serv::get_or_add_account(cert_supplier, login_req.code.as_ref(), &login_req.tenant_id.to_string(), funs).await?;
        IamCertServ::package_tardis_context_and_resp(
            Some(login_req.tenant_id.clone()),
            &oauth_info.0,
            Some(IamCertTokenKind::TokenDefault.to_string()),
            Some(oauth_info.1),
            funs,
        )
        .await
    }
}
