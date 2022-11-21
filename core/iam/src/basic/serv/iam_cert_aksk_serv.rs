use bios_basic::rbum::dto::rbum_cert_conf_dto::{RbumCertConfAddReq, RbumCertConfModifyReq};
use bios_basic::rbum::dto::rbum_cert_dto::RbumCertAddReq;
use bios_basic::rbum::dto::rbum_filer_dto::RbumCertFilterReq;
use bios_basic::rbum::rbum_enumeration::{RbumCertConfStatusKind, RbumCertRelKind, RbumCertStatusKind};
use bios_basic::rbum::serv::rbum_cert_serv::{RbumCertConfServ, RbumCertServ};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use tardis::basic::field::TrimString;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    TardisFuns, TardisFunsInst,
};

use crate::basic::dto::iam_cert_conf_dto::{IamCertConfAkSkAddOrModifyReq, IamCertConfMailVCodeAddOrModifyReq};
use crate::basic::dto::iam_cert_dto::{IamCertAkSkAddReq, IamCertMailVCodeAddReq};
use crate::basic::serv::iam_key_cache_serv::IamIdentCacheServ;
use crate::iam_config::IamBasicConfigApi;
use crate::iam_constants::RBUM_SYSTEM_OWNER;
use crate::iam_enumeration::IamCertKernelKind;

pub struct IamCertAkSkServ;

impl IamCertAkSkServ {
    ///rel_iam_item_id app_id
    pub async fn add_cert_conf(add_req: &IamCertConfAkSkAddOrModifyReq, rel_iam_item_id: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let id = RbumCertConfServ::add_rbum(
            &mut RbumCertConfAddReq {
                kind: TrimString(IamCertKernelKind::AkSk.to_string()),
                supplier: None,
                name: TrimString(IamCertKernelKind::AkSk.to_string()),
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
                is_ak_repeatable: None,
                rest_by_kinds: None,
                expire_sec: None,
                sk_lock_cycle_sec: None,
                sk_lock_err_times: None,
                sk_lock_duration_sec: None,
                coexist_num: Some(1),
                conn_uri: None,
                status: RbumCertConfStatusKind::Enabled,
                rel_rbum_domain_id: funs.iam_basic_domain_iam_id(),
                rel_rbum_item_id: rel_iam_item_id,
            },
            funs,
            ctx,
        )
        .await?;
        Ok(id)
    }

    ///never use
    pub async fn modify_cert_conf(id: &str, modify_req: &IamCertConfAkSkAddOrModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        RbumCertConfServ::modify_rbum(
            id,
            &mut RbumCertConfModifyReq {
                name: None,
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
                expire_sec: None,
                sk_lock_cycle_sec: None,
                sk_lock_err_times: None,
                sk_lock_duration_sec: None,
                coexist_num: None,
                conn_uri: None,
                status: None,
            },
            funs,
            ctx,
        );
        Ok(())
    }

    pub async fn add_cert(add_req: &IamCertAkSkAddReq, rel_rbum_id: &str, rel_rbum_cert_conf_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let new_ctx = TardisContext {
            owner: RBUM_SYSTEM_OWNER.to_string(),
            ..ctx.clone()
        };
        let id = RbumCertServ::add_rbum(
            &mut RbumCertAddReq {
                ak: TrimString(add_req.ak.clone()),
                sk: Some(TrimString(add_req.sk.clone())),
                kind: None,
                supplier: None,
                vcode: None,
                ext: None,
                start_time: None,
                end_time: None,
                conn_uri: None,
                status: RbumCertStatusKind::Enabled,
                rel_rbum_cert_conf_id: Some(rel_rbum_cert_conf_id.to_string()),
                rel_rbum_kind: RbumCertRelKind::Item,
                rel_rbum_id: rel_rbum_id.to_string(),
                is_outside: false,
            },
            funs,
            &new_ctx,
        )
        .await?;
        IamIdentCacheServ::add_aksk(&add_req.ak, &add_req.sk, rel_rbum_id, funs).await?;
        Ok(id)
    }
    pub async fn delete_cert(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let resp = RbumCertServ::peek_rbum(id, &RbumCertFilterReq { ..Default::default() }, funs, ctx).await?;
        RbumCertServ::delete_rbum(id, funs, ctx).await?;
        IamIdentCacheServ::delete_aksk(&resp.ak, funs).await?;
        Ok(())
    }
}
