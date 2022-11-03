use async_trait::async_trait;
use fancy_regex::Regex;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::chrono::{DateTime, Duration, Utc};
use tardis::db::sea_orm;
use tardis::db::sea_orm::sea_query::*;
use tardis::db::sea_orm::*;
use tardis::TardisFunsInst;
use tardis::{log, TardisFuns};

use crate::rbum::domain::{rbum_cert, rbum_cert_conf, rbum_domain, rbum_item};
use crate::rbum::dto::rbum_cert_conf_dto::{RbumCertConfAddReq, RbumCertConfDetailResp, RbumCertConfIdAndExtResp, RbumCertConfModifyReq, RbumCertConfSummaryResp};
use crate::rbum::dto::rbum_cert_dto::{RbumCertAddReq, RbumCertDetailResp, RbumCertModifyReq, RbumCertSummaryResp};
use crate::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumCertConfFilterReq, RbumCertFilterReq};
use crate::rbum::rbum_config::RbumConfigApi;
use crate::rbum::rbum_enumeration::{RbumCertRelKind, RbumCertStatusKind};
use crate::rbum::serv::rbum_crud_serv::{RbumCrudOperation, RbumCrudQueryPackage};
use crate::rbum::serv::rbum_domain_serv::RbumDomainServ;
use crate::rbum::serv::rbum_item_serv::RbumItemServ;
use crate::rbum::serv::rbum_rel_serv::RbumRelServ;
use crate::rbum::serv::rbum_set_serv::RbumSetServ;

pub struct RbumCertConfServ;

pub struct RbumCertServ;

#[async_trait]
impl RbumCrudOperation<rbum_cert_conf::ActiveModel, RbumCertConfAddReq, RbumCertConfModifyReq, RbumCertConfSummaryResp, RbumCertConfDetailResp, RbumCertConfFilterReq>
    for RbumCertConfServ
{
    fn get_table_name() -> &'static str {
        rbum_cert_conf::Entity.table_name()
    }

    async fn package_add(add_req: &RbumCertConfAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<rbum_cert_conf::ActiveModel> {
        Ok(rbum_cert_conf::ActiveModel {
            id: Set(TardisFuns::field.nanoid()),
            kind: Set(add_req.kind.to_string()),
            supplier: Set(add_req.supplier.to_string()),
            name: Set(add_req.name.to_string()),
            note: Set(add_req.note.as_ref().unwrap_or(&"".to_string()).to_string()),
            ak_note: Set(add_req.ak_note.as_ref().unwrap_or(&"".to_string()).to_string()),
            ak_rule: Set(add_req.ak_rule.as_ref().unwrap_or(&"".to_string()).to_string()),
            sk_note: Set(add_req.sk_note.as_ref().unwrap_or(&"".to_string()).to_string()),
            sk_rule: Set(add_req.sk_rule.as_ref().unwrap_or(&"".to_string()).to_string()),
            ext: Set(add_req.ext.as_ref().unwrap_or(&"".to_string()).to_string()),
            sk_dynamic: Set(add_req.sk_dynamic.unwrap_or(false)),
            sk_need: Set(add_req.sk_need.unwrap_or(true)),
            sk_encrypted: Set(add_req.sk_encrypted.unwrap_or(false)),
            repeatable: Set(add_req.repeatable.unwrap_or(true)),
            is_basic: Set(add_req.is_basic.unwrap_or(true)),
            is_ak_repeatable: Set(add_req.is_ak_repeatable.unwrap_or(false)),
            rest_by_kinds: Set(add_req.rest_by_kinds.as_ref().unwrap_or(&"".to_string()).to_string()),
            expire_sec: Set(add_req.expire_sec.unwrap_or(u32::MAX)),
            sk_lock_cycle_sec: Set(add_req.sk_lock_cycle_sec.unwrap_or(0)),
            sk_lock_err_times: Set(add_req.sk_lock_err_times.unwrap_or(0)),
            sk_lock_duration_sec: Set(add_req.sk_lock_duration_sec.unwrap_or(0)),
            coexist_num: Set(add_req.coexist_num.unwrap_or(1)),
            conn_uri: Set(add_req.conn_uri.as_ref().unwrap_or(&"".to_string()).to_string()),
            rel_rbum_domain_id: Set(add_req.rel_rbum_domain_id.to_string()),
            rel_rbum_item_id: Set(add_req.rel_rbum_item_id.as_ref().unwrap_or(&"".to_string()).to_string()),
            ..Default::default()
        })
    }

    async fn before_add_rbum(add_req: &mut RbumCertConfAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if let Some(is_basic) = add_req.is_basic {
            if is_basic {
                add_req.sk_dynamic = Some(false);
                if funs
                    .db()
                    .count(
                        Query::select()
                            .column(rbum_cert_conf::Column::Id)
                            .from(rbum_cert_conf::Entity)
                            .and_where(Expr::col(rbum_cert_conf::Column::IsBasic).eq(true))
                            .and_where(Expr::col(rbum_cert_conf::Column::RelRbumDomainId).eq(add_req.rel_rbum_domain_id.as_str()))
                            .and_where(Expr::col(rbum_cert_conf::Column::RelRbumItemId).eq(add_req.rel_rbum_item_id.as_ref().unwrap_or(&"".to_string()).as_str())),
                    )
                    .await?
                    > 0
                {
                    return Err(funs.err().conflict(&Self::get_obj_name(), "add", "is_basic already exists", "409-rbum-cert-conf-basic-exist"));
                }
            }
        }
        Self::check_scope(&add_req.rel_rbum_domain_id, RbumDomainServ::get_table_name(), funs, ctx).await?;
        if let Some(rel_rbum_item_id) = &add_req.rel_rbum_item_id {
            Self::check_scope(rel_rbum_item_id, RbumItemServ::get_table_name(), funs, ctx).await?;
        }
        if let Some(ak_rule) = &add_req.ak_rule {
            Regex::new(ak_rule).map_err(|e| funs.err().bad_request(&Self::get_obj_name(), "add", &format!("ak rule is invalid:{}", e), "400-rbum-cert-conf-ak-rule-invalid"))?;
        }
        if let Some(sk_rule) = &add_req.sk_rule {
            Regex::new(sk_rule).map_err(|e| funs.err().bad_request(&Self::get_obj_name(), "add", &format!("sk rule is invalid:{}", e), "400-rbum-cert-conf-sk-rule-invalid"))?;
        }
        if funs
            .db()
            .count(
                Query::select()
                    .column(rbum_cert_conf::Column::Id)
                    .from(rbum_cert_conf::Entity)
                    .and_where(Expr::col(rbum_cert_conf::Column::Kind).eq(add_req.kind.0.as_str()))
                    .and_where(Expr::col(rbum_cert_conf::Column::Supplier).eq(add_req.supplier.0.as_str()))
                    .and_where(Expr::col(rbum_cert_conf::Column::RelRbumDomainId).eq(add_req.rel_rbum_domain_id.as_str()))
                    .and_where(Expr::col(rbum_cert_conf::Column::RelRbumItemId).eq(add_req.rel_rbum_item_id.as_ref().unwrap_or(&"".to_string()).as_str())),
            )
            .await?
            > 0
        {
            return Err(funs.err().conflict(&Self::get_obj_name(), "add", &format!("code {} already exists", add_req.kind), "409-rbum-*-code-exist"));
        }
        Ok(())
    }

    async fn package_modify(id: &str, modify_req: &RbumCertConfModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<rbum_cert_conf::ActiveModel> {
        let mut rbum_cert_conf = rbum_cert_conf::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(name) = &modify_req.name {
            rbum_cert_conf.name = Set(name.to_string());
        }
        if let Some(note) = &modify_req.note {
            rbum_cert_conf.note = Set(note.to_string());
        }
        if let Some(ak_note) = &modify_req.ak_note {
            rbum_cert_conf.ak_note = Set(ak_note.to_string());
        }
        if let Some(ak_rule) = &modify_req.ak_rule {
            rbum_cert_conf.ak_rule = Set(ak_rule.to_string());
        }
        if let Some(sk_note) = &modify_req.sk_note {
            rbum_cert_conf.sk_note = Set(sk_note.to_string());
        }
        if let Some(sk_rule) = &modify_req.sk_rule {
            rbum_cert_conf.sk_rule = Set(sk_rule.to_string());
        }
        if let Some(ext) = &modify_req.ext {
            rbum_cert_conf.ext = Set(ext.to_string());
        }
        if let Some(sk_need) = modify_req.sk_need {
            rbum_cert_conf.sk_need = Set(sk_need);
        }
        if let Some(sk_encrypted) = modify_req.sk_encrypted {
            rbum_cert_conf.sk_encrypted = Set(sk_encrypted);
        }
        if let Some(repeatable) = modify_req.repeatable {
            rbum_cert_conf.repeatable = Set(repeatable);
        }
        if let Some(is_basic) = modify_req.is_basic {
            rbum_cert_conf.is_basic = Set(is_basic);
            let rbum_cert_conf_resp = Self::peek_rbum(id, &RbumCertConfFilterReq::default(), funs, ctx).await?;
            if is_basic {
                rbum_cert_conf.sk_dynamic = Set(!is_basic);
                if funs
                    .db()
                    .count(
                        Query::select()
                            .column(rbum_cert_conf::Column::Id)
                            .from(rbum_cert_conf::Entity)
                            .and_where(Expr::col(rbum_cert_conf::Column::IsBasic).eq(true))
                            .and_where(Expr::col(rbum_cert_conf::Column::Id).ne(id))
                            .and_where(Expr::col(rbum_cert_conf::Column::RelRbumDomainId).eq(rbum_cert_conf_resp.rel_rbum_domain_id.as_str()))
                            .and_where(Expr::col(rbum_cert_conf::Column::RelRbumItemId).eq(rbum_cert_conf_resp.rel_rbum_item_id.as_str())),
                    )
                    .await?
                    > 0
                {
                    return Err(funs.err().conflict(&Self::get_obj_name(), "modify", "is_basic already exists", "409-rbum-cert-conf-basic-exist"));
                }
            }
        }
        if let Some(rest_by_kinds) = &modify_req.rest_by_kinds {
            rbum_cert_conf.rest_by_kinds = Set(rest_by_kinds.to_string());
        }
        if let Some(expire_sec) = modify_req.expire_sec {
            rbum_cert_conf.expire_sec = Set(expire_sec);
        }
        if let Some(sk_lock_cycle_sec) = modify_req.sk_lock_cycle_sec {
            rbum_cert_conf.sk_lock_cycle_sec = Set(sk_lock_cycle_sec);
        }
        if let Some(sk_lock_err_times) = modify_req.sk_lock_err_times {
            rbum_cert_conf.sk_lock_err_times = Set(sk_lock_err_times);
        }
        if let Some(sk_lock_duration_sec) = modify_req.sk_lock_duration_sec {
            rbum_cert_conf.sk_lock_duration_sec = Set(sk_lock_duration_sec);
        }
        if let Some(coexist_num) = modify_req.coexist_num {
            rbum_cert_conf.coexist_num = Set(coexist_num);
        }
        if let Some(conn_uri) = &modify_req.conn_uri {
            rbum_cert_conf.conn_uri = Set(conn_uri.to_string());
        }
        Ok(rbum_cert_conf)
    }

    async fn after_modify_rbum(id: &str, _: &mut RbumCertConfModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let rbum_cert_conf = Self::get_rbum(id, &RbumCertConfFilterReq::default(), funs, ctx).await?;
        let key = &format!(
            "{}{}",
            funs.rbum_conf_cache_key_cert_code_(),
            TardisFuns::crypto.base64.encode(&format!(
                "{}{}{}",
                &rbum_cert_conf.kind, &rbum_cert_conf.rel_rbum_domain_id, &rbum_cert_conf.rel_rbum_item_id
            ))
        );
        funs.cache()
            .set_ex(
                key,
                &TardisFuns::json.obj_to_string(&RbumCertConfIdAndExtResp {
                    id: rbum_cert_conf.id.clone(),
                    ext: rbum_cert_conf.ext.clone(),
                })?,
                funs.rbum_conf_cache_key_cert_code_expire_sec(),
            )
            .await?;
        Ok(())
    }

    async fn before_delete_rbum(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<RbumCertConfDetailResp>> {
        if funs
            .db()
            .count(
                Query::select()
                    .column(rbum_cert_conf::Column::Id)
                    .from(rbum_cert_conf::Entity)
                    .and_where(Expr::col(rbum_cert_conf::Column::Id).eq(id))
                    .and_where(Expr::col(rbum_cert_conf::Column::IsBasic).eq(true)),
            )
            .await?
            > 0
        {
            return Err(funs.err().conflict(&Self::get_obj_name(), "delete", "is_basic is true", "409-rbum-cert-conf-basic-delete"));
        }
        Self::check_ownership(id, funs, ctx).await?;
        Self::check_exist_before_delete(id, RbumCertServ::get_table_name(), rbum_cert::Column::RelRbumCertConfId.as_str(), funs).await?;
        let result = Self::peek_rbum(
            id,
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
        let key = &format!(
            "{}{}",
            funs.rbum_conf_cache_key_cert_code_(),
            TardisFuns::crypto.base64.encode(&format!("{}{}{}", &result.kind, &result.rel_rbum_domain_id, &result.rel_rbum_item_id))
        );
        funs.cache().del(key).await?;
        Ok(None)
    }

    async fn package_query(is_detail: bool, filter: &RbumCertConfFilterReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::Id),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::Kind),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::Supplier),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::Name),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::Note),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::AkNote),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::AkRule),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::SkNote),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::SkRule),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::Ext),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::SkNeed),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::SkDynamic),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::SkEncrypted),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::Repeatable),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::IsBasic),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::IsAkRepeatable),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::RestByKinds),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::ExpireSec),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::SkLockCycleSec),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::SkLockErrTimes),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::SkLockDurationSec),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::CoexistNum),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::ConnUri),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::RelRbumDomainId),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::RelRbumItemId),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::OwnPaths),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::Owner),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::CreateTime),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::UpdateTime),
            ])
            .from(rbum_cert_conf::Entity);
        if let Some(rel_rbum_domain_id) = &filter.rel_rbum_domain_id {
            query.and_where(Expr::tbl(rbum_cert_conf::Entity, rbum_cert_conf::Column::RelRbumDomainId).eq(rel_rbum_domain_id.to_string()));
        }
        if let Some(rbum_item_id) = &filter.rel_rbum_item_id {
            query.and_where(Expr::tbl(rbum_cert_conf::Entity, rbum_cert_conf::Column::RelRbumItemId).eq(rbum_item_id.to_string()));
        }
        if is_detail {
            query
                .expr_as(Expr::tbl(rbum_domain::Entity, rbum_domain::Column::Name), Alias::new("rel_rbum_domain_name"))
                .expr_as(Expr::tbl(rbum_item::Entity, rbum_item::Column::Name).if_null(""), Alias::new("rel_rbum_item_name"))
                .inner_join(
                    rbum_domain::Entity,
                    Expr::tbl(rbum_domain::Entity, rbum_domain::Column::Id).equals(rbum_cert_conf::Entity, rbum_cert_conf::Column::RelRbumDomainId),
                )
                .left_join(
                    rbum_item::Entity,
                    Expr::tbl(rbum_item::Entity, rbum_item::Column::Id).equals(rbum_cert_conf::Entity, rbum_cert_conf::Column::RelRbumItemId),
                );
        }
        query.with_filter(Self::get_table_name(), &filter.basic, is_detail, false, ctx);
        Ok(query)
    }
}

impl RbumCertConfServ {
    pub async fn get_rbum_cert_conf_id_and_ext_by_kind_supplier(
        kind: &str,
        supplier: &str,
        rbum_domain_id: &str,
        rbum_item_id: &str,
        funs: &TardisFunsInst,
    ) -> TardisResult<Option<RbumCertConfIdAndExtResp>> {
        let key = &format!(
            "{}{}",
            funs.rbum_conf_cache_key_cert_code_(),
            TardisFuns::crypto.base64.encode(&format!("{}{}{}{}", kind, supplier, rbum_domain_id, rbum_item_id))
        );
        if let Some(cached_info) = funs.cache().get(key).await? {
            Ok(Some(TardisFuns::json.str_to_obj(&cached_info)?))
        } else if let Some(rbum_cert_conf_id_and_ext) = funs
            .db()
            .get_dto::<RbumCertConfIdAndExtResp>(
                Query::select()
                    .column(rbum_cert_conf::Column::Id)
                    .column(rbum_cert_conf::Column::Ext)
                    .from(rbum_cert_conf::Entity)
                    .and_where(Expr::col(rbum_cert_conf::Column::Kind).eq(kind))
                    .and_where(Expr::col(rbum_cert_conf::Column::Supplier).eq(supplier))
                    .and_where(Expr::col(rbum_cert_conf::Column::RelRbumDomainId).eq(rbum_domain_id))
                    .and_where(Expr::col(rbum_cert_conf::Column::RelRbumItemId).eq(rbum_item_id)),
            )
            .await?
        {
            funs.cache()
                .set_ex(
                    key,
                    &TardisFuns::json.obj_to_string(&rbum_cert_conf_id_and_ext)?,
                    funs.rbum_conf_cache_key_cert_code_expire_sec(),
                )
                .await?;
            Ok(Some(rbum_cert_conf_id_and_ext))
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl RbumCrudOperation<rbum_cert::ActiveModel, RbumCertAddReq, RbumCertModifyReq, RbumCertSummaryResp, RbumCertDetailResp, RbumCertFilterReq> for RbumCertServ {
    fn get_table_name() -> &'static str {
        rbum_cert::Entity.table_name()
    }

    async fn package_add(add_req: &RbumCertAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<rbum_cert::ActiveModel> {
        Ok(rbum_cert::ActiveModel {
            id: Set(TardisFuns::field.nanoid()),
            ak: Set(add_req.ak.to_string()),
            sk: Set(add_req.sk.as_ref().unwrap_or(&TrimString("".to_string())).to_string()),
            ext: Set(add_req.ext.as_ref().unwrap_or(&"".to_string()).to_string()),
            start_time: Set(add_req.start_time.unwrap_or_else(Utc::now)),
            end_time: Set(add_req.end_time.unwrap_or(Utc::now() + Duration::days(365 * 100))),
            conn_uri: Set(add_req.conn_uri.as_ref().unwrap_or(&"".to_string()).to_string()),
            status: Set(add_req.status.to_int()),
            rel_rbum_cert_conf_id: Set(add_req.rel_rbum_cert_conf_id.as_ref().unwrap_or(&"".to_string()).to_string()),
            rel_rbum_kind: Set(add_req.rel_rbum_kind.to_int()),
            rel_rbum_id: Set(add_req.rel_rbum_id.to_string()),
            ..Default::default()
        })
    }

    async fn before_add_rbum(add_req: &mut RbumCertAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if add_req.sk.is_some() && add_req.vcode.is_some() {
            return Err(funs.err().bad_request(&Self::get_obj_name(), "add", "sk and vcode can only have one", "400-rbum-cert-sk-vcode-only-one"));
        }
        if let Some(rel_rbum_cert_conf_id) = &add_req.rel_rbum_cert_conf_id {
            Self::check_ownership_with_table_name(rel_rbum_cert_conf_id, RbumCertConfServ::get_table_name(), funs, ctx).await?;
        }
        match add_req.rel_rbum_kind {
            RbumCertRelKind::Item => {
                if !add_req.is_outside {
                    Self::check_scope(&add_req.rel_rbum_id, RbumItemServ::get_table_name(), funs, ctx).await?;
                }
            }
            RbumCertRelKind::Set => Self::check_scope(&add_req.rel_rbum_id, RbumSetServ::get_table_name(), funs, ctx).await?,
            RbumCertRelKind::Rel => Self::check_ownership_with_table_name(&add_req.rel_rbum_id, RbumRelServ::get_table_name(), funs, ctx).await?,
        }

        if let Some(rel_rbum_cert_conf_id) = &add_req.rel_rbum_cert_conf_id {
            let rbum_cert_conf = RbumCertConfServ::peek_rbum(
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
            RbumCertServ::check_cert_conf_constraint_by_add(add_req, &rbum_cert_conf, funs, ctx).await?;
            // Encrypt Sk
            if rbum_cert_conf.sk_encrypted {
                if let Some(sk) = &add_req.sk {
                    let sk = Self::encrypt_sk(&sk.0, &add_req.ak.0, rel_rbum_cert_conf_id)?;
                    add_req.sk = Some(TrimString(sk));
                }
            }
            // Fill Time
            if let Some(start_time) = &add_req.start_time {
                add_req.end_time = Some(*start_time + Duration::seconds(rbum_cert_conf.expire_sec as i64));
            } else {
                let now = Utc::now();
                add_req.start_time = Some(now);
                add_req.end_time = Some(now + Duration::seconds(rbum_cert_conf.expire_sec as i64));
            }
            if rbum_cert_conf.sk_dynamic {
                add_req.end_time = None;
            }
        }
        Ok(())
    }

    async fn after_add_rbum(id: &str, add_req: &RbumCertAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let rbum_cert = Self::peek_rbum(
            id,
            &RbumCertFilterReq {
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
        if let Some(rel_rbum_cert_conf_id) = &rbum_cert.rel_rbum_cert_conf_id {
            let rbum_cert_conf = RbumCertConfServ::peek_rbum(
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
            // Delete Old Certs
            if rbum_cert_conf.coexist_num != 0 {
                let need_delete_rbum_cert_ids = Self::paginate_id_rbums(
                    &RbumCertFilterReq {
                        rel_rbum_kind: Some(add_req.rel_rbum_kind.clone()),
                        rel_rbum_id: Some(add_req.rel_rbum_id.clone()),
                        rel_rbum_cert_conf_ids: Some(vec![rel_rbum_cert_conf_id.clone()]),
                        ..Default::default()
                    },
                    // Skip normal records
                    2,
                    rbum_cert_conf.coexist_num as u64 - 1,
                    Some(true),
                    None,
                    funs,
                    ctx,
                )
                .await?;
                for need_delete_rbum_cert_id in need_delete_rbum_cert_ids.records {
                    Self::delete_rbum(&need_delete_rbum_cert_id, funs, ctx).await?;
                }
            }
        }
        if let Some(vcode) = &add_req.vcode {
            Self::add_vcode_to_cache(add_req.ak.0.as_str(), vcode.0.as_str(), &ctx.own_paths, funs).await?;
        }
        Ok(())
    }

    async fn package_modify(id: &str, modify_req: &RbumCertModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<rbum_cert::ActiveModel> {
        let mut rbum_cert = rbum_cert::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(ak) = &modify_req.ak {
            rbum_cert.ak = Set(ak.to_string());
        }
        if let Some(sk) = &modify_req.sk {
            rbum_cert.sk = Set(sk.to_string());
        }
        if let Some(ext) = &modify_req.ext {
            rbum_cert.ext = Set(ext.to_string());
        }
        if let Some(start_time) = modify_req.start_time {
            rbum_cert.start_time = Set(start_time);
        }
        if let Some(end_time) = modify_req.end_time {
            rbum_cert.end_time = Set(end_time);
        }
        if let Some(status) = &modify_req.status {
            rbum_cert.status = Set(status.to_int());
        }
        if let Some(conn_uri) = &modify_req.conn_uri {
            rbum_cert.conn_uri = Set(conn_uri.to_string());
        }
        Ok(rbum_cert)
    }

    async fn before_modify_rbum(id: &str, modify_req: &mut RbumCertModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        Self::check_ownership(id, funs, ctx).await?;
        let rbum_cert = Self::peek_rbum(
            id,
            &RbumCertFilterReq {
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
        if let Some(rel_rbum_cert_conf_id) = &rbum_cert.rel_rbum_cert_conf_id {
            let rbum_cert_conf = RbumCertConfServ::peek_rbum(
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
            Self::check_cert_conf_constraint_by_modify(id, modify_req, &rbum_cert_conf, funs, ctx).await?;
            // Encrypt Sk
            if (modify_req.sk.is_some() || modify_req.ak.is_some()) && rbum_cert_conf.sk_encrypted {
                if modify_req.ak.is_some() && modify_req.sk.is_none() {
                    return Err(funs.err().conflict(&Self::get_obj_name(), "modify", "sk cannot be empty", "409-rbum-cert-ak-duplicate"));
                }
                if let Some(sk) = &modify_req.sk {
                    let sk = Self::encrypt_sk(&sk.0, modify_req.ak.as_ref().unwrap_or(&TrimString(rbum_cert.ak)).as_ref(), rel_rbum_cert_conf_id)?;
                    modify_req.sk = Some(TrimString(sk));
                }
            }
        }
        Ok(())
    }

    async fn package_query(is_detail: bool, filter: &RbumCertFilterReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_cert::Entity, rbum_cert::Column::Id),
                (rbum_cert::Entity, rbum_cert::Column::Ak),
                (rbum_cert::Entity, rbum_cert::Column::Ext),
                (rbum_cert::Entity, rbum_cert::Column::StartTime),
                (rbum_cert::Entity, rbum_cert::Column::EndTime),
                (rbum_cert::Entity, rbum_cert::Column::ConnUri),
                (rbum_cert::Entity, rbum_cert::Column::Status),
                (rbum_cert::Entity, rbum_cert::Column::RelRbumCertConfId),
                (rbum_cert::Entity, rbum_cert::Column::RelRbumKind),
                (rbum_cert::Entity, rbum_cert::Column::RelRbumId),
                (rbum_cert::Entity, rbum_cert::Column::OwnPaths),
                (rbum_cert::Entity, rbum_cert::Column::Owner),
                (rbum_cert::Entity, rbum_cert::Column::CreateTime),
                (rbum_cert::Entity, rbum_cert::Column::UpdateTime),
            ])
            .expr_as(
                Expr::tbl(rbum_cert_conf::Entity, rbum_cert_conf::Column::Name).if_null(""),
                Alias::new("rel_rbum_cert_conf_name"),
            )
            .expr_as(
                Expr::tbl(rbum_cert_conf::Entity, rbum_cert_conf::Column::Name).if_null(""),
                Alias::new("rel_rbum_cert_conf_code"),
            )
            .from(rbum_cert::Entity)
            .left_join(
                rbum_cert_conf::Entity,
                Expr::tbl(rbum_cert_conf::Entity, rbum_cert_conf::Column::Id).equals(rbum_cert::Entity, rbum_cert::Column::RelRbumCertConfId),
            );
        if let Some(ak) = &filter.ak {
            query.and_where(Expr::tbl(rbum_cert::Entity, rbum_cert::Column::Ak).eq(ak.to_string()));
        }
        if let Some(status) = &filter.status {
            query.and_where(Expr::tbl(rbum_cert::Entity, rbum_cert::Column::Status).eq(status.to_int()));
        }
        if let Some(rel_rbum_cert_conf_ids) = &filter.rel_rbum_cert_conf_ids {
            query.and_where(Expr::tbl(rbum_cert::Entity, rbum_cert::Column::RelRbumCertConfId).is_in(rel_rbum_cert_conf_ids.clone()));
        }
        if let Some(rel_rbum_kind) = &filter.rel_rbum_kind {
            query.and_where(Expr::tbl(rbum_cert::Entity, rbum_cert::Column::RelRbumKind).eq(rel_rbum_kind.to_int()));
        }
        if let Some(rel_rbum_id) = &filter.rel_rbum_id {
            query.and_where(Expr::tbl(rbum_cert::Entity, rbum_cert::Column::RelRbumId).eq(rel_rbum_id.to_string()));
        }
        query.with_filter(Self::get_table_name(), &filter.basic, is_detail, false, ctx);
        Ok(query)
    }
}

impl RbumCertServ {
    pub async fn add_vcode_to_cache(ak: &str, vcode: &str, own_paths: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        funs.cache()
            .set_ex(
                format!("{}{}:{}", funs.rbum_conf_cache_key_cert_vcode_info_(), own_paths, ak).as_str(),
                vcode.to_string().as_str(),
                funs.rbum_conf_cache_key_cert_vcode_expire_sec(),
            )
            .await?;
        Ok(())
    }

    pub async fn get_vcode_in_cache(ak: &str, own_paths: &str, funs: &TardisFunsInst) -> TardisResult<Option<String>> {
        let vcode = funs.cache().get(format!("{}{}:{}", funs.rbum_conf_cache_key_cert_vcode_info_(), own_paths, ak).as_str()).await?;
        Ok(vcode)
    }

    pub async fn get_and_delete_vcode_in_cache(ak: &str, own_paths: &str, funs: &TardisFunsInst) -> TardisResult<Option<String>> {
        let vcode = funs.cache().get(format!("{}{}:{}", funs.rbum_conf_cache_key_cert_vcode_info_(), own_paths, ak).as_str()).await?;
        if vcode.is_some() {
            funs.cache().del(format!("{}{}:{}", funs.rbum_conf_cache_key_cert_vcode_info_(), own_paths, ak).as_str()).await?;
        }
        Ok(vcode)
    }

    pub async fn check_exist(ak: &str, rbum_cert_conf_id: &str, own_paths: &str, funs: &TardisFunsInst) -> TardisResult<bool> {
        let mut query = Query::select();
        query
            .column(rbum_cert::Column::Id)
            .from(rbum_cert::Entity)
            .and_where(Expr::col(rbum_cert::Column::Ak).eq(ak))
            .and_where(Expr::col(rbum_cert::Column::RelRbumCertConfId).eq(rbum_cert_conf_id))
            .and_where(Expr::col(rbum_cert::Column::OwnPaths).eq(own_paths))
            .and_where(Expr::col(rbum_cert::Column::Status).eq(RbumCertStatusKind::Enabled.to_int()))
            .and_where(Expr::col(rbum_cert::Column::StartTime).lte(Utc::now().naive_utc()));
        funs.db().count(&query).await.map(|r| r > 0)
    }

    pub async fn validate_by_spec_cert_conf(
        ak: &str,
        input_sk: &str,
        rbum_cert_conf_id: &str,
        ignore_end_time: bool,
        own_paths: &str,
        funs: &TardisFunsInst,
    ) -> TardisResult<(String, RbumCertRelKind, String)> {
        #[derive(Debug, sea_orm::FromQueryResult)]
        struct IdAndSkResp {
            pub id: String,
            pub sk: String,
            pub rel_rbum_kind: RbumCertRelKind,
            pub rel_rbum_id: String,
            pub end_time: DateTime<Utc>,
        }

        #[derive(Debug, sea_orm::FromQueryResult)]
        struct CertConfPeekResp {
            pub sk_encrypted: bool,
            pub sk_dynamic: bool,
            pub sk_lock_cycle_sec: u32,
            pub sk_lock_err_times: u8,
            pub sk_lock_duration_sec: u32,
        }
        let mut query = Query::select();
        query
            .column(rbum_cert::Column::Id)
            .column(rbum_cert::Column::Sk)
            .column(rbum_cert::Column::RelRbumKind)
            .column(rbum_cert::Column::RelRbumId)
            .column(rbum_cert::Column::EndTime)
            .from(rbum_cert::Entity)
            .and_where(Expr::col(rbum_cert::Column::Ak).eq(ak))
            .and_where(Expr::col(rbum_cert::Column::RelRbumCertConfId).eq(rbum_cert_conf_id))
            .and_where(Expr::col(rbum_cert::Column::OwnPaths).eq(own_paths))
            .and_where(Expr::col(rbum_cert::Column::Status).eq(RbumCertStatusKind::Enabled.to_int()))
            .and_where(Expr::col(rbum_cert::Column::StartTime).lte(Utc::now().naive_utc()));
        let rbum_cert = funs.db().get_dto::<IdAndSkResp>(&query).await?;
        if let Some(rbum_cert) = rbum_cert {
            if funs.cache().exists(&format!("{}{}", funs.rbum_conf_cache_key_cert_locked_(), rbum_cert.rel_rbum_id)).await? {
                return Err(funs.err().unauthorized(&Self::get_obj_name(), "valid", "cert is locked", "400-rbum-cert-lock"));
            }
            if !ignore_end_time && rbum_cert.end_time < Utc::now() {
                return Err(funs.err().conflict(&Self::get_obj_name(), "valid", "sk is expired", "409-rbum-cert-sk-expire"));
            }
            let cert_conf_peek_resp = funs
                .db()
                .get_dto::<CertConfPeekResp>(
                    Query::select()
                        .column(rbum_cert_conf::Column::SkEncrypted)
                        .column(rbum_cert_conf::Column::SkDynamic)
                        .column(rbum_cert_conf::Column::SkLockCycleSec)
                        .column(rbum_cert_conf::Column::SkLockErrTimes)
                        .column(rbum_cert_conf::Column::SkLockDurationSec)
                        .from(rbum_cert_conf::Entity)
                        .and_where(Expr::col(rbum_cert_conf::Column::Id).eq(rbum_cert_conf_id)),
                )
                .await?
                .ok_or_else(|| funs.err().not_found(&Self::get_obj_name(), "valid", "not found cert conf", "404-rbum-cert-conf-not-exist"))?;
            let input_sk = if cert_conf_peek_resp.sk_encrypted {
                Self::encrypt_sk(input_sk, ak, rbum_cert_conf_id)?
            } else {
                input_sk.to_string()
            };
            let storage_sk = if cert_conf_peek_resp.sk_dynamic {
                if let Some(cached_vcode) = Self::get_and_delete_vcode_in_cache(ak, own_paths, funs).await? {
                    cached_vcode
                } else {
                    log::warn!(
                        "validation error [vcode is not exist] by ak {},rbum_cert_conf_id {}, own_paths {}",
                        ak,
                        rbum_cert_conf_id,
                        own_paths
                    );
                    Self::process_lock_in_cache(
                        &rbum_cert.rel_rbum_id,
                        cert_conf_peek_resp.sk_lock_cycle_sec,
                        cert_conf_peek_resp.sk_lock_err_times,
                        cert_conf_peek_resp.sk_lock_duration_sec,
                        funs,
                    )
                    .await?;
                    return Err(funs.err().unauthorized(&Self::get_obj_name(), "valid", "validation error", "401-rbum-cert-valid-error"));
                }
            } else {
                rbum_cert.sk
            };
            if storage_sk == input_sk {
                funs.cache().del(&format!("{}{}", funs.rbum_conf_cache_key_cert_err_times_(), &rbum_cert.rel_rbum_id)).await?;
                Ok((rbum_cert.id, rbum_cert.rel_rbum_kind, rbum_cert.rel_rbum_id))
            } else {
                log::warn!(
                    "validation error [sk is not match] by ak {},rbum_cert_conf_id {}, own_paths {}",
                    ak,
                    rbum_cert_conf_id,
                    own_paths
                );
                Self::process_lock_in_cache(
                    &rbum_cert.rel_rbum_id,
                    cert_conf_peek_resp.sk_lock_cycle_sec,
                    cert_conf_peek_resp.sk_lock_err_times,
                    cert_conf_peek_resp.sk_lock_duration_sec,
                    funs,
                )
                .await?;
                Err(funs.err().unauthorized(&Self::get_obj_name(), "valid", "validation error", "401-rbum-cert-valid-error"))
            }
        } else {
            log::warn!("validation error by ak {},rbum_cert_conf_id {}, own_paths {}", ak, rbum_cert_conf_id, own_paths);
            Err(funs.err().unauthorized(&Self::get_obj_name(), "valid", "validation error", "401-rbum-cert-valid-error"))
        }
    }

    /// Support different credentials of ak + basic credentials of sk as validation method
    pub async fn validate_by_ak_and_basic_sk(
        ak: &str,
        input_sk: &str,
        rel_rbum_kind: &RbumCertRelKind,
        ignore_end_time: bool,
        own_paths: &str,
        funs: &TardisFunsInst,
    ) -> TardisResult<(String, RbumCertRelKind, String)> {
        #[derive(Debug, sea_orm::FromQueryResult)]
        struct IdAndSkResp {
            pub id: String,
            pub sk: String,
            pub rel_rbum_id: String,
            pub rel_rbum_cert_conf_id: String,
            pub end_time: DateTime<Utc>,
        }

        #[derive(Debug, sea_orm::FromQueryResult)]
        struct CertConfPeekResp {
            pub is_basic: bool,
            pub sk_encrypted: bool,
            pub rel_rbum_domain_id: String,
            pub sk_lock_cycle_sec: u32,
            pub sk_lock_err_times: u8,
            pub sk_lock_duration_sec: u32,
        }
        let mut query = Query::select();
        query
            .column(rbum_cert::Column::Id)
            .column(rbum_cert::Column::Sk)
            .column(rbum_cert::Column::RelRbumId)
            .column(rbum_cert::Column::EndTime)
            .column(rbum_cert::Column::RelRbumCertConfId)
            .from(rbum_cert::Entity)
            .and_where(Expr::col(rbum_cert::Column::Ak).eq(ak))
            .and_where(Expr::col(rbum_cert::Column::RelRbumKind).eq(rel_rbum_kind.to_int()))
            .and_where(Expr::col(rbum_cert::Column::OwnPaths).eq(own_paths))
            .and_where(Expr::col(rbum_cert::Column::Status).eq(RbumCertStatusKind::Enabled.to_int()))
            .and_where(Expr::col(rbum_cert::Column::StartTime).lte(Utc::now().naive_utc()));
        let rbum_cert = funs.db().get_dto::<IdAndSkResp>(&query).await?;
        if let Some(rbum_cert) = rbum_cert {
            if funs.cache().exists(&format!("{}{}", funs.rbum_conf_cache_key_cert_locked_(), rbum_cert.rel_rbum_id)).await? {
                return Err(funs.err().unauthorized(&Self::get_obj_name(), "valid", "cert is locked", "401-rbum-cert-lock"));
            }
            if !ignore_end_time && rbum_cert.end_time < Utc::now() {
                return Err(funs.err().conflict(&Self::get_obj_name(), "valid", "sk is expired", "409-rbum-cert-sk-expire"));
            }
            if let Some(rbum_cert_conf_id) = Some(rbum_cert.rel_rbum_cert_conf_id) {
                let cert_conf_peek_resp = funs
                    .db()
                    .get_dto::<CertConfPeekResp>(
                        Query::select()
                            .column(rbum_cert_conf::Column::IsBasic)
                            .column(rbum_cert_conf::Column::RelRbumDomainId)
                            .column(rbum_cert_conf::Column::SkEncrypted)
                            .column(rbum_cert_conf::Column::SkLockCycleSec)
                            .column(rbum_cert_conf::Column::SkLockErrTimes)
                            .column(rbum_cert_conf::Column::SkLockDurationSec)
                            .from(rbum_cert_conf::Entity)
                            .and_where(Expr::col(rbum_cert_conf::Column::Id).eq(rbum_cert_conf_id.as_str())),
                    )
                    .await?
                    .ok_or_else(|| funs.err().not_found(&Self::get_obj_name(), "valid", "not found cert conf", "404-rbum-cert-conf-not-exist"))?;
                let verify_input_sk = if cert_conf_peek_resp.sk_encrypted {
                    Self::encrypt_sk(input_sk, ak, rbum_cert_conf_id.as_str())?
                } else {
                    input_sk.to_string()
                };
                if rbum_cert.sk == verify_input_sk {
                    funs.cache().del(&format!("{}{}", funs.rbum_conf_cache_key_cert_err_times_(), &rbum_cert.rel_rbum_id)).await?;
                    Ok((rbum_cert.id, rel_rbum_kind.clone(), rbum_cert.rel_rbum_id))
                } else if !cert_conf_peek_resp.is_basic {
                    Ok(Self::validate_by_non_basic_cert_conf_with_basic_sk(
                        rbum_cert.rel_rbum_id.as_str(),
                        cert_conf_peek_resp.rel_rbum_domain_id.as_str(),
                        input_sk,
                        ignore_end_time,
                        funs,
                    )
                    .await?)
                } else {
                    log::warn!(
                        "validation error [sk is not match] by ak {},rel_rbum_cert_conf_id {}, own_paths {}",
                        ak,
                        rbum_cert_conf_id,
                        own_paths
                    );
                    Self::process_lock_in_cache(
                        &rbum_cert.rel_rbum_id,
                        cert_conf_peek_resp.sk_lock_cycle_sec,
                        cert_conf_peek_resp.sk_lock_err_times,
                        cert_conf_peek_resp.sk_lock_duration_sec,
                        funs,
                    )
                    .await?;
                    Err(funs.err().unauthorized(&Self::get_obj_name(), "valid", "validation error", "401-rbum-cert-valid-error"))
                }
            } else {
                log::warn!("validation error by ak {},rbum_cert_conf_id is None, own_paths {}", ak, own_paths);
                Err(funs.err().unauthorized(&Self::get_obj_name(), "valid", "validation error", "401-rbum-cert-valid-error"))
            }
        } else {
            log::warn!("validation error by ak {},rel_rbum_kind {}, own_paths {}", ak, rel_rbum_kind, own_paths);
            Err(funs.err().unauthorized(&Self::get_obj_name(), "valid", "validation error", "401-rbum-cert-valid-error"))
        }
    }

    async fn validate_by_non_basic_cert_conf_with_basic_sk(
        rel_rbum_id: &str,
        rel_rbum_domain_id: &str,
        input_sk: &str,
        ignore_end_time: bool,
        funs: &TardisFunsInst,
    ) -> TardisResult<(String, RbumCertRelKind, String)> {
        #[derive(Debug, sea_orm::FromQueryResult)]
        struct BasicCertInfoResp {
            pub id: String,
            pub ak: String,
            pub sk: String,
            pub rel_rbum_kind: RbumCertRelKind,
            pub end_time: DateTime<Utc>,
            pub sk_encrypted: bool,
            pub rel_rbum_cert_conf_id: String,
            pub sk_lock_cycle_sec: u32,
            pub sk_lock_err_times: u8,
            pub sk_lock_duration_sec: u32,
        }
        let rbum_basic_cert_info_resp = funs
            .db()
            .get_dto::<BasicCertInfoResp>(
                Query::select()
                    .expr_as(Expr::tbl(rbum_cert::Entity, rbum_cert::Column::Id).if_null(""), Alias::new("id"))
                    .column(rbum_cert::Column::Ak)
                    .column(rbum_cert::Column::Sk)
                    .column(rbum_cert::Column::RelRbumKind)
                    .column(rbum_cert::Column::EndTime)
                    .column(rbum_cert::Column::RelRbumCertConfId)
                    .column(rbum_cert_conf::Column::SkEncrypted)
                    .column(rbum_cert_conf::Column::SkLockCycleSec)
                    .column(rbum_cert_conf::Column::SkLockErrTimes)
                    .column(rbum_cert_conf::Column::SkLockDurationSec)
                    .from(rbum_cert::Entity)
                    .inner_join(
                        rbum_cert_conf::Entity,
                        Expr::tbl(rbum_cert_conf::Entity, rbum_cert_conf::Column::Id).equals(rbum_cert::Entity, rbum_cert::Column::RelRbumCertConfId),
                    )
                    .and_where(Expr::col(rbum_cert::Column::RelRbumId).eq(rel_rbum_id))
                    .and_where(Expr::tbl(rbum_cert_conf::Entity, rbum_cert_conf::Column::RelRbumDomainId).eq(rel_rbum_domain_id))
                    .and_where(Expr::tbl(rbum_cert_conf::Entity, rbum_cert_conf::Column::IsBasic).eq(true)),
            )
            .await?
            .ok_or_else(|| funs.err().not_found(&Self::get_obj_name(), "valid", "not found basic cert conf", "404-rbum-cert-conf-not-exist"))?;
        if !ignore_end_time && rbum_basic_cert_info_resp.end_time < Utc::now() {
            return Err(funs.err().conflict(&Self::get_obj_name(), "valid", "basic sk is expired", "409-rbum-cert-sk-expire"));
        }
        let verify_input_sk = if rbum_basic_cert_info_resp.sk_encrypted {
            Self::encrypt_sk(input_sk, &rbum_basic_cert_info_resp.ak, &rbum_basic_cert_info_resp.rel_rbum_cert_conf_id)?
        } else {
            input_sk.to_string()
        };

        if rbum_basic_cert_info_resp.sk == verify_input_sk {
            funs.cache().del(&format!("{}{}", funs.rbum_conf_cache_key_cert_err_times_(), rel_rbum_id)).await?;
            Ok((rbum_basic_cert_info_resp.id, rbum_basic_cert_info_resp.rel_rbum_kind, rel_rbum_id.to_string()))
        } else {
            log::warn!(
                "validation error [sk is not match] by ak {},rbum_cert_conf_id {}, rel_rbum_id {}",
                rbum_basic_cert_info_resp.ak,
                rbum_basic_cert_info_resp.rel_rbum_cert_conf_id,
                rel_rbum_id
            );
            Self::process_lock_in_cache(
                rel_rbum_id,
                rbum_basic_cert_info_resp.sk_lock_cycle_sec,
                rbum_basic_cert_info_resp.sk_lock_err_times,
                rbum_basic_cert_info_resp.sk_lock_duration_sec,
                funs,
            )
            .await?;
            Err(funs.err().unauthorized(&Self::get_obj_name(), "valid", "basic validation error", "401-rbum-cert-valid-error"))
        }
    }

    async fn process_lock_in_cache(rbum_item_id: &str, sk_lock_cycle_sec: u32, sk_lock_err_times: u8, sk_lock_duration_sec: u32, funs: &TardisFunsInst) -> TardisResult<()> {
        if sk_lock_cycle_sec == 0 || sk_lock_err_times == 0 || sk_lock_duration_sec == 0 {
            return Ok(());
        }
        let err_times = funs.cache().incr(&format!("{}{}", funs.rbum_conf_cache_key_cert_err_times_(), rbum_item_id), 1).await?;
        if sk_lock_err_times <= err_times as u8 {
            funs.cache().set_ex(&format!("{}{}", funs.rbum_conf_cache_key_cert_locked_(), rbum_item_id), "", sk_lock_duration_sec as usize).await?;
            funs.cache().del(&format!("{}{}", funs.rbum_conf_cache_key_cert_err_times_(), rbum_item_id)).await?;
        } else if err_times == 1 {
            funs.cache().expire(&format!("{}{}", funs.rbum_conf_cache_key_cert_err_times_(), rbum_item_id), sk_lock_cycle_sec as usize).await?;
        }
        Ok(())
    }

    pub async fn show_sk(id: &str, filter: &RbumCertFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        #[derive(sea_orm::FromQueryResult)]
        struct SkResp {
            pub sk: String,
        }
        let mut query = Query::select();
        query.column((rbum_cert::Entity, rbum_cert::Column::Sk)).from(rbum_cert::Entity).and_where(Expr::tbl(rbum_cert::Entity, rbum_cert::Column::Id).eq(id)).with_filter(
            Self::get_table_name(),
            &filter.basic,
            false,
            false,
            ctx,
        );
        let sk_resp = funs.db().get_dto::<SkResp>(&query).await?;
        if let Some(sk_resp) = sk_resp {
            Ok(sk_resp.sk)
        } else {
            Err(funs.err().not_found(&Self::get_obj_name(), "show_sk", "not found cert record", "404-rbum-*-obj-not-exist"))
        }
    }

    pub async fn reset_sk(id: &str, new_sk: &str, filter: &RbumCertFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let rbum_cert = Self::peek_rbum(id, filter, funs, ctx).await?;
        let new_sk = if let Some(rel_rbum_cert_conf_id) = &rbum_cert.rel_rbum_cert_conf_id {
            let rbum_cert_conf = RbumCertConfServ::peek_rbum(
                rel_rbum_cert_conf_id,
                &RbumCertConfFilterReq {
                    basic: filter.basic.clone(),
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?;
            if !rbum_cert_conf.sk_rule.is_empty()
                && !Regex::new(&rbum_cert_conf.sk_rule)
                    .map_err(|e| {
                        funs.err().bad_request(
                            &Self::get_obj_name(),
                            "reset_sk",
                            &format!("sk rule is invalid:{}", e),
                            "400-rbum-cert-conf-sk-rule-invalid",
                        )
                    })?
                    .is_match(new_sk)
                    .unwrap_or(false)
            {
                return Err(funs.err().bad_request(
                    &Self::get_obj_name(),
                    "reset_sk",
                    &format!("sk {} is not match sk rule", new_sk),
                    "400-rbum-cert-conf-sk-rule-not-match",
                ));
            }
            if rbum_cert_conf.sk_encrypted {
                Self::encrypt_sk(new_sk, &rbum_cert.ak, rel_rbum_cert_conf_id)?
            } else {
                new_sk.to_string()
            }
        } else {
            new_sk.to_string()
        };
        let old_sk = Self::show_sk(id, filter, funs, ctx).await?;
        // todo new_sk is duplicate, Later to conf repeatable to judge
        if new_sk == old_sk {
            return Err(funs.err().bad_request(
                &Self::get_obj_name(),
                "reset_sk",
                &format!("sk {} is duplicate", new_sk),
                "400-rbum-cert-reset-sk-duplicate",
            ));
        }
        funs.db()
            .update_one(
                rbum_cert::ActiveModel {
                    id: Set(id.to_string()),
                    sk: Set(new_sk),
                    ..Default::default()
                },
                ctx,
            )
            .await?;
        Ok(())
    }

    pub async fn change_sk(id: &str, original_sk: &str, input_sk: &str, filter: &RbumCertFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let rbum_cert = Self::peek_rbum(id, filter, funs, ctx).await?;
        let stored_sk = Self::show_sk(id, filter, funs, ctx).await?;
        let (new_sk, end_time) = if let Some(rel_rbum_cert_conf_id) = &rbum_cert.rel_rbum_cert_conf_id {
            let rbum_cert_conf = RbumCertConfServ::peek_rbum(rel_rbum_cert_conf_id, &RbumCertConfFilterReq::default(), funs, ctx).await?;
            let original_sk = if rbum_cert_conf.sk_encrypted {
                Self::encrypt_sk(original_sk, &rbum_cert.ak, &rbum_cert_conf.id)?
            } else {
                original_sk.to_string()
            };
            if original_sk != stored_sk {
                return Err(funs.err().unauthorized(&Self::get_obj_name(), "change_sk", "sk not match", "401-rbum-cert-ori-sk-not-match"));
            }
            if !rbum_cert_conf.sk_rule.is_empty()
                && !Regex::new(&rbum_cert_conf.sk_rule)
                    .map_err(|e| {
                        funs.err().bad_request(
                            &Self::get_obj_name(),
                            "change_sk",
                            &format!("sk rule is invalid:{}", e),
                            "400-rbum-cert-conf-sk-rule-invalid",
                        )
                    })?
                    .is_match(input_sk)
                    .unwrap_or(false)
            {
                return Err(funs.err().bad_request(
                    &Self::get_obj_name(),
                    "change_sk",
                    &format!("sk {} is not match sk rule", input_sk),
                    "400-rbum-cert-conf-sk-rule-not-match",
                ));
            }
            let new_sk = if rbum_cert_conf.sk_encrypted {
                Self::encrypt_sk(input_sk, &rbum_cert.ak, &rbum_cert_conf.id)?
            } else {
                input_sk.to_string()
            };
            if !rbum_cert_conf.repeatable && original_sk == new_sk {
                return Err(funs.err().bad_request(
                    &Self::get_obj_name(),
                    "change_sk",
                    &format!("sk {} cannot be duplicated", input_sk),
                    "400-rbum-cert-ak-duplicate",
                ));
            }
            let end_time = Utc::now() + Duration::seconds(rbum_cert_conf.expire_sec as i64);
            (new_sk, end_time)
        } else {
            if original_sk != stored_sk {
                return Err(funs.err().unauthorized(&Self::get_obj_name(), "change_sk", "sk not match", "401-rbum-cert-ori-sk-not-match"));
            }
            (input_sk.to_string(), rbum_cert.start_time + (rbum_cert.end_time - rbum_cert.start_time))
        };
        // todo new_sk is duplicate, Later to conf repeatable to judge
        if original_sk == input_sk {
            return Err(funs.err().bad_request(
                &Self::get_obj_name(),
                "reset_sk",
                &format!("sk {} is duplicate", new_sk),
                "400-rbum-cert-reset-sk-duplicate",
            ));
        }
        funs.db()
            .update_one(
                rbum_cert::ActiveModel {
                    id: Set(id.to_string()),
                    sk: Set(new_sk),
                    end_time: Set(end_time),
                    ..Default::default()
                },
                ctx,
            )
            .await?;
        Ok(())
    }

    async fn check_cert_conf_constraint_by_add(add_req: &RbumCertAddReq, rbum_cert_conf: &RbumCertConfSummaryResp, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if rbum_cert_conf.sk_need && add_req.sk.is_none() {
            return Err(funs.err().bad_request(&Self::get_obj_name(), "add", "sk is required", "400-rbum-cert-sk-require"));
        }
        if rbum_cert_conf.sk_dynamic && add_req.sk.is_some() {
            return Err(funs.err().bad_request(&Self::get_obj_name(), "add", "sk should be empty when dynamic model", "400-rbum-cert-sk-need-empty"));
        }
        if rbum_cert_conf.sk_dynamic && add_req.vcode.is_none() {
            return Err(funs.err().bad_request(&Self::get_obj_name(), "add", "vcode is required when dynamic model", "400-rbum-cert-vcode-require"));
        }
        if !rbum_cert_conf.ak_rule.is_empty()
            && !Regex::new(&rbum_cert_conf.ak_rule)
                .map_err(|e| funs.err().bad_request(&Self::get_obj_name(), "add", &format!("ak rule is invalid:{}", e), "400-rbum-cert-conf-ak-rule-invalid"))?
                .is_match(add_req.ak.as_ref())
                .unwrap_or(false)
        {
            return Err(funs.err().bad_request(
                &Self::get_obj_name(),
                "add",
                &format!("ak {} is not match ak rule", add_req.ak),
                "400-rbum-cert-conf-ak-rule-not-match",
            ));
        }
        if rbum_cert_conf.sk_need && !rbum_cert_conf.sk_rule.is_empty() {
            let sk = add_req.sk.as_ref().ok_or_else(|| funs.err().bad_request(&Self::get_obj_name(), "add", "sk is required", "400-rbum-cert-sk-require"))?.to_string();
            if !Regex::new(&rbum_cert_conf.sk_rule)
                .map_err(|e| funs.err().bad_request(&Self::get_obj_name(), "add", &format!("sk rule is invalid:{}", e), "400-rbum-cert-conf-sk-rule-invalid"))?
                .is_match(&sk)
                .unwrap_or(false)
            {
                return Err(funs.err().bad_request(
                    &Self::get_obj_name(),
                    "add",
                    &format!("sk {} is not match sk rule", &sk),
                    "400-rbum-cert-conf-sk-rule-not-match",
                ));
            }
        }
        if !rbum_cert_conf.is_ak_repeatable
            && funs
                .db()
                .count(
                    Query::select()
                        .column(rbum_cert::Column::Id)
                        .from(rbum_cert::Entity)
                        .and_where(Expr::col(rbum_cert::Column::RelRbumKind).eq(add_req.rel_rbum_kind.to_int()))
                        .and_where(Expr::col(rbum_cert::Column::Ak).eq(add_req.ak.0.as_str()))
                        .and_where(Expr::col(rbum_cert::Column::RelRbumCertConfId).eq(add_req.rel_rbum_cert_conf_id.clone()))
                        .and_where(Expr::col(rbum_cert::Column::OwnPaths).like(format!("{}%", ctx.own_paths).as_str())),
                )
                .await?
                > 0
        {
            return Err(funs.err().conflict(&Self::get_obj_name(), "add", "ak is used", "409-rbum-cert-ak-duplicate"));
        }
        Ok(())
    }

    async fn check_cert_conf_constraint_by_modify(
        id: &str,
        modify_req: &RbumCertModifyReq,
        rbum_cert_conf: &RbumCertConfSummaryResp,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        if let Some(ak) = &modify_req.ak {
            if !rbum_cert_conf.ak_rule.is_empty()
                && !Regex::new(&rbum_cert_conf.ak_rule)
                    .map_err(|e| funs.err().bad_request(&Self::get_obj_name(), "modify", &format!("ak rule is invalid:{}", e), "400-rbum-cert-conf-ak-rule-invalid"))?
                    .is_match(ak.as_ref())
                    .unwrap_or(false)
            {
                return Err(funs.err().bad_request(
                    &Self::get_obj_name(),
                    "modify",
                    &format!("ak {} is not match ak rule", ak),
                    "400-rbum-cert-conf-ak-rule-not-match",
                ));
            }
        }
        if let Some(sk) = &modify_req.sk {
            if rbum_cert_conf.sk_need
                && !rbum_cert_conf.sk_rule.is_empty()
                && !Regex::new(&rbum_cert_conf.sk_rule)
                    .map_err(|e| funs.err().bad_request(&Self::get_obj_name(), "modify", &format!("sk rule is invalid:{}", e), "400-rbum-cert-conf-sk-rule-invalid"))?
                    .is_match(sk.as_ref())
                    .unwrap_or(false)
            {
                return Err(funs.err().bad_request(
                    &Self::get_obj_name(),
                    "modify",
                    &format!("sk {} is not match sk rule", sk),
                    "400-rbum-cert-conf-sk-rule-not-match",
                ));
            }
        }
        if !rbum_cert_conf.is_ak_repeatable
            && modify_req.ak.is_some()
            && funs
                .db()
                .count(
                    Query::select()
                        .column(rbum_cert::Column::Id)
                        .from(rbum_cert::Entity)
                        .and_where(Expr::col(rbum_cert::Column::Ak).eq(modify_req.ak.as_ref().unwrap().0.as_str()))
                        .and_where(Expr::col(rbum_cert::Column::RelRbumCertConfId).eq(rbum_cert_conf.id.clone()))
                        .and_where(Expr::col(rbum_cert::Column::OwnPaths).like(format!("{}%", ctx.own_paths).as_str()))
                        .and_where(Expr::col(rbum_cert::Column::Id).ne(format!("{}%", id).as_str())),
                )
                .await?
                > 0
        {
            return Err(funs.err().conflict(&Self::get_obj_name(), "modify", "ak is used", "409-rbum-cert-ak-duplicate"));
        }
        Ok(())
    }

    fn encrypt_sk(sk: &str, ak: &str, rbum_cert_conf_id: &str) -> TardisResult<String> {
        TardisFuns::crypto.digest.sha512(format!("{}-{}-{}", sk, ak, rbum_cert_conf_id).as_str())
    }
}
