use bios_basic::rbum::dto::rbum_cert_conf_dto::{RbumCertConfAddReq, RbumCertConfModifyReq};
use bios_basic::rbum::dto::rbum_cert_dto::RbumCertAddReq;
use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumCertConfFilterReq, RbumCertFilterReq};
use bios_basic::rbum::rbum_enumeration::{RbumCertConfStatusKind, RbumCertRelKind, RbumCertStatusKind};
use bios_basic::rbum::serv::rbum_cert_serv::{RbumCertConfServ, RbumCertServ};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use tardis::basic::field::TrimString;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    TardisFunsInst,
};

use crate::basic::dto::iam_cert_conf_dto::IamCertConfAkSkAddOrModifyReq;
use crate::basic::dto::iam_cert_dto::IamCertAkSkAddReq;
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
                name: add_req.name.clone(),
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
                expire_sec: add_req.expire_sec,
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
                name: Some(modify_req.name.clone()),
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
                coexist_num: None,
                conn_uri: None,
                status: None,
            },
            funs,
            ctx,
        )
        .await
    }

    pub async fn add_cert(add_req: &IamCertAkSkAddReq, ak: &str, sk: &str, rel_rbum_cert_conf_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
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
        let tenant_id = add_req.tenant_id.clone();
        let app_id = add_req.app_id.clone();
        let new_ctx = TardisContext {
            owner: RBUM_SYSTEM_OWNER.to_string(),
            own_paths: if app_id.is_some() {
                format!("{}/{}", tenant_id, app_id.clone().unwrap_or_default())
            } else {
                tenant_id.clone()
            },
            ..ctx.clone()
        };
        let rel_rbum_id = if app_id.is_some() { app_id.clone().unwrap_or_default() } else { tenant_id.clone() };
        let id = RbumCertServ::add_rbum(
            &mut RbumCertAddReq {
                ak: ak.into(),
                sk: Some(sk.into()),
                sk_invisible: None,
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
                rel_rbum_id: rel_rbum_id.clone(),
                is_outside: true,
                is_ignore_check_sk: false,
            },
            funs,
            &new_ctx,
        )
        .await?;
        IamIdentCacheServ::add_aksk(ak, sk, &tenant_id, app_id, cert_conf.expire_sec, funs).await?;
        Ok(id)
    }
    pub async fn delete_cert(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let resp = RbumCertServ::peek_rbum(id, &RbumCertFilterReq { ..Default::default() }, funs, ctx).await?;
        RbumCertServ::delete_rbum(id, funs, ctx).await?;
        IamIdentCacheServ::delete_aksk(&resp.ak, funs).await?;
        Ok(())
    }
}
