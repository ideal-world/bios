use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumCertConfFilterReq, RbumCertFilterReq};

use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;

use tardis::rand::Rng;
use tardis::TardisFunsInst;

use bios_basic::rbum::dto::rbum_cert_conf_dto::{RbumCertConfAddReq, RbumCertConfModifyReq};
use bios_basic::rbum::dto::rbum_cert_dto::{RbumCertAddReq, RbumCertModifyReq};
use bios_basic::rbum::rbum_enumeration::{RbumCertConfStatusKind, RbumCertRelKind, RbumCertStatusKind};
use bios_basic::rbum::serv::rbum_cert_serv::{RbumCertConfServ, RbumCertServ};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;

use crate::basic::dto::iam_cert_conf_dto::IamCertConfPhoneVCodeAddOrModifyReq;
use crate::basic::dto::iam_cert_dto::{IamCertPhoneVCodeAddReq, IamCertPhoneVCodeModifyReq};
use crate::basic::dto::iam_filer_dto::IamAccountFilterReq;
use crate::iam_config::IamBasicConfigApi;
use crate::iam_enumeration::IamCertKernelKind;

use super::clients::iam_log_client::{IamLogClient, LogParamTag};
use super::clients::sms_client::SmsClient;
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
                sk_dynamic: Some(true),
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
                is_ignore_check_sk: false,
            },
            funs,
            ctx,
        )
        .await?;
        // TODO send vcode
        Self::send_activation_phone(account_id, &add_req.phone, &vcode, funs, ctx).await?;
        Ok(id)
    }

    pub async fn modify_cert(id: &str, modify_req: &IamCertPhoneVCodeModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        RbumCertServ::modify_rbum(
            id,
            &mut RbumCertModifyReq {
                ak: Some(TrimString(modify_req.phone.to_string())),
                sk: None,
                ext: None,
                start_time: None,
                end_time: None,
                conn_uri: None,
                status: None,
                is_ignore_check_sk: false,
            },
            funs,
            ctx,
        )
        .await?;
        Ok(())
    }

    pub async fn add_or_modify_cert(phone: &str, account_id: &str, rel_rbum_cert_conf_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let resp = IamCertServ::get_kernel_cert(account_id, &IamCertKernelKind::PhoneVCode, funs, ctx).await;
        match resp {
            Ok(cert) => {
                Self::modify_cert(
                    &cert.id,
                    &IamCertPhoneVCodeModifyReq {
                        phone: TrimString(phone.to_string()),
                    },
                    funs,
                    ctx,
                )
                .await?;
            }
            Err(_) => {
                Self::add_cert(
                    &IamCertPhoneVCodeAddReq {
                        phone: TrimString(phone.to_string()),
                    },
                    account_id,
                    rel_rbum_cert_conf_id,
                    funs,
                    ctx,
                )
                .await?;
            }
        }
        Ok(())
    }

    ///不需要验证直接添加cert
    pub async fn add_cert_skip_vcode(
        add_req: &IamCertPhoneVCodeAddReq,
        account_id: &str,
        rel_rbum_cert_conf_id: &str,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<String> {
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
                status: RbumCertStatusKind::Enabled,
                rel_rbum_cert_conf_id: Some(rel_rbum_cert_conf_id.to_string()),
                rel_rbum_kind: RbumCertRelKind::Item,
                rel_rbum_id: account_id.to_string(),
                is_outside: false,
                is_ignore_check_sk: false,
            },
            funs,
            ctx,
        )
        .await?;
        Ok(id)
    }

    pub async fn resend_activation_phone(account_id: &str, phone: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let vcode = Self::get_vcode();
        RbumCertServ::add_vcode_to_cache(phone, &vcode, &ctx.own_paths, funs).await?;
        Self::send_activation_phone(account_id, phone, &vcode, funs, ctx).await
    }

    async fn send_activation_phone(account_id: &str, phone: &str, vcode: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let _account_name = IamAccountServ::peek_item(account_id, &IamAccountFilterReq::default(), funs, ctx).await?.name;
        // TODO send activation
        SmsClient::send_vcode(phone, vcode, funs, ctx).await?;
        Ok(())
    }

    pub async fn activate_phone(phone: &str, input_vcode: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let ctx = IamAccountServ::new_context_if_account_is_global(ctx, funs).await?;
        if let Some(cached_vcode) = RbumCertServ::get_vcode_in_cache(phone, &ctx.own_paths, funs).await? {
            if cached_vcode == input_vcode {
                let cert = RbumCertServ::find_one_rbum(
                    &RbumCertFilterReq {
                        ak: Some(phone.to_string()),
                        status: Some(RbumCertStatusKind::Pending),
                        rel_rbum_kind: Some(RbumCertRelKind::Item),
                        rel_rbum_cert_conf_ids: Some(vec![
                            IamCertServ::get_cert_conf_id_by_kind(IamCertKernelKind::PhoneVCode.to_string().as_str(), Some(IamTenantServ::get_id_by_ctx(&ctx, funs)?), funs)
                                .await?,
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
                            is_ignore_check_sk: false,
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
        let ctx = IamAccountServ::new_context_if_account_is_global(ctx, funs).await?;
        let rel_rbum_cert_conf_id =
            IamCertServ::get_cert_conf_id_by_kind(IamCertKernelKind::PhoneVCode.to_string().as_str(), Some(IamTenantServ::get_id_by_ctx(&ctx, funs)?), funs).await?;
        Self::check_bind_phone(phone, vec![rel_rbum_cert_conf_id], &ctx.owner.clone(), funs, &ctx).await?;
        let vcode = Self::get_vcode();
        RbumCertServ::add_vcode_to_cache(phone, &vcode, &ctx.own_paths, funs).await?;
        SmsClient::send_vcode(phone, &vcode, funs, &ctx).await
    }

    pub async fn bind_phone(phone: &str, input_vcode: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let ctx = IamAccountServ::new_context_if_account_is_global(ctx, funs).await?;
        if let Some(cached_vcode) = RbumCertServ::get_vcode_in_cache(phone, &ctx.own_paths, funs).await? {
            if cached_vcode == input_vcode {
                let rel_rbum_cert_conf_id =
                    IamCertServ::get_cert_conf_id_by_kind(IamCertKernelKind::PhoneVCode.to_string().as_str(), Some(IamTenantServ::get_id_by_ctx(&ctx, funs)?), funs).await?;
                Self::check_bind_phone(phone, vec![rel_rbum_cert_conf_id.clone()], &ctx.owner.clone(), funs, &ctx).await?;
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
                        is_ignore_check_sk: false,
                    },
                    funs,
                    &ctx,
                )
                .await?;
                let op_describe = format!("绑定手机号为{}", phone);
                let _ = IamLogClient::add_ctx_task(LogParamTag::IamAccount, Some(ctx.owner.to_string()), op_describe, Some("BindPhone".to_string()), &ctx).await;
                return Ok(id);
            }
        }
        Err(funs.err().unauthorized("iam_cert_phone_vcode", "bind", "phone or verification code error", "401-iam-cert-valid"))
    }

    pub async fn check_bind_phone(phone: &str, rel_rbum_cert_conf_ids: Vec<String>, rel_rbum_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
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
            return Err(funs.err().conflict("iam_cert_phone_vcode", "bind", "phone already exist bind", "409-iam-cert-phone-bind-already-exist"));
        }
        // check existence or not
        if RbumCertServ::count_rbums(
            &RbumCertFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ak: Some(phone.to_string()),
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
            return Err(funs.err().unauthorized("iam_cert_phone_vcode", "activate", "phone already exist", "404-iam-cert-phone-not-exist"));
        }
        Ok(())
    }

    pub async fn send_login_phone(phone: &str, tenant_id: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        let own_paths = tenant_id.to_string();
        let mock_ctx = TardisContext {
            own_paths: own_paths.to_string(),
            ..Default::default()
        };
        let global_rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_kind(&IamCertKernelKind::PhoneVCode.to_string(), None, funs).await?;
        let tenant_rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_kind(&IamCertKernelKind::PhoneVCode.to_string(), Some(tenant_id.to_owned()), funs).await?;
        if RbumCertServ::count_rbums(
            &RbumCertFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ak: Some(phone.to_string()),
                rel_rbum_kind: Some(RbumCertRelKind::Item),
                rel_rbum_cert_conf_ids: Some(vec![tenant_rbum_cert_conf_id]),
                ..Default::default()
            },
            funs,
            &mock_ctx,
        )
        .await?
            > 0
        {
            let vcode = Self::get_vcode();
            RbumCertServ::add_vcode_to_cache(phone, &vcode, &own_paths, funs).await?;
            return SmsClient::send_vcode(phone, &vcode, funs, &mock_ctx).await;
        }

        if RbumCertServ::count_rbums(
            &RbumCertFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ak: Some(phone.to_string()),
                rel_rbum_kind: Some(RbumCertRelKind::Item),
                rel_rbum_cert_conf_ids: Some(vec![global_rbum_cert_conf_id]),
                ..Default::default()
            },
            funs,
            &mock_ctx,
        )
        .await?
            > 0
        {
            let vcode = Self::get_vcode();
            RbumCertServ::add_vcode_to_cache(phone, &vcode, "", funs).await?;
            return SmsClient::send_vcode(phone, &vcode, funs, &mock_ctx).await;
        }
        return Err(funs.err().not_found("iam_cert_phone_vcode", "send", "phone not find", "404-iam-cert-phone-not-exist"));
    }

    fn get_vcode() -> String {
        let mut rand = tardis::rand::thread_rng();
        let vcode: i32 = rand.gen_range(100000..999999);
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

    pub async fn send_pwd(account_id: &str, pwd: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let resp = IamCertServ::get_kernel_cert(account_id, &IamCertKernelKind::PhoneVCode, funs, ctx).await;
        match resp {
            Ok(cert) => {
                let _ = SmsClient::send_pwd(&cert.ak, pwd, funs, ctx).await;
            }
            Err(_) => info!("phone pwd not found"),
        }
        Ok(())
    }
}
