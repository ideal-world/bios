use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::error::TardisError;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;

use bios_basic::rbum::dto::rbum_cert_conf_dto::{RbumCertConfAddReq, RbumCertConfModifyReq};
use bios_basic::rbum::dto::rbum_cert_dto::RbumCertAddReq;
use bios_basic::rbum::dto::rbum_filer_dto::RbumCertFilterReq;
use bios_basic::rbum::rbum_enumeration::{RbumCertRelKind, RbumCertStatusKind};
use bios_basic::rbum::serv::rbum_cert_serv::{RbumCertConfServ, RbumCertServ};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;

use crate::basic::dto::iam_cert_conf_dto::IamUserPwdCertConfAddOrModifyReq;
use crate::basic::dto::iam_cert_dto::{IamUserPwdCertAddReq, IamUserPwdCertModifyReq};
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::iam_config::IamBasicInfoManager;
use crate::iam_enumeration::IamCertKind;

pub struct IamCertUserPwdServ;

impl<'a> IamCertUserPwdServ {
    pub async fn add_cert_conf(
        add_req: &mut IamUserPwdCertConfAddOrModifyReq,
        rel_iam_tenant_id: Option<String>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
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
                sk_encrypted: Some(true),
                repeatable: add_req.repeatable.clone(),
                is_basic: Some(true),
                rest_by_kinds: Some(format!("{},{}", IamCertKind::MailVCode.to_string(), IamCertKind::PhoneVCode.to_string())),
                expire_sec: add_req.expire_sec,
                coexist_num: Some(1),
                conn_uri: None,
                rel_rbum_domain_id: IamBasicInfoManager::get().domain_iam_id.to_string(),
                rel_rbum_item_id: rel_iam_tenant_id,
            },
            funs,
            cxt,
        )
        .await?;
        Ok(id)
    }

    pub async fn modify_cert_conf(id: &str, modify_req: &mut IamUserPwdCertConfAddOrModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
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
                repeatable: modify_req.repeatable.clone(),
                is_basic: None,
                rest_by_kinds: None,
                expire_sec: modify_req.expire_sec,
                coexist_num: None,
                conn_uri: None,
            },
            funs,
            cxt,
        )
        .await?;
        Ok(())
    }

    pub async fn add_cert(
        add_req: &mut IamUserPwdCertAddReq,
        iam_item_id: &str,
        rel_iam_tenant_id: Option<&str>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<()> {
        RbumCertServ::add_rbum(
            &mut RbumCertAddReq {
                ak: add_req.ak.clone(),
                sk: Some(add_req.sk.clone()),
                ext: None,
                start_time: None,
                end_time: None,
                conn_uri: None,
                status: RbumCertStatusKind::Enabled,
                rel_rbum_cert_conf_id: Some(IamCertServ::get_id_by_code(IamCertKind::UserPwd.to_string().as_str(), rel_iam_tenant_id, funs).await?),
                rel_rbum_kind: RbumCertRelKind::Item,
                rel_rbum_id: iam_item_id.to_string(),
            },
            funs,
            cxt,
        )
        .await?;
        Ok(())
    }

    pub async fn modify_cert(
        modify_req: &mut IamUserPwdCertModifyReq,
        iam_item_id: &str,
        rel_iam_tenant_id: &str,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<()> {
        let certs = RbumCertServ::find_rbums(
            &RbumCertFilterReq {
                rel_rbum_kind: Some(RbumCertRelKind::Item),
                rel_rbum_id: Some(iam_item_id.to_string()),
                rel_rbum_cert_conf_id: Some(IamCertServ::get_id_by_code(IamCertKind::UserPwd.to_string().as_str(), Some(rel_iam_tenant_id), funs).await?),
                ..Default::default()
            },
            None,
            None,
            funs,
            cxt,
        )
        .await?;
        if certs.len() > 1 {
            return Err(TardisError::NotFound(format!("there are multiple credentials of kind {}", IamCertKind::UserPwd)));
        }
        if let Some(cert) = certs.get(0) {
            RbumCertServ::change_sk(&cert.id, &modify_req.original_sk.0, &modify_req.new_sk.0, &RbumCertFilterReq::default(), funs, cxt).await
        } else {
            Err(TardisError::NotFound(format!("cannot find credential of kind {}", IamCertKind::UserPwd)))
        }
    }
}
