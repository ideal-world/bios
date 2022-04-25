use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::result::TardisResult;

use bios_basic::rbum::serv::rbum_cert_serv::RbumCertServ;

use crate::basic::dto::iam_account_dto::AccountInfoResp;
use crate::basic::dto::iam_cert_dto::IamMailVCodeCertAddReq;
use crate::basic::serv::iam_cert_mail_vcode_serv::IamCertMailVCodeServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::console_passport::dto::iam_cp_cert_dto::{IamCpMailVCodeLoginGenVCodeReq, IamCpMailVCodeLoginReq};
use crate::iam_enumeration::IamCertKind;

pub struct IamCpCertMailVCodeServ;

impl<'a> IamCpCertMailVCodeServ {
    pub async fn add_cert_mail_vocde(add_req: &IamMailVCodeCertAddReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<String> {
        IamCertMailVCodeServ::add_cert(add_req, &cxt.owner, &IamTenantServ::get_id_by_cxt(cxt)?, funs, cxt).await
    }

    pub async fn delete_cert_mail_vocde(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        IamCertServ::delete_cert(id, funs, cxt).await
    }

    pub async fn resend_activation_mail(mail: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        IamCertMailVCodeServ::resend_activation_mail(&cxt.owner, mail, funs, cxt).await
    }

    pub async fn activate_mail(mail: &str, vcode: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        IamCertMailVCodeServ::activate_mail(mail, vcode, funs, cxt).await
    }

    pub async fn send_login_mail(req: &IamCpMailVCodeLoginGenVCodeReq, funs: &TardisFunsInst<'a>) -> TardisResult<()> {
        IamCertMailVCodeServ::send_login_mail(&req.mail, &req.tenant_id, funs).await
    }

    pub async fn login_by_mail_vocde(login_req: &IamCpMailVCodeLoginReq, funs: &TardisFunsInst<'a>) -> TardisResult<AccountInfoResp> {
        let rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_code(&IamCertKind::MailVCode.to_string(), Some(&login_req.tenant_id), funs).await?;
        let (rbum_cert_id, _, rbum_item_id) = RbumCertServ::validate(&login_req.mail, &login_req.vcode.0, &rbum_cert_conf_id, &login_req.tenant_id, funs).await?;
        let resp = IamCertServ::package_tardis_context_and_resp(
            Some(login_req.tenant_id.clone()),
            &login_req.mail,
            &rbum_item_id,
            &rbum_cert_id,
            login_req.flag.clone(),
            funs,
        )
        .await?;
        Ok(resp)
    }
}
