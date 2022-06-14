use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::TardisFunsInst;

use bios_basic::rbum::dto::rbum_cert_conf_dto::{RbumCertConfAddReq, RbumCertConfModifyReq};
use bios_basic::rbum::dto::rbum_cert_dto::RbumCertAddReq;
use bios_basic::rbum::dto::rbum_filer_dto::RbumCertFilterReq;
use bios_basic::rbum::rbum_enumeration::{RbumCertRelKind, RbumCertStatusKind};
use bios_basic::rbum::serv::rbum_cert_serv::{RbumCertConfServ, RbumCertServ};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;

use crate::basic::dto::iam_cert_conf_dto::IamUserPwdCertConfAddOrModifyReq;
use crate::basic::dto::iam_cert_dto::{IamUserPwdCertAddReq, IamUserPwdCertModifyReq, IamUserPwdCertRestReq};
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_key_cache_serv::IamIdentCacheServ;
use crate::iam_config::IamBasicConfigApi;
use crate::iam_enumeration::IamCertKind;

pub struct IamCertUserPwdServ;

impl<'a> IamCertUserPwdServ {
    pub async fn add_cert_conf(
        add_req: &IamUserPwdCertConfAddOrModifyReq,
        rel_iam_item_id: Option<String>,
        funs: &TardisFunsInst<'a>,
        ctx: &TardisContext,
    ) -> TardisResult<String> {
        let id = RbumCertConfServ::add_rbum(
            &mut RbumCertConfAddReq {
                code: TrimString(IamCertKind::UserPwd.to_string()),
                name: TrimString(IamCertKind::UserPwd.to_string()),
                note: None,
                ak_note: add_req.ak_note.clone(),
                ak_rule: add_req.ak_rule.clone(),
                sk_note: add_req.sk_note.clone(),
                sk_rule: add_req.sk_rule.clone(),
                sk_need: Some(true),
                sk_dynamic: None,
                sk_encrypted: Some(true),
                repeatable: add_req.repeatable,
                is_basic: Some(true),
                rest_by_kinds: Some(format!("{},{}", IamCertKind::MailVCode, IamCertKind::PhoneVCode)),
                expire_sec: add_req.expire_sec,
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

    pub async fn modify_cert_conf(id: &str, modify_req: &IamUserPwdCertConfAddOrModifyReq, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<()> {
        RbumCertConfServ::modify_rbum(
            id,
            &mut RbumCertConfModifyReq {
                name: None,
                note: None,
                ak_note: modify_req.ak_note.clone(),
                ak_rule: modify_req.ak_rule.clone(),
                sk_note: modify_req.sk_note.clone(),
                sk_rule: modify_req.sk_rule.clone(),
                sk_need: None,
                sk_encrypted: None,
                repeatable: modify_req.repeatable,
                is_basic: None,
                rest_by_kinds: None,
                expire_sec: modify_req.expire_sec,
                coexist_num: None,
                conn_uri: None,
            },
            funs,
            ctx,
        )
        .await?;
        if modify_req.ak_rule.is_some() || modify_req.sk_rule.is_some() || modify_req.repeatable.is_some() || modify_req.expire_sec.is_some() {
            IamCertServ::clean_cache_by_cert_conf(id, None, funs, ctx).await?;
        }
        Ok(())
    }

    pub async fn add_cert(
        add_req: &IamUserPwdCertAddReq,
        rel_iam_item_id: &str,
        rel_rbum_cert_conf_id: Option<String>,
        funs: &TardisFunsInst<'a>,
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
        funs: &TardisFunsInst<'a>,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let cert = RbumCertServ::find_one_rbum(
            &RbumCertFilterReq {
                rel_rbum_kind: Some(RbumCertRelKind::Item),
                rel_rbum_id: Some(rel_iam_item_id.to_string()),
                rel_rbum_cert_conf_id: Some(rel_rbum_cert_conf_id.to_string()),
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
            Err(funs.err().not_found("cert_user_pwd", "modify", &format!("not found credential of kind {:?}", IamCertKind::UserPwd)))
        }
    }

    pub async fn reset_sk(
        modify_req: &IamUserPwdCertRestReq,
        rel_iam_item_id: &str,
        rel_rbum_cert_conf_id: &str,
        funs: &TardisFunsInst<'a>,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let cert = RbumCertServ::find_one_rbum(
            &RbumCertFilterReq {
                rel_rbum_kind: Some(RbumCertRelKind::Item),
                rel_rbum_id: Some(rel_iam_item_id.to_string()),
                rel_rbum_cert_conf_id: Some(rel_rbum_cert_conf_id.to_string()),
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
            Err(funs.err().not_found("cert_user_pwd", "reset_sk", &format!("not found credential of kind {:?}", IamCertKind::UserPwd)))
        }
    }
}
