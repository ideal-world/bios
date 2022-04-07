use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisRelDBlConnection;
use tardis::TardisFuns;

use bios_basic::rbum::serv::rbum_cert_serv::RbumCertServ;

use crate::basic::dto::iam_cert_dto::IamUserPwdCertModifyReq;
use crate::basic::enumeration::{IamCertKind, IamCertTokenKind};
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_cert_token_serv::IamCertTokenServ;
use crate::basic::serv::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::console_passport::dto::iam_cp_cert_dto::IamCpUserPwdLoginReq;

pub struct IamCpCertUserPwdServ;

impl<'a> IamCpCertUserPwdServ {
    pub async fn login_by_user_pwd(login_req: &mut IamCpUserPwdLoginReq, funs: &TardisFunsInst<'a>) -> TardisResult<TardisContext> {
        let rbum_cert_conf_id = IamCertServ::get_id_by_code(&IamCertKind::UserPwd.to_string(), Some(&login_req.tenant_id), db).await?;
        let (rbum_cert_id, rbum_item_id) = RbumCertServ::validate(&login_req.ak.0, &login_req.sk.0, &rbum_cert_conf_id, &login_req.tenant_id, db).await?;
        let token = TardisFuns::crypto.key.generate_token()?;
        let token_kind = IamCertTokenKind::parse(&login_req.flag);
        let tardis_context = IamCertServ::get_tardis_context(
            Some(&login_req.tenant_id),
            None,
            &login_req.ak.0,
            &rbum_item_id,
            Some(&token),
            Some(&token_kind.to_string()),
            db,
        )
        .await?;
        IamCertTokenServ::add_cert(&token, token_kind, &rbum_item_id, &login_req.tenant_id, &rbum_cert_id, db, &tardis_context).await?;
        Ok(tardis_context)
    }

    pub async fn modify_cert_user_pwd(modify_req: &mut IamUserPwdCertModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        IamCertUserPwdServ::modify_cert(modify_req, &cxt.account_id, &IamTenantServ::get_id_by_cxt(cxt)?, db, cxt).await
    }
}
