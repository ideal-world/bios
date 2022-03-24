use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisRelDBlConnection;
use tardis::TardisFuns;
use tardis::web::web_resp::TardisPage;

use bios_basic::rbum::dto::filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::dto::rbum_cert_conf_dto::{RbumCertConfAddReq, RbumCertConfDetailResp, RbumCertConfModifyReq, RbumCertConfSummaryResp};
use bios_basic::rbum::dto::rbum_cert_dto::RbumCertAddReq;
use bios_basic::rbum::rbum_enumeration::RbumCertStatusKind;
use bios_basic::rbum::serv::rbum_cert_serv::{RbumCertConfServ, RbumCertServ};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;

use crate::basic::constants;
use crate::basic::dto::iam_cert_conf_dto::{IamMailVCodeCertConfAddOrModifyReq, IamPhoneVCodeCertConfAddOrModifyReq, IamUserPwdCertConfAddOrModifyReq};
use crate::basic::enumeration::IamIdentKind;

pub struct IamCertServ;

impl<'a> IamCertServ {
    pub fn get_new_pwd() -> String {
        TardisFuns::field.nanoid_len(10)
    }

    pub async fn init_global_ident_conf(db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::add_cert_conf_user_pwd(
            &mut IamUserPwdCertConfAddOrModifyReq {
                ak_note: None,
                ak_rule: None,
                sk_note: None,
                sk_rule: None,
                repeatable: Some(true),
                expire_sec: None,
            },
            None,
            db,
            cxt,
        )
        .await?;

        Self::add_cert_conf_mail_vcode(&mut IamMailVCodeCertConfAddOrModifyReq { ak_note: None, ak_rule: None }, None, db, cxt).await?;

        Self::add_cert_conf_phone_vcode(&mut IamPhoneVCodeCertConfAddOrModifyReq { ak_note: None, ak_rule: None }, None, db, cxt).await?;

        Ok(())
    }

    pub async fn add_cert_conf_user_pwd(
        add_req: &mut IamUserPwdCertConfAddOrModifyReq,
        rel_iam_tenant_id: Option<String>,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<()> {
        RbumCertConfServ::add_rbum(
            &mut RbumCertConfAddReq {
                code: TrimString(IamIdentKind::UserPwd.to_string()),
                name: TrimString(IamIdentKind::UserPwd.to_string()),
                note: None,
                ak_note: add_req.ak_note.clone(),
                ak_rule: add_req.ak_rule.clone(),
                sk_note: add_req.sk_note.clone(),
                sk_rule: add_req.sk_rule.clone(),
                sk_need: Some(true),
                sk_encrypted: Some(true),
                repeatable: add_req.repeatable.clone(),
                is_basic: Some(true),
                rest_by_kinds: Some(format!("{},{}", IamIdentKind::MailVCode.to_string(), IamIdentKind::PhoneVCode.to_string())),
                expire_sec: add_req.expire_sec,
                coexist_num: None,
                rel_rbum_domain_id: constants::get_rbum_basic_info().domain_iam_id.to_string(),
                rel_rbum_item_id: rel_iam_tenant_id,
            },
            db,
            cxt,
        )
        .await?;
        Ok(())
    }

    pub async fn modify_cert_conf_user_pwd(id: &str, modify_req: &mut IamUserPwdCertConfAddOrModifyReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
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

    pub async fn add_cert_conf_mail_vcode(
        add_req: &mut IamMailVCodeCertConfAddOrModifyReq,
        rel_iam_tenant_id: Option<String>,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<()> {
        RbumCertConfServ::add_rbum(
            &mut RbumCertConfAddReq {
                code: TrimString(IamIdentKind::MailVCode.to_string()),
                name: TrimString(IamIdentKind::MailVCode.to_string()),
                note: None,
                ak_note: add_req.ak_note.clone(),
                ak_rule: add_req.ak_rule.clone(),
                sk_note: None,
                sk_rule: None,
                sk_need: Some(false),
                sk_encrypted: Some(false),
                repeatable: None,
                is_basic: Some(false),
                rest_by_kinds: None,
                expire_sec: None,
                coexist_num: None,
                rel_rbum_domain_id: constants::get_rbum_basic_info().domain_iam_id.to_string(),
                rel_rbum_item_id: rel_iam_tenant_id,
            },
            db,
            cxt,
        )
        .await?;
        Ok(())
    }

    pub async fn modify_cert_conf_mail_vcode(
        id: &str,
        modify_req: &mut IamMailVCodeCertConfAddOrModifyReq,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<()> {
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
            },
            db,
            cxt,
        )
        .await?;
        Ok(())
    }

    pub async fn add_cert_conf_phone_vcode(
        add_req: &mut IamPhoneVCodeCertConfAddOrModifyReq,
        rel_iam_tenant_id: Option<String>,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<()> {
        RbumCertConfServ::add_rbum(
            &mut RbumCertConfAddReq {
                code: TrimString(IamIdentKind::PhoneVCode.to_string()),
                name: TrimString(IamIdentKind::PhoneVCode.to_string()),
                note: None,
                ak_note: add_req.ak_note.clone(),
                ak_rule: add_req.ak_rule.clone(),
                sk_note: None,
                sk_rule: None,
                sk_need: Some(false),
                sk_encrypted: Some(false),
                repeatable: None,
                is_basic: Some(false),
                rest_by_kinds: None,
                expire_sec: None,
                coexist_num: None,
                rel_rbum_domain_id: constants::get_rbum_basic_info().domain_iam_id.to_string(),
                rel_rbum_item_id: rel_iam_tenant_id,
            },
            db,
            cxt,
        )
        .await?;
        Ok(())
    }

    pub async fn modify_cert_conf_phone_vcode(
        id: &str,
        modify_req: &mut IamPhoneVCodeCertConfAddOrModifyReq,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<()> {
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
            },
            db,
            cxt,
        )
        .await?;
        Ok(())
    }

    pub async fn get_cert_conf(id: &str, rbum_item_id: Option<String>, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<RbumCertConfDetailResp> {
        RbumCertConfServ::get_rbum(
            id,
            &RbumBasicFilterReq {
                rbum_domain_id: Some(constants::get_rbum_basic_info().domain_iam_id.to_string()),
                rbum_item_id,
                ..Default::default()
            },
            db,
            cxt,
        )
        .await
    }

    pub async fn paginate_cert_conf(
        q_name: Option<String>,
        rbum_item_id: Option<String>,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumCertConfSummaryResp>> {
        RbumCertConfServ::paginate_rbums(
            &RbumBasicFilterReq {
                name: q_name,
                rbum_domain_id: Some(constants::get_rbum_basic_info().domain_iam_id.to_string()),
                rbum_item_id,
                ..Default::default()
            },
            page_number,
            page_size,
            desc_sort_by_create,
            desc_sort_by_update,
            db,
            cxt,
        )
        .await
    }

    pub async fn delete_cert_conf(id: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        RbumCertConfServ::delete_rbum(id, db, cxt).await
    }

    pub async fn add_ident(
        ak: &str,
        sk: Option<&str>,
        ident_kind: IamIdentKind,
        rel_iam_tenant_id: Option<&str>,
        iam_item_id: &str,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<()> {
        RbumCertServ::add_rbum(
            &mut RbumCertAddReq {
                ak: TrimString(ak.to_string()),
                sk: sk.map(|v| TrimString::from(v.to_string())),
                ext: None,
                start_time: None,
                end_time: None,
                coexist_flag: None,
                status: RbumCertStatusKind::Enabled,
                rel_rbum_cert_conf_id: Self::get_id_by_code(ident_kind.to_string().as_str(), rel_iam_tenant_id, db).await?,
                rel_rbum_item_id: Some(iam_item_id.to_string()),
            },
            db,
            cxt,
        )
        .await?;
        Ok(())
    }

    pub async fn get_id_by_code(code: &str, rel_iam_tenant_id: Option<&str>, db: &TardisRelDBlConnection<'a>) -> TardisResult<String> {
        RbumCertConfServ::get_rbum_cert_conf_id_by_code(code, &constants::get_rbum_basic_info().domain_iam_id, rel_iam_tenant_id.unwrap_or(""), db)
            .await?
            .ok_or_else(|| TardisError::NotFound(format!("cert config code {} not found", code)))
    }
}
