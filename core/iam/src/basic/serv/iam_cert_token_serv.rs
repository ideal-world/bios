use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::TardisFunsInst;

use bios_basic::rbum::dto::rbum_cert_conf_dto::{RbumCertConfAddReq, RbumCertConfModifyReq};
use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumCertConfFilterReq};
use bios_basic::rbum::serv::rbum_cert_serv::RbumCertConfServ;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;

use crate::basic::dto::iam_cert_conf_dto::{IamTokenCertConfAddReq, IamTokenCertConfModifyReq};
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_key_cache_serv::IamIdentCacheServ;
use crate::iam_config::IamBasicConfigApi;
use crate::iam_enumeration::IamCertTokenKind;

pub struct IamCertTokenServ;

impl IamCertTokenServ {
    pub async fn add_cert_conf(
        add_req: &IamTokenCertConfAddReq,
        token_kind: IamCertTokenKind,
        rel_iam_item_id: Option<String>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
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
                ext: None,
                sk_need: Some(false),
                sk_dynamic: None,
                sk_encrypted: Some(false),
                repeatable: None,
                is_basic: Some(false),
                rest_by_kinds: None,
                expire_sec: add_req.expire_sec,
                sk_lock_cycle_sec: None,
                sk_lock_err_times: None,
                sk_lock_duration_sec: None,
                coexist_num: Some(add_req.coexist_num),
                conn_uri: None,
                rel_rbum_domain_id: funs.iam_basic_domain_iam_id(),
                rel_rbum_item_id: rel_iam_item_id,
            },
            funs,
            ctx,
        )
        .await?;
        Ok(())
    }

    pub async fn modify_cert_conf(id: &str, modify_req: &IamTokenCertConfModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        RbumCertConfServ::modify_rbum(
            id,
            &mut RbumCertConfModifyReq {
                name: modify_req.name.clone(),
                note: None,
                ak_note: None,
                ak_rule: None,
                sk_note: None,
                sk_rule: None,
                ext: None,
                sk_need: None,
                sk_encrypted: None,
                repeatable: None,
                is_basic: None,
                rest_by_kinds: None,
                expire_sec: modify_req.expire_sec,
                sk_lock_cycle_sec: None,
                sk_lock_err_times: None,
                sk_lock_duration_sec: None,
                coexist_num: modify_req.coexist_num,
                conn_uri: None,
            },
            funs,
            ctx,
        )
        .await?;
        if modify_req.expire_sec.is_some() || modify_req.coexist_num.is_some() {
            IamCertServ::clean_cache_by_cert_conf(id, None, funs, ctx).await?;
        }
        Ok(())
    }

    pub async fn add_cert(
        token: &str,
        token_kind: &IamCertTokenKind,
        rel_iam_item_id: &str,
        rel_rbum_cert_conf_id: &str,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let cert_conf = RbumCertConfServ::peek_rbum(
            rel_rbum_cert_conf_id,
            &RbumCertConfFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        IamIdentCacheServ::add_token(token, token_kind, rel_iam_item_id, cert_conf.expire_sec, cert_conf.coexist_num, funs).await
    }

    pub async fn delete_cert(token: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        IamIdentCacheServ::delete_token_by_token(token, funs).await
    }
}
