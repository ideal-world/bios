use bios_basic::helper::request_helper::get_remote_ip;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::{TardisFuns, TardisFunsInst};

use bios_basic::rbum::dto::rbum_cert_conf_dto::{RbumCertConfAddReq, RbumCertConfModifyReq};
use bios_basic::rbum::dto::rbum_cert_dto::{RbumCertAddReq, RbumCertModifyReq};
use bios_basic::rbum::dto::rbum_filer_dto::{RbumCertConfFilterReq, RbumCertFilterReq};
use bios_basic::rbum::rbum_enumeration::{RbumCertConfStatusKind, RbumCertRelKind, RbumCertStatusKind};
use bios_basic::rbum::serv::rbum_cert_serv::{RbumCertConfServ, RbumCertServ};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;

use crate::basic::dto::iam_cert_conf_dto::IamCertConfUserPwdAddOrModifyReq;
use crate::basic::dto::iam_cert_dto::{IamCertUserNameNewReq, IamCertUserPwdAddReq, IamCertUserPwdModifyReq, IamCertUserPwdRestReq};
use crate::basic::serv::iam_key_cache_serv::IamIdentCacheServ;
use crate::iam_config::IamBasicConfigApi;
use crate::iam_enumeration::IamCertKernelKind;

use super::clients::iam_log_client::{IamLogClient, LogParamTag};
use super::iam_account_serv::IamAccountServ;
use super::iam_cert_mail_vcode_serv::IamCertMailVCodeServ;
use super::iam_cert_phone_vcode_serv::IamCertPhoneVCodeServ;
use super::iam_cert_serv::IamCertServ;

pub struct IamCertUserPwdServ;

impl IamCertUserPwdServ {
    pub async fn add_cert_conf(add_req: &IamCertConfUserPwdAddOrModifyReq, rel_iam_item_id: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let id = RbumCertConfServ::add_rbum(
            &mut RbumCertConfAddReq {
                kind: TrimString(IamCertKernelKind::UserPwd.to_string()),
                supplier: None,
                name: TrimString(IamCertKernelKind::UserPwd.to_string()),
                note: None,
                ak_rule: Some(IamCertUserPwdServ::parse_ak_rule(add_req, funs)?),
                ak_note: None,
                sk_rule: Some(IamCertUserPwdServ::parse_sk_rule(add_req, funs)?),
                sk_note: None,
                ext: Some(TardisFuns::json.obj_to_string(&add_req)?),
                sk_need: Some(true),
                sk_dynamic: None,
                sk_encrypted: Some(true),
                repeatable: Some(add_req.repeatable),
                is_basic: Some(true),
                is_ak_repeatable: None,
                rest_by_kinds: Some(format!("{},{}", IamCertKernelKind::MailVCode, IamCertKernelKind::PhoneVCode)),
                expire_sec: Some(add_req.expire_sec),
                sk_lock_cycle_sec: Some(add_req.sk_lock_cycle_sec),
                sk_lock_err_times: Some(add_req.sk_lock_err_times),
                sk_lock_duration_sec: Some(add_req.sk_lock_duration_sec),
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

    pub async fn modify_cert_conf(id: &str, modify_req: &IamCertConfUserPwdAddOrModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let conf = RbumCertConfServ::get_rbum(id, &RbumCertConfFilterReq { ..Default::default() }, funs, ctx).await?;
        let ext = TardisFuns::json.str_to_obj::<IamCertConfUserPwdAddOrModifyReq>(conf.ext.as_str())?;
        let mut log_tasks = vec![];
        if modify_req.sk_rule_len_max != ext.sk_rule_len_max || modify_req.sk_rule_len_min != ext.sk_rule_len_min {
            log_tasks.push((
                format!("修改密码长度{}-{}", modify_req.sk_rule_len_min, modify_req.sk_rule_len_max),
                "ModifyPasswordLength".to_string(),
            ));
        }
        if modify_req.sk_rule_need_num != ext.sk_rule_need_num
            || modify_req.sk_rule_need_uppercase != ext.sk_rule_need_uppercase
            || modify_req.sk_rule_need_lowercase != ext.sk_rule_need_lowercase
            || modify_req.sk_rule_need_spec_char != ext.sk_rule_need_spec_char
        {
            log_tasks.push(("修改密码复杂度".to_string(), "ModifyPasswordComplexity".to_string()));
        }
        if modify_req.expire_sec != ext.expire_sec {
            log_tasks.push((format!("修改密码有效期为{}", modify_req.expire_sec / 86400), "ModifyPasswordValidityPeriod".to_string()));
        }
        if modify_req.sk_lock_cycle_sec != ext.sk_lock_cycle_sec
            || modify_req.sk_lock_err_times != ext.sk_lock_err_times
            || modify_req.sk_lock_duration_sec != ext.sk_lock_duration_sec
        {
            log_tasks.push((
                format!(
                    "修改密码错误设定为{}分钟内连续错误{}次即被锁定{}分钟",
                    modify_req.sk_lock_cycle_sec / 60,
                    modify_req.sk_lock_err_times,
                    modify_req.sk_lock_duration_sec / 60
                ),
                "ModifyPasswordErrorSetting".to_string(),
            ));
        }
        for (op_describe, op_kind) in log_tasks {
            let _ = IamLogClient::add_ctx_task(LogParamTag::SecurityAlarm, None, op_describe, Some(op_kind), ctx).await;
        }

        RbumCertConfServ::modify_rbum(
            id,
            &mut RbumCertConfModifyReq {
                name: None,
                note: None,
                ak_rule: Some(IamCertUserPwdServ::parse_ak_rule(modify_req, funs)?),
                ak_note: None,
                sk_rule: Some(IamCertUserPwdServ::parse_sk_rule(modify_req, funs)?),
                sk_note: None,
                ext: Some(TardisFuns::json.obj_to_string(modify_req)?),
                sk_need: None,
                sk_encrypted: None,
                repeatable: Some(modify_req.repeatable),
                is_basic: None,
                rest_by_kinds: None,
                expire_sec: Some(modify_req.expire_sec),
                sk_lock_cycle_sec: Some(modify_req.sk_lock_cycle_sec),
                sk_lock_err_times: Some(modify_req.sk_lock_err_times),
                sk_lock_duration_sec: Some(modify_req.sk_lock_duration_sec),
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

    pub async fn add_cert(
        add_req: &IamCertUserPwdAddReq,
        rel_iam_item_id: &str,
        rel_rbum_cert_conf_id: Option<String>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let status = if let Some(add_req_status) = &add_req.status {
            add_req_status.clone()
        } else {
            RbumCertStatusKind::Pending
        };
        IamCertUserPwdServ::check_sk_contains_ak(&add_req.ak, &add_req.sk, funs)?;
        RbumCertServ::add_rbum(
            &mut RbumCertAddReq {
                ak: add_req.ak.clone(),
                sk: Some(add_req.sk.clone()),
                is_ignore_check_sk: add_req.is_ignore_check_sk,
                kind: None,
                supplier: None,
                vcode: None,
                ext: None,
                start_time: None,
                end_time: None,
                conn_uri: None,
                status,
                rel_rbum_cert_conf_id,
                rel_rbum_kind: RbumCertRelKind::Item,
                rel_rbum_id: rel_iam_item_id.to_string(),
                is_outside: false,
            },
            funs,
            ctx,
        )
        .await?;
        Ok(())
    }

    pub async fn modify_cert(
        modify_req: &IamCertUserPwdModifyReq,
        rel_iam_item_id: &str,
        rel_rbum_cert_conf_id: &str,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let cert = RbumCertServ::find_one_rbum(
            &RbumCertFilterReq {
                rel_rbum_kind: Some(RbumCertRelKind::Item),
                rel_rbum_id: Some(rel_iam_item_id.to_string()),
                rel_rbum_cert_conf_ids: Some(vec![rel_rbum_cert_conf_id.to_string()]),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        if let Some(cert) = cert {
            IamCertUserPwdServ::check_sk_contains_ak(&cert.ak, &modify_req.new_sk, funs)?;
            RbumCertServ::change_sk(&cert.id, &modify_req.original_sk.0, &modify_req.new_sk.0, &RbumCertFilterReq::default(), funs, ctx).await?;
            IamCertPhoneVCodeServ::send_pwd(rel_iam_item_id, &modify_req.new_sk.0, funs, ctx).await?;
            IamCertMailVCodeServ::send_pwd(rel_iam_item_id, &modify_req.new_sk.0, funs, ctx).await?;
            RbumCertServ::modify_rbum(
                &cert.id,
                &mut RbumCertModifyReq {
                    ak: None,
                    sk: None,
                    is_ignore_check_sk: false,
                    ext: None,
                    start_time: None,
                    end_time: None,
                    conn_uri: None,
                    status: RbumCertStatusKind::Enabled.into(),
                },
                funs,
                ctx,
            )
            .await?;

            let _ = IamLogClient::add_ctx_task(
                LogParamTag::IamAccount,
                Some(ctx.owner.clone()),
                "修改密码".to_string(),
                Some("ModifyPassword".to_string()),
                ctx,
            )
            .await;
            IamIdentCacheServ::delete_tokens_and_contexts_by_account_id(rel_iam_item_id, get_remote_ip(&ctx).await?, funs).await
        } else {
            Err(funs.err().not_found(
                "iam_cert_user_pwd",
                "modify",
                &format!("not found credential of kind {:?}", IamCertKernelKind::UserPwd),
                "404-iam-cert-kind-not-exist",
            ))
        }
    }

    pub async fn modify_ak_cert(account_id: &str, modify_req: &IamCertUserNameNewReq, rel_rbum_cert_conf_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let cert = RbumCertServ::find_one_rbum(
            &RbumCertFilterReq {
                ak: Some(modify_req.original_ak.to_string()),
                rel_rbum_kind: Some(RbumCertRelKind::Item),
                rel_rbum_id: Some(account_id.to_string()),
                rel_rbum_cert_conf_ids: Some(vec![rel_rbum_cert_conf_id.to_string()]),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        if let Some(cert) = cert {
            let cert_resp = RbumCertServ::find_one_rbum(
                &RbumCertFilterReq {
                    ak: Some(modify_req.new_ak.to_string()),
                    rel_rbum_kind: Some(RbumCertRelKind::Item),
                    rel_rbum_cert_conf_ids: Some(vec![rel_rbum_cert_conf_id.to_string()]),
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?;
            if cert_resp.is_some() {
                return Err(funs.err().conflict("ak_cert", "modify", "ak is used", "409-rbum-cert-ak-duplicate"));
            }
            IamCertUserPwdServ::check_sk_contains_ak(&modify_req.new_ak, &modify_req.sk, funs)?;
            RbumCertServ::modify_rbum(
                &cert.id,
                &mut RbumCertModifyReq {
                    ak: Some(modify_req.new_ak.clone()),
                    sk: Some(modify_req.sk.clone()),
                    is_ignore_check_sk: true,
                    ext: None,
                    start_time: None,
                    end_time: None,
                    conn_uri: None,
                    status: None,
                },
                funs,
                ctx,
            )
            .await?;
            IamAccountServ::async_add_or_modify_account_search(cert.rel_rbum_id, Box::new(true), "".to_string(), funs, ctx).await?;
            Ok(())
        } else {
            Err(funs.err().not_found(
                "ak_cert",
                "modify",
                &format!("not found credential of kind {:?}", IamCertKernelKind::UserPwd),
                "404-iam-cert-kind-not-exist",
            ))
        }
    }

    //todo 统一reset_sk_for_pending_status方法
    //可进行重置后的状态：Pending、Enabled、Disabled
    pub async fn reset_sk(modify_req: &IamCertUserPwdRestReq, rel_iam_item_id: &str, rel_rbum_cert_conf_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let new_sk = if let Some(new_sk) = &modify_req.new_sk {
            new_sk.to_string()
        } else {
            IamCertServ::get_new_pwd()
        };
        let cert = RbumCertServ::find_one_rbum(
            &RbumCertFilterReq {
                rel_rbum_kind: Some(RbumCertRelKind::Item),
                rel_rbum_id: Some(rel_iam_item_id.to_string()),
                rel_rbum_cert_conf_ids: Some(vec![rel_rbum_cert_conf_id.to_string()]),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        if let Some(cert) = cert {
            IamCertUserPwdServ::check_sk_contains_ak(&cert.ak, &new_sk, funs)?;
            RbumCertServ::reset_sk(&cert.id, &new_sk, true, &RbumCertFilterReq::default(), funs, ctx).await?;
            IamCertPhoneVCodeServ::send_pwd(rel_iam_item_id, &new_sk, funs, ctx).await?;
            IamCertMailVCodeServ::send_pwd(rel_iam_item_id, &new_sk, funs, ctx).await?;
            RbumCertServ::modify_rbum(
                &cert.id,
                &mut RbumCertModifyReq {
                    ak: None,
                    sk: None,
                    is_ignore_check_sk: false,
                    ext: None,
                    start_time: None,
                    end_time: None,
                    conn_uri: None,
                    status: RbumCertStatusKind::Pending.into(),
                },
                funs,
                ctx,
            )
            .await?;
            let result = IamIdentCacheServ::delete_tokens_and_contexts_by_account_id(rel_iam_item_id, get_remote_ip(&ctx).await?, funs).await;

            let _ = IamLogClient::add_ctx_task(
                LogParamTag::IamAccount,
                Some(rel_iam_item_id.to_string()),
                "重置账号密码".to_string(),
                Some("ResetAccountPassword".to_string()),
                ctx,
            )
            .await;

            result
        } else {
            Err(funs.err().not_found(
                "iam_cert_user_pwd",
                "reset_sk",
                &format!("not found credential of kind {:?}", IamCertKernelKind::UserPwd),
                "404-iam-cert-kind-not-exist",
            ))
        }
    }

    pub async fn reset_sk_to_enable_status(
        modify_req: &IamCertUserPwdRestReq,
        rel_iam_item_id: &str,
        rel_rbum_cert_conf_id: &str,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let new_sk = if let Some(new_sk) = &modify_req.new_sk {
            new_sk.to_string()
        } else {
            IamCertServ::get_new_pwd()
        };
        let cert = RbumCertServ::find_one_rbum(
            &RbumCertFilterReq {
                rel_rbum_kind: Some(RbumCertRelKind::Item),
                rel_rbum_id: Some(rel_iam_item_id.to_string()),
                rel_rbum_cert_conf_ids: Some(vec![rel_rbum_cert_conf_id.to_string()]),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        if let Some(cert) = cert {
            IamCertUserPwdServ::check_sk_contains_ak(&cert.ak, &new_sk, funs)?;
            RbumCertServ::reset_sk(&cert.id, &new_sk, true, &RbumCertFilterReq::default(), funs, ctx).await?;
            // IamCertPhoneVCodeServ::send_pwd(rel_iam_item_id, &new_sk, funs, ctx).await?;
            // IamCertMailVCodeServ::send_pwd(rel_iam_item_id, &new_sk, funs, ctx).await?;
            RbumCertServ::modify_rbum(
                &cert.id,
                &mut RbumCertModifyReq {
                    ak: None,
                    sk: None,
                    is_ignore_check_sk: false,
                    ext: None,
                    start_time: None,
                    end_time: None,
                    conn_uri: None,
                    status: RbumCertStatusKind::Enabled.into(),
                },
                funs,
                ctx,
            )
            .await?;

            let _ = IamLogClient::add_ctx_task(
                LogParamTag::IamAccount,
                Some(rel_iam_item_id.to_string()),
                "重置账号密码".to_string(),
                Some("ResetAccountPassword".to_string()),
                ctx,
            )
            .await;
        } else {
            return Err(funs.err().not_found(
                "iam_cert_user_pwd",
                "reset_sk",
                &format!("not found credential of kind {:?}", IamCertKernelKind::UserPwd),
                "404-iam-cert-kind-not-exist",
            ));
        }
        Ok(())
    }

    //todo 限定conf
    pub async fn rename_ak_if_duplicate(ak: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<TrimString> {
        // \_ sql 转义-> _
        let mut count_duplicate_ak = RbumCertServ::count_rbums(
            &RbumCertFilterReq {
                ak_like: Some(TrimString(format!(r"{ak}\_",)).to_string()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        count_duplicate_ak += RbumCertServ::count_rbums(
            &RbumCertFilterReq {
                ak: Some(ak.to_string()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        if count_duplicate_ak > 0 {
            let string = RbumCertServ::find_rbums(
                &RbumCertFilterReq {
                    ak_like: Some(TrimString(format!(r"{ak}\_",)).to_string()),
                    ..Default::default()
                },
                None,
                Some(true),
                funs,
                ctx,
            )
            .await?
            .first()
            .map(|r| r.ak.clone())
            .unwrap_or_else(|| "".to_string());
            let vec_str: Vec<&str> = string.split('_').collect();
            if vec_str.len() != 2 {
                Ok(format!("{ak}_{count_duplicate_ak}").into())
            } else {
                let parse_u32 = vec_str[vec_str.len() - 1].parse::<u32>();
                if let Ok(count) = parse_u32 {
                    Ok(format!("{}_{}", ak, count + 1).into())
                } else {
                    Ok(format!("{}_{}", ak, count_duplicate_ak + 1).into())
                }
            }
        } else {
            Ok(ak.into())
        }
    }

    fn parse_ak_rule(cert_conf_by_user_pwd: &IamCertConfUserPwdAddOrModifyReq, funs: &TardisFunsInst) -> TardisResult<String> {
        if cert_conf_by_user_pwd.ak_rule_len_max < cert_conf_by_user_pwd.ak_rule_len_min {
            return Err(funs.err().bad_request(
                "iam_cert_conf",
                "add_tenant",
                "incorrect [ak_rule_len_min] or [ak_rule_len_max]",
                "400-iam-cert-ak-len-incorrect",
            ));
        }
        Ok(format!(
            "^[0-9a-zA-Z-_@:\\.]{{{},{}}}$",
            cert_conf_by_user_pwd.ak_rule_len_min, cert_conf_by_user_pwd.ak_rule_len_max
        ))
    }

    fn parse_sk_rule(cert_conf_by_user_pwd: &IamCertConfUserPwdAddOrModifyReq, funs: &TardisFunsInst) -> TardisResult<String> {
        if cert_conf_by_user_pwd.sk_rule_len_max < cert_conf_by_user_pwd.sk_rule_len_min {
            return Err(funs.err().bad_request(
                "iam_cert_conf",
                "add_tenant",
                "incorrect [sk_rule_len_min] or [sk_rule_len_max]",
                "400-iam-cert-sk-len-incorrect",
            ));
        }
        Ok(format!(
            "^{}{}{}{}.{{{},{}}}$",
            if cert_conf_by_user_pwd.sk_rule_need_num { "(?=.*\\d)" } else { "" },
            if cert_conf_by_user_pwd.sk_rule_need_lowercase { "(?=.*[a-z])" } else { "" },
            if cert_conf_by_user_pwd.sk_rule_need_uppercase { "(?=.*[A-Z])" } else { "" },
            if cert_conf_by_user_pwd.sk_rule_need_spec_char { "(?=.*[$@!%*#?&.\\-_^])" } else { "" },
            cert_conf_by_user_pwd.sk_rule_len_min,
            cert_conf_by_user_pwd.sk_rule_len_max
        ))
    }

    // 不在rbum_cert_serve在做检查是因为其他凭证sk不需要去检查是否包含ak, 但是在这里做检查是因为用户密码凭证需要检查
    fn check_sk_contains_ak(ak: &str, sk: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        if sk.to_lowercase().contains(&ak.to_lowercase()) {
            return Err(funs.err().bad_request("iam_cert", "check ak sk", "sk can not contains ak", "400-iam-cert-sk-contains-ak"));
        }
        Ok(())
    }
}
