use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::{TardisFuns, TardisFunsInst};

use bios_basic::rbum::dto::rbum_cert_conf_dto::{RbumCertConfAddReq, RbumCertConfModifyReq};
use bios_basic::rbum::dto::rbum_cert_dto::RbumCertAddReq;
use bios_basic::rbum::dto::rbum_filer_dto::RbumCertFilterReq;
use bios_basic::rbum::rbum_enumeration::{RbumCertRelKind, RbumCertStatusKind};
use bios_basic::rbum::serv::rbum_cert_serv::{RbumCertConfServ, RbumCertServ};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;

use crate::basic::dto::iam_cert_conf_dto::IamUserPwdCertConfAddOrModifyReq;
use crate::basic::dto::iam_cert_dto::{IamUserPwdCertAddReq, IamUserPwdCertModifyReq, IamUserPwdCertRestReq};
use crate::basic::serv::iam_key_cache_serv::IamIdentCacheServ;
use crate::iam_config::IamBasicConfigApi;
use crate::iam_enumeration::IamCertKernelKind;

pub struct IamCertUserPwdServ;

impl IamCertUserPwdServ {
    pub async fn add_cert_conf(add_req: &IamUserPwdCertConfAddOrModifyReq, rel_iam_item_id: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let id = RbumCertConfServ::add_rbum(
            &mut RbumCertConfAddReq {
                code: TrimString(IamCertKernelKind::UserPwd.to_string()),
                name: TrimString(IamCertKernelKind::UserPwd.to_string()),
                note: None,
                ak_rule: Some(IamCertUserPwdServ::parse_ak_rule(&add_req, funs)?),
                ak_note: None,
                sk_rule: Some(IamCertUserPwdServ::parse_sk_rule(&add_req, funs)?),
                sk_note: None,
                ext: Some(TardisFuns::json.obj_to_string(&add_req)?),
                sk_need: Some(true),
                sk_dynamic: None,
                sk_encrypted: Some(true),
                repeatable: Some(add_req.repeatable),
                is_basic: Some(true),
                rest_by_kinds: Some(format!("{},{}", IamCertKernelKind::MailVCode, IamCertKernelKind::PhoneVCode)),
                expire_sec: Some(add_req.expire_sec),
                sk_lock_cycle_sec: Some(add_req.sk_lock_cycle_sec),
                sk_lock_err_times: Some(add_req.sk_lock_err_times),
                sk_lock_duration_sec: Some(add_req.sk_lock_duration_sec),
                coexist_num: Some(1),
                conn_uri: None,
                rel_rbum_domain_id: funs.iam_basic_domain_iam_id(),
                rel_rbum_item_id: rel_iam_item_id,
            },
            funs,
            ctx,
        )
        .await?;
        Ok(id)
    }

    pub async fn modify_cert_conf(id: &str, modify_req: &IamUserPwdCertConfAddOrModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
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
            },
            funs,
            ctx,
        )
        .await?;
        Ok(())
    }

    pub async fn add_cert(
        add_req: &IamUserPwdCertAddReq,
        rel_iam_item_id: &str,
        rel_rbum_cert_conf_id: Option<String>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        RbumCertServ::add_rbum(
            &mut RbumCertAddReq {
                ak: add_req.ak.clone(),
                sk: Some(add_req.sk.clone()),
                vcode: None,
                ext: None,
                start_time: None,
                end_time: None,
                conn_uri: None,
                status: RbumCertStatusKind::Enabled,
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
        modify_req: &IamUserPwdCertModifyReq,
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
            RbumCertServ::change_sk(&cert.id, &modify_req.original_sk.0, &modify_req.new_sk.0, &RbumCertFilterReq::default(), funs, ctx).await?;
            IamIdentCacheServ::delete_tokens_and_contexts_by_account_id(rel_iam_item_id, funs).await
        } else {
            Err(funs.err().not_found(
                "iam_cert_user_pwd",
                "modify",
                &format!("not found credential of kind {:?}", IamCertKernelKind::UserPwd),
                "404-iam-cert-kind-not-exist",
            ))
        }
    }

    pub async fn reset_sk(modify_req: &IamUserPwdCertRestReq, rel_iam_item_id: &str, rel_rbum_cert_conf_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
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
            RbumCertServ::reset_sk(&cert.id, &modify_req.new_sk.0, &RbumCertFilterReq::default(), funs, ctx).await?;
            IamIdentCacheServ::delete_tokens_and_contexts_by_account_id(rel_iam_item_id, funs).await
        } else {
            Err(funs.err().not_found(
                "iam_cert_user_pwd",
                "reset_sk",
                &format!("not found credential of kind {:?}", IamCertKernelKind::UserPwd),
                "404-iam-cert-kind-not-exist",
            ))
        }
    }

    fn parse_ak_rule(cert_conf_by_user_pwd: &IamUserPwdCertConfAddOrModifyReq, funs: &TardisFunsInst) -> TardisResult<String> {
        if cert_conf_by_user_pwd.ak_rule_len_max < cert_conf_by_user_pwd.ak_rule_len_min {
            return Err(funs.err().bad_request(
                "iam_cert_conf",
                "add_tenant",
                "incorrect [ak_rule_len_min] or [ak_rule_len_max]",
                "400-iam-cert-ak-len-incorrect",
            ));
        }
        Ok(format!(
            "^[0-9a-z-_@\\.]{{{},{}}}$",
            cert_conf_by_user_pwd.ak_rule_len_min, cert_conf_by_user_pwd.ak_rule_len_max
        ))
    }

    fn parse_sk_rule(cert_conf_by_user_pwd: &IamUserPwdCertConfAddOrModifyReq, funs: &TardisFunsInst) -> TardisResult<String> {
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
            if cert_conf_by_user_pwd.sk_rule_need_spec_char { "(?=.*[$@!%*#?&])" } else { "" },
            cert_conf_by_user_pwd.sk_rule_len_min,
            cert_conf_by_user_pwd.sk_rule_len_max
        ))
    }
}
