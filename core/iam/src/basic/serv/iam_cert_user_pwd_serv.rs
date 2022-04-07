use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisRelDBlConnection;

use bios_basic::rbum::dto::filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::dto::rbum_cert_conf_dto::{RbumCertConfAddReq, RbumCertConfModifyReq};
use bios_basic::rbum::dto::rbum_cert_dto::RbumCertAddReq;
use bios_basic::rbum::rbum_enumeration::RbumCertStatusKind;
use bios_basic::rbum::serv::rbum_cert_serv::{RbumCertConfServ, RbumCertServ};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;

use crate::basic::constants;
use crate::basic::dto::iam_cert_conf_dto::IamUserPwdCertConfAddOrModifyReq;
use crate::basic::dto::iam_cert_dto::{IamUserPwdCertAddReq, IamUserPwdCertModifyReq};
use crate::basic::enumeration::IamCertKind;
use crate::basic::serv::iam_cert_serv::IamCertServ;

pub struct IamCertUserPwdServ;

impl<'a> IamCertUserPwdServ {
    pub async fn add_cert_conf(
        add_req: &mut IamUserPwdCertConfAddOrModifyReq,
        rel_iam_tenant_id: Option<String>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<()> {
        RbumCertConfServ::add_rbum(
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
                rel_rbum_domain_id: constants::get_rbum_basic_info().domain_iam_id.to_string(),
                rel_rbum_item_id: rel_iam_tenant_id,
            },
            db,
            cxt,
        )
        .await?;
        Ok(())
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
            },
            db,
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
                status: RbumCertStatusKind::Enabled,
                rel_rbum_cert_conf_id: IamCertServ::get_id_by_code(IamCertKind::UserPwd.to_string().as_str(), rel_iam_tenant_id, db).await?,
                rel_rbum_item_id: Some(iam_item_id.to_string()),
            },
            db,
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
            &RbumBasicFilterReq {
                rbum_item_id: Some(iam_item_id.to_string()),
                rbum_cert_conf_id: Some(IamCertServ::get_id_by_code(IamCertKind::UserPwd.to_string().as_str(), Some(rel_iam_tenant_id), db).await?),
                ..Default::default()
            },
            None,
            None,
            db,
            cxt,
        )
        .await?;
        if certs.len() > 0 {
            return Err(TardisError::NotFound(format!(
                "there are multiple credentials of kind {}",
                IamCertKind::UserPwd.to_string()
            )));
        }
        if let Some(cert) = certs.get(0) {
            RbumCertServ::change_sk(&cert.id, &modify_req.original_sk.0, &modify_req.new_sk.0, &RbumBasicFilterReq::default(), db, cxt).await
        } else {
            Err(TardisError::NotFound(format!("cannot find credential of kind {}", IamCertKind::UserPwd.to_string())))
        }
    }
}
