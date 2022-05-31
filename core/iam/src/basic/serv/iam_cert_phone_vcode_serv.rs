use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::rand::Rng;

use bios_basic::rbum::dto::rbum_cert_conf_dto::{RbumCertConfAddReq, RbumCertConfModifyReq};
use bios_basic::rbum::dto::rbum_cert_dto::RbumCertAddReq;
use bios_basic::rbum::rbum_enumeration::{RbumCertRelKind, RbumCertStatusKind};
use bios_basic::rbum::serv::rbum_cert_serv::{RbumCertConfServ, RbumCertServ};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;

use crate::basic::dto::iam_cert_conf_dto::IamPhoneVCodeCertConfAddOrModifyReq;
use crate::basic::dto::iam_cert_dto::IamPhoneVCodeCertAddReq;
use crate::iam_config::IamBasicConfigApi;
use crate::iam_enumeration::IamCertKind;

pub struct IamCertPhoneVCodeServ;

impl<'a> IamCertPhoneVCodeServ {
    pub async fn add_cert_conf(
        add_req: &IamPhoneVCodeCertConfAddOrModifyReq,
        rel_iam_item_id: Option<String>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<String> {
        let id = RbumCertConfServ::add_rbum(
            &mut RbumCertConfAddReq {
                code: TrimString(IamCertKind::PhoneVCode.to_string()),
                name: TrimString(IamCertKind::PhoneVCode.to_string()),
                note: None,
                ak_note: add_req.ak_note.clone(),
                ak_rule: add_req.ak_rule.clone(),
                sk_note: None,
                sk_rule: None,
                sk_need: Some(false),
                sk_dynamic: None,
                sk_encrypted: Some(false),
                repeatable: None,
                is_basic: Some(false),
                rest_by_kinds: None,
                expire_sec: None,
                coexist_num: Some(1),
                conn_uri: None,
                rel_rbum_domain_id: funs.iam_basic_domain_iam_id(),
                rel_rbum_item_id: rel_iam_item_id,
            },
            funs,
            cxt,
        )
        .await?;
        Ok(id)
    }

    pub async fn modify_cert_conf(id: &str, modify_req: &IamPhoneVCodeCertConfAddOrModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
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
                conn_uri: None,
            },
            funs,
            cxt,
        )
        .await?;
        Ok(())
    }

    pub async fn add_cert(
        add_req: &IamPhoneVCodeCertAddReq,
        account_id: &str,
        rel_rbum_cert_conf_id: &str,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<String> {
        let vcode = Self::get_vcode();
        let id = RbumCertServ::add_rbum(
            &mut RbumCertAddReq {
                ak: TrimString(add_req.phone.to_string()),
                sk: None,
                vcode: Some(TrimString(vcode.clone())),
                ext: None,
                start_time: None,
                end_time: None,
                conn_uri: None,
                status: RbumCertStatusKind::Pending,
                rel_rbum_cert_conf_id: Some(rel_rbum_cert_conf_id.to_string()),
                rel_rbum_kind: RbumCertRelKind::Item,
                rel_rbum_id: account_id.to_string(),
            },
            funs,
            cxt,
        )
        .await?;
        // TODO send vcode
        Ok(id)
    }

    fn get_vcode() -> String {
        let mut rand = tardis::rand::thread_rng();
        let vcode: i32 = rand.gen_range(1000..9999);
        format!("{}", vcode)
    }

    // TODO
}
