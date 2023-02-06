use bios_basic::rbum::dto::rbum_filer_dto::{RbumCertConfFilterReq, RbumCertFilterReq};
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::mail::mail_client::{TardisMailClient, TardisMailSendReq};
use tardis::rand::Rng;
use tardis::TardisFunsInst;

use bios_basic::rbum::dto::rbum_cert_conf_dto::{RbumCertConfAddReq, RbumCertConfModifyReq};
use bios_basic::rbum::dto::rbum_cert_dto::{RbumCertAddReq, RbumCertModifyReq};
use bios_basic::rbum::rbum_enumeration::{RbumCertConfStatusKind, RbumCertRelKind, RbumCertStatusKind};
use bios_basic::rbum::serv::rbum_cert_serv::{RbumCertConfServ, RbumCertServ};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;

use crate::basic::dto::iam_cert_conf_dto::IamCertConfPhoneVCodeAddOrModifyReq;
use crate::basic::dto::iam_cert_dto::IamCertPhoneVCodeAddReq;
use crate::basic::dto::iam_filer_dto::IamAccountFilterReq;
use crate::iam_config::{IamBasicConfigApi, IamConfig};
use crate::iam_enumeration::IamCertKernelKind;

use super::iam_account_serv::IamAccountServ;
use super::iam_cert_serv::IamCertServ;
use super::iam_tenant_serv::IamTenantServ;

pub struct IamCertPhoneVCodeServ;

impl IamCertPhoneVCodeServ {
    pub async fn add_cert_conf(add_req: &IamCertConfPhoneVCodeAddOrModifyReq, rel_iam_item_id: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let id = RbumCertConfServ::add_rbum(
            &mut RbumCertConfAddReq {
                kind: TrimString(IamCertKernelKind::PhoneVCode.to_string()),
                supplier: None,
                name: TrimString(IamCertKernelKind::PhoneVCode.to_string()),
                note: None,
                ak_note: add_req.ak_note.clone(),
                ak_rule: add_req.ak_rule.clone(),
                sk_note: None,
                sk_rule: None,
                ext: None,
                sk_need: Some(false),
                sk_dynamic: None,
                sk_encrypted: Some(false),
                repeatable: None,
                is_basic: Some(false),
                is_ak_repeatable: None,
                rest_by_kinds: None,
                expire_sec: None,
                sk_lock_cycle_sec: None,
                sk_lock_err_times: None,
                sk_lock_duration_sec: None,
                coexist_num: Some(1),
                conn_uri: None,
                status: RbumCertConfStatusKind::Enabled,
                rel_rbum_domain_id: funs.iam_basic_domain_iam_id(),
                rel_rbum_item_id: rel_iam_item_id,
            },
            funs,
            ctx,
        )
        .await?;
        Ok(id)
    }

    pub async fn modify_cert_conf(id: &str, modify_req: &IamCertConfPhoneVCodeAddOrModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        RbumCertConfServ::modify_rbum(
            id,
            &mut RbumCertConfModifyReq {
                name: None,
                note: None,
                ak_note: modify_req.ak_note.clone(),
                ak_rule: modify_req.ak_rule.clone(),
                sk_note: None,
                sk_rule: None,
                ext: None,
                sk_need: None,
                sk_encrypted: None,
                repeatable: None,
                is_basic: None,
                rest_by_kinds: None,
                expire_sec: None,
                sk_lock_cycle_sec: None,
                sk_lock_err_times: None,
                sk_lock_duration_sec: None,
                coexist_num: None,
                conn_uri: None,
                status: None,
            },
            funs,
            ctx,
        )
        .await?;
        Ok(())
    }

    pub async fn add_cert(add_req: &IamCertPhoneVCodeAddReq, account_id: &str, rel_rbum_cert_conf_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let vcode = Self::get_vcode();
        let id = RbumCertServ::add_rbum(
            &mut RbumCertAddReq {
                ak: TrimString(add_req.phone.to_string()),
                sk: None,
                kind: None,
                supplier: None,
                vcode: Some(TrimString(vcode.clone())),
                ext: None,
                start_time: None,
                end_time: None,
                conn_uri: None,
                status: RbumCertStatusKind::Pending,
                rel_rbum_cert_conf_id: Some(rel_rbum_cert_conf_id.to_string()),
                rel_rbum_kind: RbumCertRelKind::Item,
                rel_rbum_id: account_id.to_string(),
                is_outside: false,
            },
            funs,
            ctx,
        )
        .await?;
        // TODO send vcode
        Self::send_activation_phone(account_id, &add_req.phone, &vcode, funs, ctx).await?;
        Ok(id)
    }

    pub async fn resend_activation_phone(account_id: &str, phone: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let vcode = Self::get_vcode();
        RbumCertServ::add_vcode_to_cache(phone, &vcode, &ctx.own_paths, funs).await?;
        Self::send_activation_phone(account_id, phone, &vcode, funs, ctx).await
    }

    async fn send_activation_phone(account_id: &str, phone: &str, vcode: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let account_name = IamAccountServ::peek_item(account_id, &IamAccountFilterReq::default(), funs, ctx).await?.name;
        let mut subject = funs.conf::<IamConfig>().phone_template_cert_activate_title.clone();
        let mut content = funs.conf::<IamConfig>().phone_template_cert_activate_content.clone();
        subject = subject.replace("{account_name}", &account_name).replace("{vcode}", vcode);
        content = content.replace("{account_name}", &account_name).replace("{vcode}", vcode);

        TardisMailClient::send_quiet(
            funs.module_code().to_string(),
            TardisMailSendReq {
                subject,
                txt_body: content,
                html_body: None,
                to: vec![phone.to_string()],
                reply_to: None,
                cc: None,
                bcc: None,
                from: None,
            },
        )?;
        Ok(())
    }

    pub async fn activate_phone(phone: &str, input_vcode: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if let Some(cached_vcode) = RbumCertServ::get_and_delete_vcode_in_cache(phone, &ctx.own_paths, funs).await? {
            if cached_vcode == input_vcode {
                let cert = RbumCertServ::find_one_rbum(
                    &RbumCertFilterReq {
                        ak: Some(phone.to_string()),
                        status: Some(RbumCertStatusKind::Pending),
                        rel_rbum_kind: Some(RbumCertRelKind::Item),
                        rel_rbum_cert_conf_ids: Some(vec![
                            IamCertServ::get_cert_conf_id_by_kind(IamCertKernelKind::PhoneVCode.to_string().as_str(), Some(IamTenantServ::get_id_by_ctx(ctx, funs)?), funs).await?,
                        ]),
                        ..Default::default()
                    },
                    funs,
                    ctx,
                )
                .await?;
                return if let Some(cert) = cert {
                    RbumCertServ::modify_rbum(
                        &cert.id,
                        &mut RbumCertModifyReq {
                            status: Some(RbumCertStatusKind::Enabled),
                            ak: None,
                            sk: None,
                            ext: None,
                            start_time: None,
                            end_time: None,
                            conn_uri: None,
                        },
                        funs,
                        ctx,
                    )
                    .await?;
                    Ok(())
                } else {
                    Err(funs.err().not_found(
                        "iam_cert_phone_vcode",
                        "activate",
                        &format!("not found credential of kind {:?}", IamCertKernelKind::PhoneVCode),
                        "404-iam-cert-kind-not-exist",
                    ))
                };
            }
        }
        Err(funs.err().unauthorized("iam_cert_phone_vcode", "activate", "email or verification code error", "401-iam-cert-valid"))
    }

    pub async fn send_bind_phone(phone: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if RbumCertServ::count_rbums(
            &RbumCertFilterReq {
                ak: Some(phone.to_string()),
                rel_rbum_kind: Some(RbumCertRelKind::Item),
                rel_rbum_cert_conf_ids: Some(vec![
                    IamCertServ::get_cert_conf_id_by_kind(IamCertKernelKind::PhoneVCode.to_string().as_str(), Some(IamTenantServ::get_id_by_ctx(ctx, funs)?), funs).await?,
                ]),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
            > 0
        {
            return Err(funs.err().unauthorized("iam_cert_phone_vcode", "activate", "phone already exist", "401-iam-cert-valid"));
        }
        let vcode = Self::get_vcode();
        let account_name = IamAccountServ::peek_item(&ctx.owner, &IamAccountFilterReq::default(), funs, ctx).await?.name;
        RbumCertServ::add_vcode_to_cache(phone, &vcode, &ctx.own_paths, funs).await?;
        let mut subject = funs.conf::<IamConfig>().phone_template_cert_activate_title.clone();
        let mut content = funs.conf::<IamConfig>().phone_template_cert_activate_content.clone();
        subject = subject.replace("{account_name}", &account_name).replace("{vcode}", &vcode);
        content = content.replace("{account_name}", &account_name).replace("{vcode}", &vcode);
        TardisMailClient::send_quiet(
            funs.module_code().to_string(),
            TardisMailSendReq {
                subject,
                txt_body: content,
                html_body: None,
                to: vec![phone.to_string()],
                reply_to: None,
                cc: None,
                bcc: None,
                from: None,
            },
        )?;
        Ok(())
    }

    pub async fn bind_phone(phone: &str, input_vcode: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        if let Some(cached_vcode) = RbumCertServ::get_and_delete_vcode_in_cache(phone, &ctx.own_paths, funs).await? {
            if cached_vcode == input_vcode {
                let rel_rbum_cert_conf_id =
                    IamCertServ::get_cert_conf_id_by_kind(IamCertKernelKind::PhoneVCode.to_string().as_str(), Some(IamTenantServ::get_id_by_ctx(ctx, funs)?), funs).await?;
                let id = RbumCertServ::add_rbum(
                    &mut RbumCertAddReq {
                        ak: TrimString(phone.trim().to_string()),
                        sk: None,
                        kind: None,
                        supplier: None,
                        vcode: Some(TrimString(input_vcode.to_string())),
                        ext: None,
                        start_time: None,
                        end_time: None,
                        conn_uri: None,
                        status: RbumCertStatusKind::Enabled,
                        rel_rbum_cert_conf_id: Some(rel_rbum_cert_conf_id),
                        rel_rbum_kind: RbumCertRelKind::Item,
                        rel_rbum_id: ctx.owner.clone(),
                        is_outside: false,
                    },
                    funs,
                    ctx,
                )
                .await?;
                return Ok(id);
            }
        }
        Err(funs.err().unauthorized("iam_cert_phone_vcode", "activate", "phone or verification code error", "401-iam-cert-valid"))
    }

    pub async fn send_login_phone(phone: &str, own_paths: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        let vcode = Self::get_vcode();
        RbumCertServ::add_vcode_to_cache(phone, &vcode, own_paths, funs).await?;
        let mut subject = funs.conf::<IamConfig>().phone_template_cert_login_title.clone();
        let mut content = funs.conf::<IamConfig>().phone_template_cert_login_content.clone();
        subject = subject.replace("{vcode}", &vcode);
        content = content.replace("{vcode}", &vcode);
        // todo send sms web
        Ok(())
    }

    fn get_vcode() -> String {
        let mut rand = tardis::rand::thread_rng();
        let vcode: i32 = rand.gen_range(1000..9999);
        format!("{vcode}")
    }

    pub async fn add_or_enable_cert_conf(
        add_req: &IamCertConfPhoneVCodeAddOrModifyReq,
        rel_iam_item_id: Option<String>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<String> {
        let cert_result = RbumCertConfServ::do_find_one_rbum(
            &RbumCertConfFilterReq {
                kind: Some(TrimString(IamCertKernelKind::PhoneVCode.to_string())),
                rel_rbum_item_id: rel_iam_item_id.clone(),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        let result = if let Some(cert_result) = cert_result {
            IamCertServ::enabled_cert_conf(&cert_result.id, funs, ctx).await?;
            cert_result.id
        } else {
            Self::add_cert_conf(add_req, rel_iam_item_id, funs, ctx).await?
        };
        Ok(result)
    }

    // TODO
}
