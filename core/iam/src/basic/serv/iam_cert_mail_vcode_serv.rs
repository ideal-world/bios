use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisRelDBlConnection;

use bios_basic::rbum::dto::rbum_cert_conf_dto::{RbumCertConfAddReq, RbumCertConfModifyReq};
use bios_basic::rbum::serv::rbum_cert_serv::RbumCertConfServ;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;

use crate::basic::constants;
use crate::basic::dto::iam_cert_conf_dto::IamMailVCodeCertConfAddOrModifyReq;
use crate::basic::enumeration::IamCertKind;

pub struct IamCertMailVCodeServ;

impl<'a> IamCertMailVCodeServ {
    pub async fn add_cert_conf(
        add_req: &mut IamMailVCodeCertConfAddOrModifyReq,
        rel_iam_tenant_id: Option<String>,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<()> {
        RbumCertConfServ::add_rbum(
            &mut RbumCertConfAddReq {
                code: TrimString(IamCertKind::MailVCode.to_string()),
                name: TrimString(IamCertKind::MailVCode.to_string()),
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

    pub async fn modify_cert_conf(id: &str, modify_req: &mut IamMailVCodeCertConfAddOrModifyReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
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

    // TODO
}
