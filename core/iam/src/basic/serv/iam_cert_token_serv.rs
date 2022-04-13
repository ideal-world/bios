use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;

use bios_basic::rbum::dto::rbum_cert_conf_dto::{RbumCertConfAddReq, RbumCertConfModifyReq};
use bios_basic::rbum::dto::rbum_cert_dto::RbumCertAddReq;
use bios_basic::rbum::rbum_enumeration::{RbumCertRelKind, RbumCertStatusKind};
use bios_basic::rbum::serv::rbum_cert_serv::{RbumCertConfServ, RbumCertServ};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;

use crate::iam_constants;
use crate::basic::dto::iam_cert_conf_dto::{IamTokenCertConfAddReq, IamTokenCertConfModifyReq};
use crate::iam_enumeration::IamCertTokenKind;
use crate::basic::serv::iam_cert_serv::IamCertServ;

pub struct IamCertTokenServ;

impl<'a> IamCertTokenServ {
    pub async fn add_cert_conf(
        add_req: &mut IamTokenCertConfAddReq,
        token_kind: IamCertTokenKind,
        rel_iam_tenant_id: Option<String>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<()> {
        RbumCertConfServ::add_rbum(
            &mut RbumCertConfAddReq {
                code: TrimString(token_kind.to_string()),
                name: TrimString(add_req.name.to_string()),
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
                expire_sec: add_req.expire_sec,
                coexist_num: Some(add_req.coexist_num),
                conn_uri: None,
                rel_rbum_domain_id: iam_constants::get_rbum_basic_info().domain_iam_id.to_string(),
                rel_rbum_item_id: rel_iam_tenant_id,
            },
            funs,
            cxt,
        )
        .await?;
        Ok(())
    }

    pub async fn modify_cert_conf(id: &str, modify_req: &mut IamTokenCertConfModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        RbumCertConfServ::modify_rbum(
            id,
            &mut RbumCertConfModifyReq {
                name: modify_req.name.clone(),
                note: None,
                ak_note: None,
                ak_rule: None,
                sk_note: None,
                sk_rule: None,
                sk_need: None,
                sk_encrypted: None,
                repeatable: None,
                is_basic: None,
                rest_by_kinds: None,
                expire_sec: modify_req.expire_sec,
                coexist_num: modify_req.coexist_num,
                conn_uri: None,
            },
            funs,
            cxt,
        )
        .await?;
        Ok(())
    }

    pub async fn add_cert(
        token: &str,
        token_kind: IamCertTokenKind,
        iam_item_id: &str,
        rel_iam_tenant_id: &str,
        from_cert_id: &str,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<()> {
        // TODO cache
        RbumCertServ::add_rbum(
            &mut RbumCertAddReq {
                ak: TrimString(token.to_string()),
                sk: None,
                ext: Some(from_cert_id.to_string()),
                start_time: None,
                end_time: None,
                status: RbumCertStatusKind::Enabled,
                rel_rbum_cert_conf_id: Some(IamCertServ::get_id_by_code(token_kind.to_string().as_str(), Some(rel_iam_tenant_id), funs).await?),
                rel_rbum_kind: RbumCertRelKind::Item,
                rel_rbum_id: iam_item_id.to_string(),
            },
            funs,
            cxt,
        )
        .await?;
        Ok(())
    }
}
