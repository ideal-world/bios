use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::error::TardisError;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::mail::mail_client::{TardisMailClient, TardisMailSendReq};
use tardis::rand::Rng;

use bios_basic::rbum::dto::rbum_cert_conf_dto::{RbumCertConfAddReq, RbumCertConfModifyReq};
use bios_basic::rbum::dto::rbum_cert_dto::{RbumCertAddReq, RbumCertModifyReq};
use bios_basic::rbum::dto::rbum_filer_dto::RbumCertFilterReq;
use bios_basic::rbum::rbum_enumeration::{RbumCertRelKind, RbumCertStatusKind};
use bios_basic::rbum::serv::rbum_cert_serv::{RbumCertConfServ, RbumCertServ};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_cert_conf_dto::IamMailVCodeCertConfAddOrModifyReq;
use crate::basic::dto::iam_cert_dto::IamMailVCodeCertAddReq;
use crate::basic::dto::iam_filer_dto::IamAccountFilterReq;
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::iam_config::{IamBasicInfoManager, IamConfig};
use crate::iam_enumeration::IamCertKind;

pub struct IamCertMailVCodeServ;

impl<'a> IamCertMailVCodeServ {
    pub async fn add_cert_conf(
        add_req: &IamMailVCodeCertConfAddOrModifyReq,
        rel_tenant_id: Option<String>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<String> {
        let id = RbumCertConfServ::add_rbum(
            &mut RbumCertConfAddReq {
                code: TrimString(IamCertKind::MailVCode.to_string()),
                name: TrimString(IamCertKind::MailVCode.to_string()),
                note: None,
                ak_note: add_req.ak_note.clone(),
                ak_rule: add_req.ak_rule.clone(),
                sk_note: None,
                sk_rule: None,
                sk_need: Some(false),
                sk_dynamic: Some(true),
                sk_encrypted: Some(false),
                repeatable: None,
                is_basic: Some(false),
                rest_by_kinds: None,
                expire_sec: None,
                coexist_num: Some(1),
                conn_uri: None,
                rel_rbum_domain_id: IamBasicInfoManager::get().domain_iam_id.to_string(),
                rel_rbum_item_id: rel_tenant_id,
            },
            funs,
            cxt,
        )
        .await?;
        Ok(id)
    }

    pub async fn modify_cert_conf(id: &str, modify_req: &IamMailVCodeCertConfAddOrModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        RbumCertConfServ::modify_rbum(
            id,
            &mut RbumCertConfModifyReq {
                name: None,
                note: None,
                ak_note: modify_req.ak_note.clone(),
                ak_rule: modify_req.ak_rule.clone(),
                sk_note: None,
                sk_rule: None,
                sk_need: None,
                sk_encrypted: None,
                repeatable: None,
                is_basic: None,
                rest_by_kinds: None,
                expire_sec: None,
                coexist_num: None,
                conn_uri: None,
            },
            funs,
            cxt,
        )
        .await?;
        Ok(())
    }

    pub async fn add_cert(add_req: &IamMailVCodeCertAddReq, account_id: &str, rel_tenant_id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<String> {
        let vcode = Self::get_vcode();
        let id = RbumCertServ::add_rbum(
            &mut RbumCertAddReq {
                ak: TrimString(add_req.mail.trim().to_string()),
                sk: None,
                vcode: Some(TrimString(vcode.clone())),
                ext: None,
                start_time: None,
                end_time: None,
                conn_uri: None,
                status: RbumCertStatusKind::Pending,
                rel_rbum_cert_conf_id: Some(IamCertServ::get_cert_conf_id_by_code(IamCertKind::MailVCode.to_string().as_str(), Some(rel_tenant_id), funs).await?),
                rel_rbum_kind: RbumCertRelKind::Item,
                rel_rbum_id: account_id.to_string(),
            },
            funs,
            cxt,
        )
        .await?;
        Self::send_activation_mail(account_id, &add_req.mail, &vcode, funs, cxt).await?;
        Ok(id)
    }

    pub async fn resend_activation_mail(account_id: &str, mail: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        let vcode = Self::get_vcode();
        RbumCertServ::add_vcode_to_cache(mail, &vcode, &cxt.own_paths, funs).await?;
        Self::send_activation_mail(account_id, mail, &vcode, funs, cxt).await
    }

    async fn send_activation_mail(account_id: &str, mail: &str, vcode: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        let account_name = IamAccountServ::get_item(account_id, &IamAccountFilterReq::default(), funs, cxt).await?.name;
        let mut subject = funs.conf::<IamConfig>().mail_template_cert_activate_title.clone();
        let mut content = funs.conf::<IamConfig>().mail_template_cert_activate_content.clone();
        subject = subject.replace("{account_name}", &account_name).replace("{vcode}", vcode);
        content = content.replace("{account_name}", &account_name).replace("{vcode}", vcode);

        TardisMailClient::send_quiet(
            funs.module_code().to_string(),
            TardisMailSendReq {
                subject,
                txt_body: content,
                html_body: None,
                to: vec![mail.to_string()],
                reply_to: None,
                cc: None,
                bcc: None,
                from: None,
            },
        )?;
        Ok(())
    }

    pub async fn activate_mail(mail: &str, input_vcode: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        if let Some(cached_vcode) = RbumCertServ::get_and_delete_vcode_in_cache(mail, &cxt.own_paths, funs).await? {
            if cached_vcode == input_vcode {
                let certs = RbumCertServ::find_rbums(
                    &RbumCertFilterReq {
                        ak: Some(mail.to_string()),
                        status: Some(RbumCertStatusKind::Pending),
                        rel_rbum_kind: Some(RbumCertRelKind::Item),
                        rel_rbum_cert_conf_id: Some(
                            IamCertServ::get_cert_conf_id_by_code(IamCertKind::MailVCode.to_string().as_str(), Some(&IamTenantServ::get_id_by_cxt(cxt)?), funs).await?,
                        ),
                        ..Default::default()
                    },
                    None,
                    None,
                    funs,
                    cxt,
                )
                .await?;
                if certs.len() > 1 {
                    return Err(TardisError::NotFound(format!("there are multiple credentials of kind {:?}", IamCertKind::MailVCode)));
                }
                if let Some(cert) = certs.get(0) {
                    RbumCertServ::modify_rbum(
                        &cert.id,
                        &mut RbumCertModifyReq {
                            ext: None,
                            start_time: None,
                            end_time: None,
                            conn_uri: None,
                            status: Some(RbumCertStatusKind::Enabled),
                        },
                        funs,
                        cxt,
                    )
                    .await?;
                    return Ok(());
                } else {
                    return Err(TardisError::NotFound(format!("cannot find credential of kind {:?}", IamCertKind::MailVCode)));
                }
            }
        }
        return Err(TardisError::Unauthorized("Email or verification code error".to_string()));
    }

    pub async fn send_login_mail(mail: &str, tenant_id: &str, funs: &TardisFunsInst<'a>) -> TardisResult<()> {
        let vcode = Self::get_vcode();
        RbumCertServ::add_vcode_to_cache(mail, &vcode, tenant_id, funs).await?;
        let mut subject = funs.conf::<IamConfig>().mail_template_cert_login_title.clone();
        let mut content = funs.conf::<IamConfig>().mail_template_cert_login_content.clone();
        subject = subject.replace("{vcode}", &vcode);
        content = content.replace("{vcode}", &vcode);
        TardisMailClient::send_quiet(
            funs.module_code().to_string(),
            TardisMailSendReq {
                subject,
                txt_body: content,
                html_body: None,
                to: vec![mail.to_string()],
                reply_to: None,
                cc: None,
                bcc: None,
                from: None,
            },
        )?;
        Ok(())
    }

    fn get_vcode() -> String {
        let mut rand = tardis::rand::thread_rng();
        let vcode: i32 = rand.gen_range(1000..9999);
        format!("{}", vcode)
    }
}
