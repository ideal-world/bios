use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisRelDBlConnection;
use tardis::TardisFuns;

use bios_basic::rbum::dto::rbum_cert_conf_dto::RbumCertConfAddReq;
use bios_basic::rbum::dto::rbum_cert_dto::RbumCertAddReq;
use bios_basic::rbum::enumeration::RbumCertStatusKind;
use bios_basic::rbum::serv::rbum_cert_serv::{RbumCertConfServ, RbumCertServ};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;

use crate::basic::constants;
use crate::basic::enumeration::IamIdentKind;

pub struct IamCertServ;

impl IamCertServ {
    pub fn get_new_pwd() -> String {
        TardisFuns::field.nanoid_len(10)
    }

    pub async fn init_global_ident_conf<'a>(db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        RbumCertConfServ::add_rbum(
            &mut RbumCertConfAddReq {
                code: TrimString(IamIdentKind::UserPwd.to_string()),
                name: TrimString(IamIdentKind::UserPwd.to_string()),
                note: None,
                ak_note: None,
                ak_rule: None,
                sk_note: None,
                sk_rule: None,
                sk_need: None,
                sk_encrypted: Some(true),
                repeatable: None,
                is_basic: Some(true),
                rest_by_kinds: Some(format!("{},{}", IamIdentKind::MailVCode.to_string(), IamIdentKind::PhoneVCode.to_string())),
                expire_sec: None,
                coexist_num: None,
                rel_rbum_domain_id: constants::get_rbum_basic_info().domain_iam_id.to_string(),
                rel_rbum_item_id: None,
            },
            db,
            cxt,
        )
        .await?;

        RbumCertConfServ::add_rbum(
            &mut RbumCertConfAddReq {
                code: TrimString(IamIdentKind::MailVCode.to_string()),
                name: TrimString(IamIdentKind::MailVCode.to_string()),
                note: None,
                ak_note: None,
                ak_rule: None,
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
                rel_rbum_item_id: None,
            },
            db,
            cxt,
        )
        .await?;

        RbumCertConfServ::add_rbum(
            &mut RbumCertConfAddReq {
                code: TrimString(IamIdentKind::PhoneVCode.to_string()),
                name: TrimString(IamIdentKind::PhoneVCode.to_string()),
                note: None,
                ak_note: None,
                ak_rule: None,
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
                rel_rbum_item_id: None,
            },
            db,
            cxt,
        )
        .await?;
        Ok(())
    }

    pub async fn add_ident<'a>(
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

    pub async fn get_id_by_code<'a>(code: &str, rel_iam_tenant_id: Option<&str>, db: &TardisRelDBlConnection<'a>) -> TardisResult<String> {
        RbumCertConfServ::get_rbum_cert_conf_id_by_code(code, &constants::get_rbum_basic_info().domain_iam_id, rel_iam_tenant_id.unwrap_or(""), db)
            .await?
            .ok_or_else(|| TardisError::NotFound(format!("cert config code {} not found", code)))
    }
}
