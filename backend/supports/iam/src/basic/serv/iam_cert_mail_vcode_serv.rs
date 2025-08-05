use bios_basic::rbum::dto::rbum_cert_conf_dto::{RbumCertConfAddReq, RbumCertConfModifyReq};
use bios_basic::rbum::dto::rbum_cert_dto::{RbumCertAddReq, RbumCertModifyReq};
use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumCertConfFilterReq, RbumCertFilterReq};
use bios_basic::rbum::rbum_enumeration::{RbumCertConfStatusKind, RbumCertRelKind, RbumCertStatusKind};
use bios_basic::rbum::serv::rbum_cert_serv::{RbumCertConfServ, RbumCertServ};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::rand::Rng;
use tardis::TardisFunsInst;

use crate::basic::dto::iam_cert_conf_dto::IamCertConfMailVCodeAddOrModifyReq;
use crate::basic::dto::iam_cert_dto::{IamCertMailVCodeAddReq, IamCertMailVCodeModifyReq};
use crate::basic::dto::iam_filer_dto::IamAccountFilterReq;
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::iam_config::IamBasicConfigApi;
use crate::iam_enumeration::IamCertKernelKind;

use super::clients::iam_log_client::{IamLogClient, LogParamTag};
use super::clients::iam_search_client::IamSearchClient;
use super::clients::mail_client::MailClient;

pub struct IamCertMailVCodeServ;

impl IamCertMailVCodeServ {
    pub async fn add_cert_conf(add_req: &IamCertConfMailVCodeAddOrModifyReq, rel_iam_item_id: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let id = RbumCertConfServ::add_rbum(
            &mut RbumCertConfAddReq {
                kind: TrimString(IamCertKernelKind::MailVCode.to_string()),
                supplier: None,
                name: TrimString(IamCertKernelKind::MailVCode.to_string()),
                note: None,
                ak_note: add_req.ak_note.clone(),
                ak_rule: add_req.ak_rule.clone(),
                sk_note: None,
                sk_rule: None,
                ext: None,
                sk_need: Some(false),
                sk_dynamic: Some(true),
                sk_encrypted: Some(false),
                repeatable: None,
                is_basic: Some(false),
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

    pub async fn modify_cert_conf(id: &str, modify_req: &IamCertConfMailVCodeAddOrModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
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

    pub async fn add_cert(add_req: &IamCertMailVCodeAddReq, account_id: &str, rel_rbum_cert_conf_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let vcode = Self::get_vcode();
        let id = RbumCertServ::add_rbum(
            &mut RbumCertAddReq {
                ak: TrimString(add_req.mail.trim().to_string()),
                sk: None,
                sk_invisible: None,
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
                ignore_check_sk: false,
            },
            funs,
            ctx,
        )
        .await?;
        Self::send_activation_mail(account_id, &add_req.mail, &vcode, funs, ctx).await?;
        Ok(id)
    }

    pub async fn add_cert_skip_activate(
        add_req: &IamCertMailVCodeAddReq,
        account_id: &str,
        rel_rbum_cert_conf_id: &str,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<String> {
        let vcode = Self::get_vcode();
        let id = RbumCertServ::add_rbum(
            &mut RbumCertAddReq {
                ak: TrimString(add_req.mail.trim().to_string()),
                sk: None,
                sk_invisible: None,
                kind: None,
                supplier: None,
                vcode: Some(TrimString(vcode.clone())),
                ext: None,
                start_time: None,
                end_time: None,
                conn_uri: None,
                status: RbumCertStatusKind::Enabled,
                rel_rbum_cert_conf_id: Some(rel_rbum_cert_conf_id.to_string()),
                rel_rbum_kind: RbumCertRelKind::Item,
                rel_rbum_id: account_id.to_string(),
                is_outside: false,
                ignore_check_sk: false,
            },
            funs,
            ctx,
        )
        .await?;
        Ok(id)
    }

    pub async fn modify_cert(id: &str, modify_req: &IamCertMailVCodeModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        RbumCertServ::modify_rbum(
            id,
            &mut RbumCertModifyReq {
                ak: Some(TrimString(modify_req.mail.to_string())),
                sk: None,
                sk_invisible: None,

                ext: None,
                start_time: None,
                end_time: None,
                conn_uri: None,
                status: None,
                ignore_check_sk: false,
            },
            funs,
            ctx,
        )
        .await?;
        Ok(())
    }

    pub async fn add_or_modify_cert(mail: &str, account_id: &str, rel_rbum_cert_conf_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let resp = IamCertServ::get_kernel_cert(account_id, &IamCertKernelKind::MailVCode, funs, ctx).await;
        match resp {
            Ok(cert) => {
                Self::modify_cert(&cert.id, &IamCertMailVCodeModifyReq { mail: mail.to_string() }, funs, ctx).await?;
            }
            Err(_) => {
                Self::add_cert(&IamCertMailVCodeAddReq { mail: mail.to_string() }, account_id, rel_rbum_cert_conf_id, funs, ctx).await?;
            }
        }
        Ok(())
    }

    pub async fn resend_activation_mail(account_id: &str, mail: &str, cool_down_id_sec: Option<u32>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let vcode = Self::get_vcode();
        let rel_rbum_cert_conf_id =
            IamCertServ::get_cert_conf_id_by_kind(IamCertKernelKind::MailVCode.to_string().as_str(), Some(IamTenantServ::get_id_by_ctx(ctx, funs)?), funs).await?;
        RbumCertServ::add_vcode_to_cache(mail, &vcode, &rel_rbum_cert_conf_id, cool_down_id_sec, funs, ctx).await?;
        Self::send_activation_mail(account_id, mail, &vcode, funs, ctx).await
    }

    async fn send_activation_mail(account_id: &str, mail: &str, vcode: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let account_name = IamAccountServ::peek_item(account_id, &IamAccountFilterReq::default(), funs, ctx).await?.name;
        MailClient::send_cert_activate_vcode(mail, Some(account_name), vcode, funs).await?;
        Ok(())
    }

    pub async fn activate_mail(mail: &str, input_vcode: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let ctx = IamAccountServ::new_context_if_account_is_global(ctx, funs).await?;
        if let Some(cached_vcode) = RbumCertServ::get_vcode_in_cache(mail, &ctx.own_paths, funs).await? {
            if cached_vcode == input_vcode {
                let cert = RbumCertServ::find_one_rbum(
                    &RbumCertFilterReq {
                        ak: Some(mail.to_string()),
                        status: Some(RbumCertStatusKind::Pending),
                        rel_rbum_kind: Some(RbumCertRelKind::Item),
                        rel_rbum_cert_conf_ids: Some(vec![
                            IamCertServ::get_cert_conf_id_by_kind(IamCertKernelKind::MailVCode.to_string().as_str(), Some(IamTenantServ::get_id_by_ctx(&ctx, funs)?), funs).await?,
                        ]),
                        ..Default::default()
                    },
                    funs,
                    &ctx,
                )
                .await?;
                return if let Some(cert) = cert {
                    RbumCertServ::modify_rbum(
                        &cert.id,
                        &mut RbumCertModifyReq {
                            status: Some(RbumCertStatusKind::Enabled),
                            ak: None,
                            sk: None,
                            sk_invisible: None,

                            ignore_check_sk: false,
                            ext: None,
                            start_time: None,
                            end_time: None,
                            conn_uri: None,
                        },
                        funs,
                        &ctx,
                    )
                    .await?;
                    Ok(())
                } else {
                    Err(funs.err().not_found(
                        "iam_cert_mail_vcode",
                        "activate",
                        &format!("not found credential of kind {:?}", IamCertKernelKind::MailVCode),
                        "404-iam-cert-kind-not-exist",
                    ))
                };
            }
        }
        Err(funs.err().unauthorized("iam_cert_mail_vcode", "activate", "email or verification code error", "401-iam-cert-valid"))
    }

    pub async fn send_bind_mail(mail: &str, cool_down_id_sec: Option<u32>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let ctx = IamAccountServ::new_context_if_account_is_global(ctx, funs).await?;
        let rel_rbum_cert_conf_id =
            IamCertServ::get_cert_conf_id_by_kind(IamCertKernelKind::MailVCode.to_string().as_str(), Some(IamTenantServ::get_id_by_ctx(&ctx, funs)?), funs).await?;
        // Self::check_bind_mail(mail, vec![rel_rbum_cert_conf_id], &ctx.owner, funs, &ctx).await?;
        let vcode = Self::get_vcode();
        let account_name = IamAccountServ::peek_item(&ctx.owner, &IamAccountFilterReq::default(), funs, &ctx).await?.name;
        RbumCertServ::add_vcode_to_cache(mail, &vcode, &rel_rbum_cert_conf_id, cool_down_id_sec, funs, &ctx).await?;
        MailClient::send_cert_activate_vcode(mail, Some(account_name), &vcode, funs).await?;
        Ok(())
    }

    pub async fn bind_mail(mail: &str, input_vcode: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let ctx = IamAccountServ::new_context_if_account_is_global(ctx, funs).await?;
        if let Some(cached_vcode) = RbumCertServ::get_vcode_in_cache(mail, &ctx.own_paths, funs).await? {
            if cached_vcode == input_vcode {
                let rel_rbum_cert_conf_id =
                    IamCertServ::get_cert_conf_id_by_kind(IamCertKernelKind::MailVCode.to_string().as_str(), Some(IamTenantServ::get_id_by_ctx(&ctx, funs)?), funs).await?;
                Self::check_mail_bound(mail, vec![rel_rbum_cert_conf_id.clone()], funs, &ctx).await?;
                let id = if Self::check_account_bind_mail(vec![rel_rbum_cert_conf_id.clone()], &ctx.owner.clone(), funs, &ctx).await.is_ok() {
                    RbumCertServ::add_rbum(
                        &mut RbumCertAddReq {
                            ak: TrimString(mail.trim().to_string()),
                            sk: None,
                            sk_invisible: None,
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
                            ignore_check_sk: false,
                        },
                        funs,
                        &ctx,
                    )
                    .await?
                } else {
                    let id = RbumCertServ::find_id_rbums(
                        &RbumCertFilterReq {
                            status: Some(RbumCertStatusKind::Enabled),
                            rel_rbum_id: Some(ctx.owner.clone()),
                            rel_rbum_kind: Some(RbumCertRelKind::Item),
                            rel_rbum_cert_conf_ids: Some(vec![rel_rbum_cert_conf_id]),
                            ..Default::default()
                        },
                        None,
                        None,
                        funs,
                        &ctx,
                    )
                    .await?
                    .pop()
                    .ok_or_else(|| funs.err().unauthorized("iam_cert_mail_vcode", "activate", "email or verification code error", "401-iam-cert-valid"))?;
                    RbumCertServ::modify_rbum(
                        &id,
                        &mut RbumCertModifyReq {
                            ak: Some(TrimString(mail.trim().to_string())),
                            sk: None,
                            sk_invisible: None,
                            ignore_check_sk: true,
                            ext: None,
                            start_time: None,
                            end_time: None,
                            conn_uri: None,
                            status: None,
                        },
                        funs,
                        &ctx,
                    )
                    .await?;
                    id
                };
                IamSearchClient::async_add_or_modify_account_search(&ctx.owner, Box::new(true), "", funs, &ctx).await?;
                let op_describe = format!("绑定邮箱为{}", mail);
                let _ = IamLogClient::add_ctx_task(LogParamTag::IamAccount, Some(ctx.owner.to_string()), op_describe, Some("BindMailbox".to_string()), &ctx).await;

                return Ok(id);
            }
        }
        Err(funs.err().unauthorized("iam_cert_mail_vcode", "activate", "email or verification code error", "401-iam-cert-valid"))
    }

    async fn check_account_bind_mail(rel_rbum_cert_conf_ids: Vec<String>, rel_rbum_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        // check bind or not
        if RbumCertServ::count_rbums(
            &RbumCertFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                rel_rbum_id: Some(rel_rbum_id.to_owned()),
                rel_rbum_kind: Some(RbumCertRelKind::Item),
                rel_rbum_cert_conf_ids: Some(rel_rbum_cert_conf_ids.clone()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
            > 0
        {
            return Err(funs.err().conflict("iam_cert_mail_vcode", "bind", "email already exist bind", "409-iam-cert-email-bind-already-exist"));
        }
        Ok(())
    }

    async fn check_mail_bound(mail: &str, rel_rbum_cert_conf_ids: Vec<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        // check existence or not
        if RbumCertServ::count_rbums(
            &RbumCertFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ak: Some(mail.to_string()),
                rel_rbum_kind: Some(RbumCertRelKind::Item),
                rel_rbum_cert_conf_ids: Some(rel_rbum_cert_conf_ids),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
            > 0
        {
            return Err(funs.err().unauthorized("iam_cert_mail_vcode", "activate", "email already exist", "404-iam-cert-email-not-exist"));
        }
        Ok(())
    }

    pub async fn send_login_mail(mail: &str, tenant_id: &str, cool_down_id_sec: Option<u32>, funs: &TardisFunsInst) -> TardisResult<()> {
        let own_paths = tenant_id.to_string();
        let mock_ctx = TardisContext {
            own_paths: own_paths.to_string(),
            ..Default::default()
        };
        let global_rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_kind(&IamCertKernelKind::MailVCode.to_string(), None, funs).await?;
        let tenant_rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_kind(&IamCertKernelKind::MailVCode.to_string(), Some(tenant_id.to_owned()), funs).await?;
        if RbumCertServ::count_rbums(
            &RbumCertFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ak: Some(mail.to_string()),
                rel_rbum_kind: Some(RbumCertRelKind::Item),
                rel_rbum_cert_conf_ids: Some(vec![tenant_rbum_cert_conf_id.clone()]),
                ..Default::default()
            },
            funs,
            &mock_ctx,
        )
        .await?
            > 0
        {
            let vcode = Self::get_vcode();
            RbumCertServ::add_vcode_to_cache(mail, &vcode, &tenant_rbum_cert_conf_id, cool_down_id_sec, funs, &mock_ctx).await?;
            MailClient::send_vcode(mail, None, &vcode, funs).await?;
            return Ok(());
        }
        if RbumCertServ::count_rbums(
            &RbumCertFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ak: Some(mail.to_string()),
                rel_rbum_kind: Some(RbumCertRelKind::Item),
                rel_rbum_cert_conf_ids: Some(vec![global_rbum_cert_conf_id.clone()]),
                ..Default::default()
            },
            funs,
            &mock_ctx,
        )
        .await?
            > 0
        {
            let vcode = Self::get_vcode();
            RbumCertServ::add_vcode_to_cache(
                mail,
                &vcode,
                &global_rbum_cert_conf_id,
                cool_down_id_sec,
                funs,
                &TardisContext {
                    own_paths: "".to_string(),
                    ..Default::default()
                },
            )
            .await?;
            MailClient::send_vcode(mail, None, &vcode, funs).await?;
            return Ok(());
        }
        return Err(funs.err().not_found("iam_cert_phone_vcode", "send", "email not find", "404-iam-cert-email-not-exist"));
    }

    fn get_vcode() -> String {
        let mut rand = tardis::rand::thread_rng();
        let vcode: i32 = rand.gen_range(1000..9999);
        format!("{vcode}")
    }

    pub async fn add_or_enable_cert_conf(
        add_req: &IamCertConfMailVCodeAddOrModifyReq,
        rel_iam_item_id: Option<String>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<String> {
        let cert_result = RbumCertConfServ::do_find_one_rbum(
            &RbumCertConfFilterReq {
                kind: Some(TrimString(IamCertKernelKind::MailVCode.to_string())),
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

    pub async fn send_pwd(account_id: &str, pwd: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let resp = IamCertServ::get_kernel_cert(account_id, &IamCertKernelKind::MailVCode, funs, ctx).await;
        match resp {
            Ok(cert) => {
                let _ = MailClient::async_send_pwd(&cert.ak, pwd, funs, ctx).await;
            }
            Err(_) => info!("mail pwd not found"),
        }
        Ok(())
    }
}
