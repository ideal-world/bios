use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::result::TardisResult;
use tardis::TardisFuns;

use bios_basic::rbum::serv::rbum_cert_serv::RbumCertServ;

use crate::basic::dto::iam_cert_dto::IamUserPwdCertModifyReq;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_cert_token_serv::IamCertTokenServ;
use crate::basic::serv::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::console_passport::dto::iam_cp_cert_dto::{IamCpUserPwdLoginReq, LoginResp};
use crate::iam_enumeration::{IamCertKind, IamCertTokenKind};

pub struct IamCpCertUserPwdServ;

impl<'a> IamCpCertUserPwdServ {
    pub async fn login_by_user_pwd(login_req: &mut IamCpUserPwdLoginReq, funs: &TardisFunsInst<'a>) -> TardisResult<LoginResp> {
        let tenant_id = if let Some(tenant_id) = &login_req.tenant_id { tenant_id } else { "" };
        let rbum_cert_conf_id = IamCertServ::get_id_by_code(&IamCertKind::UserPwd.to_string(), Some(tenant_id), funs).await?;
        let (rbum_cert_id, _, rbum_item_id) = RbumCertServ::validate(&login_req.ak.0, &login_req.sk.0, &rbum_cert_conf_id, tenant_id, funs).await?;
        let token = TardisFuns::crypto.key.generate_token()?;
        let token_kind = IamCertTokenKind::parse(&login_req.flag);
        let (context, resp) =
            IamCertServ::package_tardis_context_and_resp(Some(tenant_id), None, &login_req.ak.0, &rbum_item_id, Some(&token), Some(&token_kind.to_string()), funs).await?;
        IamCertTokenServ::add_cert(&token, token_kind, &rbum_item_id, tenant_id, &rbum_cert_id, funs, &context).await?;
        Ok(resp)
    }

    pub async fn modify_cert_user_pwd(modify_req: &mut IamUserPwdCertModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        IamCertUserPwdServ::modify_cert(modify_req, &cxt.owner, &IamTenantServ::get_id_by_cxt(cxt)?, funs, cxt).await
    }
}
