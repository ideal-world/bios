use tardis::TardisFunsInst;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;

use bios_basic::rbum::helper::rbum_scope_helper::get_max_level_id_by_context;
use bios_basic::rbum::serv::rbum_cert_serv::RbumCertServ;

use crate::basic::dto::iam_account_dto::AccountInfoResp;
use crate::basic::dto::iam_cert_dto::IamMailVCodeCertAddReq;
use crate::basic::serv::iam_cert_mail_vcode_serv::IamCertMailVCodeServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::console_passport::dto::iam_cp_cert_dto::IamCpMailVCodeLoginReq;
use crate::iam_enumeration::IamCertKind;

pub struct IamCpCertMailVCodeServ;

impl<'a> IamCpCertMailVCodeServ {
    pub async fn add_cert_mail_vocde(add_req: &IamMailVCodeCertAddReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<String> {
        let rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_code(IamCertKind::MailVCode.to_string().as_str(), get_max_level_id_by_context(cxt), funs).await?;
        IamCertMailVCodeServ::add_cert(add_req, &cxt.owner, &rbum_cert_conf_id, funs, cxt).await
    }

    pub async fn login_by_mail_vocde(login_req: &IamCpMailVCodeLoginReq, funs: &TardisFunsInst<'a>) -> TardisResult<AccountInfoResp> {
        let rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_code(&IamCertKind::MailVCode.to_string(), Some(login_req.tenant_id.clone()), funs).await?;
        let (_, _, rbum_item_id) = RbumCertServ::validate(&login_req.mail, &login_req.vcode.0, &rbum_cert_conf_id, &login_req.tenant_id, funs).await?;
        let resp = IamCertServ::package_tardis_context_and_resp(Some(login_req.tenant_id.clone()), &login_req.mail, &rbum_item_id, login_req.flag.clone(), funs).await?;
        Ok(resp)
    }
}
