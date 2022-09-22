use crate::basic::dto::iam_account_dto::IamAccountInfoResp;
use crate::basic::serv::iam_cert_oauth2_by_code_serv::IamCertOAuth2ByCodeServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::console_passport::dto::iam_cp_cert_dto::IamCpOAuth2ByCodeLoginReq;
use crate::iam_enumeration::{IamCertExtKind, IamCertTokenKind};
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::TardisFunsInst;

pub struct IamCpCertOAuth2ByCodeServ;

impl IamCpCertOAuth2ByCodeServ {
    pub async fn get_ak(cert_kind: IamCertExtKind, tenant_id: String, funs: &TardisFunsInst) -> TardisResult<String> {
        let cert_conf_id = IamCertServ::get_cert_conf_id_by_code(&cert_kind.to_string(), Some(tenant_id.clone()), funs).await?;
        let mock_ctx = TardisContext {
            own_paths: tenant_id,
            ..Default::default()
        };
        let cert_conf = IamCertOAuth2ByCodeServ::get_cert_conf(&cert_conf_id, funs, &mock_ctx).await?;
        Ok(cert_conf.ak)
    }

    pub async fn login_or_register(cert_kind: IamCertExtKind, login_req: &IamCpOAuth2ByCodeLoginReq, funs: &TardisFunsInst) -> TardisResult<IamAccountInfoResp> {
        let oauth_info = IamCertOAuth2ByCodeServ::get_or_add_account(cert_kind, login_req.code.as_ref(), &login_req.tenant_id.to_string(), funs).await?;
        IamCertServ::package_tardis_context_and_resp(
            Some(login_req.tenant_id.clone()),
            &oauth_info.0,
            Some(IamCertTokenKind::TokenWechatMp.to_string()),
            Some(oauth_info.1),
            funs,
        )
        .await
    }
}
