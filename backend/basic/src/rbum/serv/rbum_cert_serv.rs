use async_trait::async_trait;
use fancy_regex::Regex;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::cache::AsyncCommands;
use tardis::chrono::{DateTime, Duration, TimeDelta, Utc};
use tardis::db::reldb_client::IdResp;
use tardis::db::sea_orm::sea_query::*;
use tardis::db::sea_orm::*;
use tardis::db::sea_orm::{self, IdenStatic};
use tardis::TardisFunsInst;
use tardis::{log, TardisFuns};

use crate::rbum::domain::{rbum_cert, rbum_cert_conf, rbum_domain, rbum_item};
use crate::rbum::dto::rbum_cert_conf_dto::{RbumCertConfAddReq, RbumCertConfDetailResp, RbumCertConfIdAndExtResp, RbumCertConfModifyReq, RbumCertConfSummaryResp};
use crate::rbum::dto::rbum_cert_dto::{RbumCertAddReq, RbumCertDetailResp, RbumCertModifyReq, RbumCertSummaryResp};
use crate::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumCertConfFilterReq, RbumCertFilterReq};
use crate::rbum::rbum_config::RbumConfigApi;
use crate::rbum::rbum_enumeration::{RbumCertConfStatusKind, RbumCertRelKind, RbumCertStatusKind};
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

    async fn before_add_rbum(add_req: &mut RbumCertConfAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if let Some(is_basic) = add_req.is_basic {
            // If is_basic is true, sk_dynamic must be false
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
            Regex::new(ak_rule).map_err(|e| funs.err().bad_request(&Self::get_obj_name(), "add", &format!("ak rule is invalid:{e}"), "400-rbum-cert-conf-ak-rule-invalid"))?;
        }
        if let Some(sk_rule) = &add_req.sk_rule {
            Regex::new(sk_rule).map_err(|e| funs.err().bad_request(&Self::get_obj_name(), "add", &format!("sk rule is invalid:{e}"), "400-rbum-cert-conf-sk-rule-invalid"))?;
        }
        if funs
            .db()
            .count(
                Query::select()
                    .column(rbum_cert_conf::Column::Id)
                    .from(rbum_cert_conf::Entity)
                    .and_where(Expr::col(rbum_cert_conf::Column::Kind).eq(add_req.kind.to_string()))
                    .and_where(Expr::col(rbum_cert_conf::Column::Supplier).eq(add_req.supplier.as_ref().unwrap_or(&TrimString::from("")).to_string()))
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

    async fn package_add(add_req: &RbumCertConfAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<rbum_cert_conf::ActiveModel> {
        Ok(rbum_cert_conf::ActiveModel {
            id: Set(TardisFuns::field.nanoid()),
            kind: Set(add_req.kind.to_string()),
            supplier: Set(add_req.supplier.as_ref().unwrap_or(&TrimString("".to_string())).to_string()),
            name: Set(add_req.name.to_string()),
            note: Set(add_req.note.as_ref().unwrap_or(&"".to_string()).to_string()),
            ak_note: Set(add_req.ak_note.as_ref().unwrap_or(&"".to_string()).to_string()),
            ak_rule: Set(add_req.ak_rule.as_ref().unwrap_or(&"".to_string()).to_string()),
            sk_note: Set(add_req.sk_note.as_ref().unwrap_or(&"".to_string()).to_string()),
            sk_rule: Set(add_req.sk_rule.as_ref().unwrap_or(&"".to_string()).to_string()),
            ext: Set(add_req.ext.as_ref().unwrap_or(&"".to_string()).to_string()),
            sk_need: Set(add_req.sk_need.unwrap_or(true)),
            sk_dynamic: Set(add_req.sk_dynamic.unwrap_or(false)),
            sk_encrypted: Set(add_req.sk_encrypted.unwrap_or(false)),
            repeatable: Set(add_req.repeatable.unwrap_or(true)),
            is_basic: Set(add_req.is_basic.unwrap_or(false)),
            rest_by_kinds: Set(add_req.rest_by_kinds.as_ref().unwrap_or(&"".to_string()).to_string()),
            expire_sec: Set(add_req.expire_sec.unwrap_or(3600 * 24 * 365)),
            sk_lock_cycle_sec: Set(add_req.sk_lock_cycle_sec.unwrap_or(0)),
            sk_lock_err_times: Set(add_req.sk_lock_err_times.unwrap_or(0)),
            sk_lock_duration_sec: Set(add_req.sk_lock_duration_sec.unwrap_or(0)),
            coexist_num: Set(add_req.coexist_num.unwrap_or(1)),
            conn_uri: Set(add_req.conn_uri.as_ref().unwrap_or(&"".to_string()).to_string()),
            status: Set(add_req.status.to_int()),
            rel_rbum_domain_id: Set(add_req.rel_rbum_domain_id.to_string()),
            rel_rbum_item_id: Set(add_req.rel_rbum_item_id.as_ref().unwrap_or(&"".to_string()).to_string()),
            ..Default::default()
        })
    }

    async fn before_modify_rbum(id: &str, modify_req: &mut RbumCertConfModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        Self::check_ownership(id, funs, ctx).await?;
        if let Some(true) = modify_req.is_basic {
            let rbum_cert_conf_resp = Self::peek_rbum(id, &RbumCertConfFilterReq::default(), funs, ctx).await?;
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
        Ok(())
    }

    async fn package_modify(id: &str, modify_req: &RbumCertConfModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<rbum_cert_conf::ActiveModel> {
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
            if is_basic {
                // If is_basic is true, sk_dynamic must be false
                rbum_cert_conf.sk_dynamic = Set(false);
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
        if let Some(status) = &modify_req.status {
            rbum_cert_conf.status = Set(status.to_int());
        }
        Ok(rbum_cert_conf)
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
        if let Some(kind) = &filter.kind {
            query.and_where(Expr::col((rbum_cert_conf::Entity, rbum_cert_conf::Column::Kind)).eq(kind.to_string()));
        }
        if let Some(supplier) = &filter.supplier {
            query.and_where(Expr::col((rbum_cert_conf::Entity, rbum_cert::Column::Supplier)).eq(supplier.to_string()));
        }
        if let Some(status) = &filter.status {
            query.and_where(Expr::col((rbum_cert_conf::Entity, rbum_cert::Column::Status)).eq(status.to_int()));
        }
        if let Some(rel_rbum_domain_id) = &filter.rel_rbum_domain_id {
            query.and_where(Expr::col((rbum_cert_conf::Entity, rbum_cert_conf::Column::RelRbumDomainId)).eq(rel_rbum_domain_id.to_string()));
        }
        if let Some(rbum_item_id) = &filter.rel_rbum_item_id {
            query.and_where(Expr::col((rbum_cert_conf::Entity, rbum_cert_conf::Column::RelRbumItemId)).eq(rbum_item_id.to_string()));
        }
        if is_detail {
            query
                .expr_as(Expr::col((rbum_domain::Entity, rbum_domain::Column::Name)), Alias::new("rel_rbum_domain_name"))
                .expr_as(Expr::col((rbum_item::Entity, rbum_item::Column::Name)).if_null(""), Alias::new("rel_rbum_item_name"))
                .inner_join(
                    rbum_domain::Entity,
                    Expr::col((rbum_domain::Entity, rbum_domain::Column::Id)).equals((rbum_cert_conf::Entity, rbum_cert_conf::Column::RelRbumDomainId)),
                )
                .left_join(
                    rbum_item::Entity,
                    Expr::col((rbum_item::Entity, rbum_item::Column::Id)).equals((rbum_cert_conf::Entity, rbum_cert_conf::Column::RelRbumItemId)),
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
        ignore_status: bool,
        rbum_domain_id: &str,
        rbum_item_id: &str,
        funs: &TardisFunsInst,
    ) -> TardisResult<Option<RbumCertConfIdAndExtResp>> {
        let mut conf_info_stat = Query::select();
        conf_info_stat
            .column(rbum_cert_conf::Column::Id)
            .column(rbum_cert_conf::Column::Ext)
            .from(rbum_cert_conf::Entity)
            .and_where(Expr::col(rbum_cert_conf::Column::Kind).eq(kind))
            .and_where(Expr::col(rbum_cert_conf::Column::RelRbumDomainId).eq(rbum_domain_id))
            .and_where(Expr::col(rbum_cert_conf::Column::RelRbumItemId).eq(rbum_item_id));
        if !ignore_status {
            conf_info_stat.and_where(Expr::col(rbum_cert_conf::Column::Status).eq(RbumCertConfStatusKind::Enabled.to_int()));
        }
        if !supplier.is_empty() {
            conf_info_stat.and_where(Expr::col(rbum_cert_conf::Column::Supplier).eq(supplier));
        }
        if let Some(rbum_cert_conf_id_and_ext) = funs.db().get_dto::<RbumCertConfIdAndExtResp>(&conf_info_stat).await? {
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

    async fn before_add_rbum(add_req: &mut RbumCertAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if add_req.sk.is_some() && add_req.vcode.is_some() {
            return Err(funs.err().bad_request(&Self::get_obj_name(), "add", "sk and vcode can only have one", "400-rbum-cert-sk-vcode-only-one"));
        }
        if add_req.start_time.is_none() {
            add_req.start_time = Some(Utc::now());
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
            Self::check_ownership_with_table_name(rel_rbum_cert_conf_id, RbumCertConfServ::get_table_name(), funs, ctx).await?;

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
                    .map_err(|e| funs.err().bad_request(&Self::get_obj_name(), "add", &format!("ak rule is invalid:{e}"), "400-rbum-cert-conf-ak-rule-invalid"))?
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
                    .map_err(|e| funs.err().bad_request(&Self::get_obj_name(), "add", &format!("sk rule is invalid:{e}"), "400-rbum-cert-conf-sk-rule-invalid"))?
                    .is_match(&sk)
                    .unwrap_or(false)
                    && !add_req.ignore_check_sk
                {
                    return Err(funs.err().bad_request(
                        &Self::get_obj_name(),
                        "add",
                        &format!("sk {} is not match sk rule", &sk),
                        "400-rbum-cert-conf-sk-rule-not-match",
                    ));
                }
            }

            if funs
                .db()
                .count(
                    Query::select()
                        .column(rbum_cert::Column::Id)
                        .from(rbum_cert::Entity)
                        .and_where(Expr::col(rbum_cert::Column::RelRbumKind).eq(add_req.rel_rbum_kind.to_int()))
                        .and_where(Expr::col(rbum_cert::Column::Ak).eq(add_req.ak.to_string()))
                        .and_where(Expr::col(rbum_cert::Column::RelRbumCertConfId).eq(add_req.rel_rbum_cert_conf_id.clone()))
                        .and_where(Expr::col(rbum_cert::Column::OwnPaths).like(format!("{}%", ctx.own_paths).as_str())),
                )
                .await?
                > 0
            {
                return Err(funs.err().conflict(&Self::get_obj_name(), "add", "ak is used", "409-rbum-cert-ak-duplicate"));
            }

            // Encrypt Sk
            if rbum_cert_conf.sk_encrypted {
                if let Some(sk) = &add_req.sk {
                    let sk = Self::encrypt_sk(sk, &add_req.ak, rel_rbum_cert_conf_id)?;
                    add_req.sk = Some(TrimString(sk));
                }
            }
            // Fill Time
            if add_req.end_time.is_none() {
                add_req.end_time = Some(add_req.start_time.expect("ignore") + Duration::try_seconds(rbum_cert_conf.expire_sec).unwrap_or(TimeDelta::MAX));
            }
            // Dynamic Sk do not require an expiration time
            if rbum_cert_conf.sk_dynamic {
                add_req.end_time = Some(Utc::now() + Duration::try_days(365 * 100).expect("ignore"));
            }
        } else {
            // Fill Time
            if add_req.end_time.is_none() {
                add_req.end_time = Some(add_req.start_time.expect("ignore") + Duration::try_days(365 * 100).expect("ignore"));
            }
        }

        Ok(())
    }

    async fn package_add(add_req: &RbumCertAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<rbum_cert::ActiveModel> {
        Ok(rbum_cert::ActiveModel {
            id: Set(TardisFuns::field.nanoid()),
            ak: Set(add_req.ak.to_string()),
            sk: Set(add_req.sk.as_ref().unwrap_or(&TrimString("".to_string())).to_string()),
            sk_invisible: Set(add_req.sk_invisible.unwrap_or(false)),
            kind: Set(add_req.kind.as_ref().unwrap_or(&"".to_string()).to_string()),
            supplier: Set(add_req.supplier.as_ref().unwrap_or(&"".to_string()).to_string()),
            ext: Set(add_req.ext.as_ref().unwrap_or(&"".to_string()).to_string()),
            start_time: Set(add_req.start_time.expect("ignore")),
            end_time: Set(add_req.end_time.expect("ignore")),
            conn_uri: Set(add_req.conn_uri.as_ref().unwrap_or(&"".to_string()).to_string()),
            status: Set(add_req.status.to_int()),
            rel_rbum_cert_conf_id: Set(add_req.rel_rbum_cert_conf_id.as_ref().unwrap_or(&"".to_string()).to_string()),
            rel_rbum_kind: Set(add_req.rel_rbum_kind.to_int()),
            rel_rbum_id: Set(add_req.rel_rbum_id.to_string()),
            ..Default::default()
        })
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
            if !rel_rbum_cert_conf_id.is_empty() {
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
                    let need_delete_rbum_cert_ids = funs
                        .db()
                        .find_dtos::<IdResp>(
                            Query::select()
                                .column(rbum_cert::Column::Id)
                                .from(rbum_cert::Entity)
                                .and_where(Expr::col(rbum_cert::Column::RelRbumCertConfId).eq(rel_rbum_cert_conf_id))
                                .and_where(Expr::col(rbum_cert::Column::RelRbumKind).eq(add_req.rel_rbum_kind.to_int()))
                                .and_where(Expr::col(rbum_cert::Column::RelRbumId).eq(&add_req.rel_rbum_id))
                                .order_by((rbum_cert::Entity, rbum_cert::Column::CreateTime), Order::Desc)
                                .offset(rbum_cert_conf.coexist_num as u64),
                        )
                        .await?;
                    for need_delete_rbum_cert_id in need_delete_rbum_cert_ids {
                        Self::delete_rbum(&need_delete_rbum_cert_id.id, funs, ctx).await?;
                    }
                }
            }
            if let Some(vcode) = &add_req.vcode {
                // here we don't add cool down limit for vcode
                Self::add_vcode_to_cache(&add_req.ak, vcode, rel_rbum_cert_conf_id, None, funs, ctx).await?;
            }
        }
        Ok(())
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
            if !rel_rbum_cert_conf_id.is_empty() {
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

                if let Some(ak) = &modify_req.ak {
                    if !rbum_cert_conf.ak_rule.is_empty()
                        && !Regex::new(&rbum_cert_conf.ak_rule)
                            .map_err(|e| funs.err().bad_request(&Self::get_obj_name(), "modify", &format!("ak rule is invalid:{e}"), "400-rbum-cert-conf-ak-rule-invalid"))?
                            .is_match(ak.as_ref())
                            .unwrap_or(false)
                    {
                        return Err(funs.err().bad_request(
                            &Self::get_obj_name(),
                            "modify",
                            &format!("ak {ak} is not match ak rule"),
                            "400-rbum-cert-conf-ak-rule-not-match",
                        ));
                    }
                }
                if let Some(sk) = &modify_req.sk {
                    if rbum_cert_conf.sk_need
                        && !rbum_cert_conf.sk_rule.is_empty()
                        && !Regex::new(&rbum_cert_conf.sk_rule)
                            .map_err(|e| funs.err().bad_request(&Self::get_obj_name(), "modify", &format!("sk rule is invalid:{e}"), "400-rbum-cert-conf-sk-rule-invalid"))?
                            .is_match(sk.as_ref())
                            .unwrap_or(false)
                        && !modify_req.ignore_check_sk
                    {
                        return Err(funs.err().bad_request(
                            &Self::get_obj_name(),
                            "modify",
                            &format!("sk {sk} is not match sk rule"),
                            "400-rbum-cert-conf-sk-rule-not-match",
                        ));
                    }
                }
                if modify_req.ak.is_some()
                    && funs
                        .db()
                        .count(
                            Query::select()
                                .column(rbum_cert::Column::Id)
                                .from(rbum_cert::Entity)
                                .and_where(Expr::col(rbum_cert::Column::Ak).eq(modify_req.ak.as_ref().expect("ignore").to_string()))
                                .and_where(Expr::col(rbum_cert::Column::RelRbumCertConfId).eq(rbum_cert_conf.id.clone()))
                                .and_where(Expr::col(rbum_cert::Column::OwnPaths).like(format!("{}%", ctx.own_paths).as_str()))
                                .and_where(Expr::col(rbum_cert::Column::Id).ne(id.to_string().as_str())),
                        )
                        .await?
                        > 0
                {
                    return Err(funs.err().conflict(&Self::get_obj_name(), "modify", "ak is used", "409-rbum-cert-ak-duplicate"));
                }

                // Encrypt Sk
                if (modify_req.sk.is_some() || modify_req.ak.is_some()) && rbum_cert_conf.sk_encrypted {
                    if modify_req.ak.is_some() && modify_req.sk.is_none() {
                        return Err(funs.err().conflict(&Self::get_obj_name(), "modify", "sk cannot be empty", "409-rbum-cert-ak-duplicate"));
                    }
                    if let Some(sk) = &modify_req.sk {
                        let sk = Self::encrypt_sk(sk, modify_req.ak.as_ref().unwrap_or(&TrimString(rbum_cert.ak)).as_ref(), rel_rbum_cert_conf_id)?;
                        modify_req.sk = Some(TrimString(sk));
                    }
                }
            }
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
        if let Some(sk_invisible) = &modify_req.sk_invisible {
            rbum_cert.sk_invisible = Set(*sk_invisible);
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

    async fn package_query(is_detail: bool, filter: &RbumCertFilterReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_cert::Entity, rbum_cert::Column::Id),
                (rbum_cert::Entity, rbum_cert::Column::Kind),
                (rbum_cert::Entity, rbum_cert::Column::Supplier),
                (rbum_cert::Entity, rbum_cert::Column::Ak),
                (rbum_cert::Entity, rbum_cert::Column::SkInvisible),
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
                Expr::col((rbum_cert_conf::Entity, rbum_cert_conf::Column::Name)).if_null(""),
                Alias::new("rel_rbum_cert_conf_name"),
            )
            .from(rbum_cert::Entity)
            .left_join(
                rbum_cert_conf::Entity,
                Expr::col((rbum_cert_conf::Entity, rbum_cert_conf::Column::Id)).equals((rbum_cert::Entity, rbum_cert::Column::RelRbumCertConfId)),
            );
        if let Some(id) = &filter.id {
            query.and_where(Expr::col((rbum_cert::Entity, rbum_cert::Column::Id)).eq(id.to_string()));
        }
        if let Some(ak) = &filter.ak {
            query.and_where(Expr::col((rbum_cert::Entity, rbum_cert::Column::Ak)).eq(ak.to_string()));
        }
        if let Some(ak) = &filter.ak_like {
            query.and_where(Expr::col((rbum_cert::Entity, rbum_cert::Column::Ak)).like(format!("{ak}%")));
        }
        if let Some(kind) = &filter.kind {
            query.and_where(Expr::col((rbum_cert::Entity, rbum_cert::Column::Kind)).eq(kind.to_string()));
        }
        if let Some(supplier) = &filter.suppliers {
            query.and_where(Expr::col((rbum_cert::Entity, rbum_cert::Column::Supplier)).is_in::<&str, Vec<&str>>(supplier.iter().map(|s| &s[..]).collect()));
        }
        if let Some(status) = &filter.status {
            query.and_where(Expr::col((rbum_cert::Entity, rbum_cert::Column::Status)).eq(status.to_int()));
        }
        if let Some(ext) = &filter.ext {
            query.and_where(Expr::col((rbum_cert::Entity, rbum_cert::Column::Ext)).eq(ext.to_string()));
        }
        if let Some(rel_rbum_cert_conf_ids) = &filter.rel_rbum_cert_conf_ids {
            query.and_where(Expr::col((rbum_cert::Entity, rbum_cert::Column::RelRbumCertConfId)).is_in(rel_rbum_cert_conf_ids.clone()));
        }
        if let Some(rel_rbum_kind) = &filter.rel_rbum_kind {
            query.and_where(Expr::col((rbum_cert::Entity, rbum_cert::Column::RelRbumKind)).eq(rel_rbum_kind.to_int()));
        }
        if let Some(rel_rbum_id) = &filter.rel_rbum_id {
            query.and_where(Expr::col((rbum_cert::Entity, rbum_cert::Column::RelRbumId)).eq(rel_rbum_id.to_string()));
        }
        if let Some(rel_rbum_ids) = &filter.rel_rbum_ids {
            query.and_where(Expr::col((rbum_cert::Entity, rbum_cert::Column::RelRbumId)).is_in(rel_rbum_ids.clone()));
        }
        query.with_filter(Self::get_table_name(), &filter.basic, is_detail, false, ctx);
        Ok(query)
    }
}

impl RbumCertServ {
    /// Add dynamic sk（verification code） to cache
    ///
    ///
    /// 添加动态sk（验证码）到缓存
    pub async fn add_vcode_to_cache(ak: &str, vcode: &str, cert_conf_id: &str, cool_down_in_sec: Option<u32>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let rbum_cert_conf = RbumCertConfServ::peek_rbum(cert_conf_id, &RbumCertConfFilterReq::default(), funs, ctx).await?;
        funs.cache()
            .set_ex(
                format!("{}{}:{}", funs.rbum_conf_cache_key_cert_vcode_info_(), &ctx.own_paths, ak).as_str(),
                vcode.to_string().as_str(),
                rbum_cert_conf.expire_sec as u64,
            )
            .await?;
        // check vcode cool down
        let cool_down_key = format!("{}{}:{}:cd", funs.rbum_conf_cache_key_cert_vcode_info_(), &ctx.own_paths, ak);
        let ttl: i32 = funs.cache().cmd().await?.ttl(&cool_down_key).await?;
        if ttl > 0 {
            let message = format!("vcode send still cooling down until {} secs latter", ttl);
            return Err(funs.err().bad_request(&Self::get_obj_name(), "add_vcode_to_cache", &message, "400-rbum-cert-vcode-cool-down"));
        } else if let Some(cool_down_in_sec) = cool_down_in_sec {
            // set cool down key
            funs.cache().set_ex(&cool_down_key, "", cool_down_in_sec as u64).await?;
        }

        Ok(())
    }

    /// Get dynamic sk（verification code） from cache
    ///
    /// 从缓存中获取动态sk（验证码）
    pub async fn get_vcode_in_cache(ak: &str, own_paths: &str, funs: &TardisFunsInst) -> TardisResult<Option<String>> {
        let vcode = funs.cache().get(format!("{}{}:{}", funs.rbum_conf_cache_key_cert_vcode_info_(), own_paths, ak).as_str()).await?;
        Ok(vcode)
    }

    /// Get and delete dynamic sk（verification code） from cache
    ///
    /// 从缓存中获取并删除动态sk（验证码）
    pub async fn get_and_delete_vcode_in_cache(ak: &str, own_paths: &str, funs: &TardisFunsInst) -> TardisResult<Option<String>> {
        let vcode = funs.cache().get(format!("{}{}:{}", funs.rbum_conf_cache_key_cert_vcode_info_(), own_paths, ak).as_str()).await?;
        if vcode.is_some() {
            funs.cache().del(format!("{}{}:{}", funs.rbum_conf_cache_key_cert_vcode_info_(), own_paths, ak).as_str()).await?;
        }
        Ok(vcode)
    }

    /// Check whether the certificate is exist
    ///
    /// 检查凭证是否存在
    pub async fn check_exist(ak: &str, rbum_cert_conf_id: &str, own_paths: &str, funs: &TardisFunsInst) -> TardisResult<bool> {
        let mut query = Query::select();
        query
            .column(rbum_cert::Column::Id)
            .from(rbum_cert::Entity)
            .and_where(Expr::col(rbum_cert::Column::Ak).eq(ak))
            .and_where(Expr::col(rbum_cert::Column::RelRbumCertConfId).eq(rbum_cert_conf_id))
            .and_where(Expr::col(rbum_cert::Column::OwnPaths).eq(own_paths))
            .and_where(Expr::col(rbum_cert::Column::Status).eq(RbumCertStatusKind::Enabled.to_int()));
        funs.db().count(&query).await.map(|r| r > 0)
    }

    /// Validate the validity of the certificate according to the specified certificate configuration
    ///
    /// 根据指定的凭证配置，验证凭证的合法性
    ///
    /// # Parameters
    /// - `ak` - Access key
    /// - `input_sk` - Secret key
    /// - `rbum_cert_conf_id` - Certificate configuration id
    /// - `ignore_end_time` - Whether to ignore the expiration time
    /// - `own_paths` - Own paths
    /// - `funs` - TardisFunsInst
    ///
    /// # Returns
    /// - (the certificate id, certificate relationship type, and certificate relationship id)
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
            pub sk_lock_cycle_sec: i32,
            pub sk_lock_err_times: i16,
            pub sk_lock_duration_sec: i32,
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
            .and_where(Expr::col(rbum_cert::Column::Status).ne(RbumCertStatusKind::Disabled.to_int()))
            .and_where(Expr::col(rbum_cert::Column::StartTime).lte(Utc::now().naive_utc()));
        let rbum_cert = funs.db().get_dto::<IdAndSkResp>(&query).await?;
        if let Some(rbum_cert) = rbum_cert {
            if Self::cert_is_locked(&rbum_cert.rel_rbum_id, funs).await? {
                return Err(funs.err().error("400-rbum-cert-lock", &Self::get_obj_name(), "valid", "cert is locked", "400-rbum-cert-lock"));
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
                if let Some(cached_vcode) = Self::get_vcode_in_cache(ak, own_paths, funs).await? {
                    cached_vcode
                } else {
                    log::warn!(
                        "validation error [vcode is not exist] by ak {},rbum_cert_conf_id {}, own_paths {}",
                        ak,
                        rbum_cert_conf_id,
                        own_paths
                    );
                    Self::after_validate_fail(
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
                Self::after_validate_success(&rbum_cert.rel_rbum_id, funs).await?;
                Ok((rbum_cert.id, rbum_cert.rel_rbum_kind, rbum_cert.rel_rbum_id))
            } else {
                log::warn!(
                    "validation error [sk is not match] by ak {},rbum_cert_conf_id {}, own_paths {}",
                    ak,
                    rbum_cert_conf_id,
                    own_paths
                );
                Self::after_validate_fail(
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

    /// Validate the validity of the certificate using the current or basic certificate configuration
    ///
    /// 使用当前或基础凭证配置，验证凭证的合法性
    ///
    /// # Parameters
    /// - `ak` - Access key
    /// - `input_sk` - Secret key
    /// - `rel_rbum_kind` - Certificate relationship type
    /// - `ignore_end_time` - Whether to ignore the expiration time
    /// - `own_paths` - Own paths
    /// - `allowed_kinds` - Allowed certificate configuration types
    /// - `funs` - TardisFunsInst
    ///
    /// # Returns
    /// - (the certificate id, certificate relationship type, and certificate relationship id)
    pub async fn validate_by_ak_and_basic_sk(
        ak: &str,
        input_sk: &str,
        rel_rbum_kind: &RbumCertRelKind,
        ignore_end_time: bool,
        own_paths: Option<String>,
        allowed_kinds: Vec<&str>,
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
            pub rel_rbum_item_id: String,
            pub sk_lock_cycle_sec: i32,
            pub sk_lock_err_times: i16,
            pub sk_lock_duration_sec: i32,
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
            // Exclude disabled state, have enabled and pending
            .and_where(Expr::col(rbum_cert::Column::Status).ne(RbumCertStatusKind::Disabled.to_int()))
            .and_where(Expr::col(rbum_cert::Column::StartTime).lte(Utc::now().naive_utc()))
            //basic sk must have cert conf
            .and_where(Expr::col(rbum_cert::Column::RelRbumCertConfId).ne(""));
        if let Some(own_paths) = own_paths.clone() {
            query.and_where(Expr::col(rbum_cert::Column::OwnPaths).eq(own_paths));
        }
        let rbum_cert = funs.db().get_dto::<IdAndSkResp>(&query).await?;
        if let Some(rbum_cert) = rbum_cert {
            if Self::cert_is_locked(&rbum_cert.rel_rbum_id, funs).await? {
                return Err(funs.err().unauthorized(&Self::get_obj_name(), "valid_lock", "cert is locked", "401-rbum-cert-lock"));
            }
            if !ignore_end_time && rbum_cert.end_time < Utc::now() {
                return Err(funs.err().conflict(&Self::get_obj_name(), "valid", "sk is expired", "409-rbum-cert-sk-expire"));
            }
            if !rbum_cert.rel_rbum_cert_conf_id.is_empty() {
                let cert_conf_peek_resp = funs
                    .db()
                    .get_dto::<CertConfPeekResp>(
                        Query::select()
                            .column(rbum_cert_conf::Column::IsBasic)
                            .column(rbum_cert_conf::Column::RelRbumDomainId)
                            .column(rbum_cert_conf::Column::RelRbumItemId)
                            .column(rbum_cert_conf::Column::SkEncrypted)
                            .column(rbum_cert_conf::Column::SkLockCycleSec)
                            .column(rbum_cert_conf::Column::SkLockErrTimes)
                            .column(rbum_cert_conf::Column::SkLockDurationSec)
                            .from(rbum_cert_conf::Entity)
                            .and_where(Expr::col(rbum_cert_conf::Column::Id).eq(rbum_cert.rel_rbum_cert_conf_id.as_str()))
                            .and_where(Expr::col(rbum_cert_conf::Column::Kind).is_in(allowed_kinds)),
                    )
                    .await?
                    .ok_or_else(|| funs.err().not_found(&Self::get_obj_name(), "valid", "not found cert conf", "404-rbum-cert-conf-not-exist"))?;
                let verify_input_sk = if cert_conf_peek_resp.sk_encrypted {
                    Self::encrypt_sk(input_sk, ak, rbum_cert.rel_rbum_cert_conf_id.as_str())?
                } else {
                    input_sk.to_string()
                };
                if rbum_cert.sk == verify_input_sk {
                    Self::after_validate_success(&rbum_cert.rel_rbum_id, funs).await?;
                    Ok((rbum_cert.id, rel_rbum_kind.clone(), rbum_cert.rel_rbum_id))
                } else if !cert_conf_peek_resp.is_basic {
                    // Ok(...) ?
                    Ok(Self::validate_by_non_basic_cert_conf_with_basic_sk(
                        rbum_cert.rel_rbum_id.as_str(),
                        cert_conf_peek_resp.rel_rbum_domain_id.as_str(),
                        cert_conf_peek_resp.rel_rbum_item_id.as_str(),
                        input_sk,
                        ignore_end_time,
                        funs,
                    )
                    .await?)
                } else {
                    log::warn!(
                        "validation error [sk is not match] by ak {},rel_rbum_cert_conf_id {}, own_paths {:?}",
                        ak,
                        rbum_cert.rel_rbum_cert_conf_id,
                        own_paths
                    );
                    Self::after_validate_fail(
                        &rbum_cert.rel_rbum_id,
                        cert_conf_peek_resp.sk_lock_cycle_sec,
                        cert_conf_peek_resp.sk_lock_err_times,
                        cert_conf_peek_resp.sk_lock_duration_sec,
                        funs,
                    )
                    .await?;
                    Err(funs.err().unauthorized(&Self::get_obj_name(), "valid", "validation error", "401-rbum-usrpwd-cert-valid-error"))
                }
            } else {
                log::warn!("validation error by ak {},rbum_cert_conf_id is None, own_paths {:?}", ak, own_paths);
                Err(funs.err().unauthorized(&Self::get_obj_name(), "valid", "validation error", "401-rbum-usrpwd-cert-valid-error"))
            }
        } else {
            log::warn!("validation error by ak {},rel_rbum_kind {}, own_paths {:?}", ak, rel_rbum_kind, own_paths);
            Err(funs.err().unauthorized(&Self::get_obj_name(), "valid", "validation error", "401-rbum-usrpwd-cert-valid-error"))
        }
    }

    /// Validate the validity of the certificate using the basic certificate configuration
    ///
    /// 使用基础凭证配置，验证凭证的合法性
    ///
    /// # Parameters
    /// - `cert_rel_rbum_id` - Certificate relationship id
    /// - `cert_conf_rel_rbum_domain_id` - Certificate configuration relationship domain id
    /// - `cert_conf_rel_rbum_item_id` - Certificate configuration relationship item id
    /// - `input_sk` - Secret key
    /// - `ignore_end_time` - Whether to ignore the expiration time
    /// - `funs` - TardisFunsInst
    ///
    /// # Returns
    /// - (the certificate id, certificate relationship type, and certificate relationship id)
    async fn validate_by_non_basic_cert_conf_with_basic_sk(
        cert_rel_rbum_id: &str,
        cert_conf_rel_rbum_domain_id: &str,
        cert_conf_rel_rbum_item_id: &str,
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
            pub sk_lock_cycle_sec: i32,
            pub sk_lock_err_times: i16,
            pub sk_lock_duration_sec: i32,
        }
        let rbum_basic_cert_info_resp = funs
            .db()
            .get_dto::<BasicCertInfoResp>(
                Query::select()
                    .expr_as(Expr::col((rbum_cert::Entity, rbum_cert::Column::Id)).if_null(""), Alias::new("id"))
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
                        Expr::col((rbum_cert_conf::Entity, rbum_cert_conf::Column::Id)).equals((rbum_cert::Entity, rbum_cert::Column::RelRbumCertConfId)),
                    )
                    .and_where(Expr::col(rbum_cert::Column::RelRbumId).eq(cert_rel_rbum_id))
                    .and_where(Expr::col((rbum_cert_conf::Entity, rbum_cert_conf::Column::RelRbumDomainId)).eq(cert_conf_rel_rbum_domain_id))
                    .and_where(Expr::col((rbum_cert_conf::Entity, rbum_cert_conf::Column::RelRbumItemId)).eq(cert_conf_rel_rbum_item_id))
                    .and_where(Expr::col((rbum_cert_conf::Entity, rbum_cert_conf::Column::IsBasic)).eq(true)),
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
            Self::after_validate_success(cert_rel_rbum_id, funs).await?;
            Ok((rbum_basic_cert_info_resp.id, rbum_basic_cert_info_resp.rel_rbum_kind, cert_rel_rbum_id.to_string()))
        } else {
            log::warn!(
                "validation error [sk is not match] by ak {},rbum_cert_conf_id {}, rel_rbum_id {}",
                rbum_basic_cert_info_resp.ak,
                rbum_basic_cert_info_resp.rel_rbum_cert_conf_id,
                cert_rel_rbum_id
            );
            Self::after_validate_fail(
                cert_rel_rbum_id,
                rbum_basic_cert_info_resp.sk_lock_cycle_sec,
                rbum_basic_cert_info_resp.sk_lock_err_times,
                rbum_basic_cert_info_resp.sk_lock_duration_sec,
                funs,
            )
            .await?;
            Err(funs.err().unauthorized(&Self::get_obj_name(), "valid", "basic validation error", "401-rbum-cert-valid-error"))
        }
    }

    /// Show sk
    ///
    /// 显示sk
    pub async fn show_sk(id: &str, filter: &RbumCertFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        #[derive(sea_orm::FromQueryResult)]
        struct SkResp {
            pub sk: String,
        }
        let mut query = Query::select();
        query.column((rbum_cert::Entity, rbum_cert::Column::Sk)).from(rbum_cert::Entity).and_where(Expr::col((rbum_cert::Entity, rbum_cert::Column::Id)).eq(id)).with_filter(
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

    /// Reset sk
    ///
    /// 重置sk
    pub async fn reset_sk(id: &str, new_sk: &str, is_ignore_check_sk: bool, filter: &RbumCertFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let rbum_cert = Self::peek_rbum(id, filter, funs, ctx).await?;
        let mut repeatable = true;
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
            repeatable = rbum_cert_conf.repeatable;
            if !rbum_cert_conf.sk_rule.is_empty()
                && !Regex::new(&rbum_cert_conf.sk_rule)
                    .map_err(|e| funs.err().bad_request(&Self::get_obj_name(), "reset_sk", &format!("sk rule is invalid:{e}"), "400-rbum-cert-conf-sk-rule-invalid"))?
                    .is_match(new_sk)
                    .unwrap_or(false)
                && !is_ignore_check_sk
            {
                return Err(funs.err().bad_request(
                    &Self::get_obj_name(),
                    "reset_sk",
                    &format!("sk {new_sk} is not match sk rule"),
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
        let stored_sk = Self::show_sk(id, filter, funs, ctx).await?;
        if new_sk == stored_sk && !repeatable {
            return Err(funs.err().bad_request(&Self::get_obj_name(), "reset_sk", &format!("sk {new_sk} is duplicate"), "400-rbum-cert-reset-sk-duplicate"));
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

    /// Change sk
    ///
    /// 更改sk
    pub async fn change_sk(id: &str, original_sk: &str, input_sk: &str, filter: &RbumCertFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let rbum_cert = Self::peek_rbum(id, filter, funs, ctx).await?;
        let stored_sk = Self::show_sk(id, filter, funs, ctx).await?;
        if input_sk.to_lowercase().contains(rbum_cert.ak.to_lowercase().as_str()) {
            return Err(funs.err().bad_request(&Self::get_obj_name(), "change_sk", "sk can not contain ak", "400-rbum-cert-sk-contains-ak"));
        }
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
                    .map_err(|e| funs.err().bad_request(&Self::get_obj_name(), "change_sk", &format!("sk rule is invalid:{e}"), "400-rbum-cert-conf-sk-rule-invalid"))?
                    .is_match(input_sk)
                    .unwrap_or(false)
            {
                return Err(funs.err().bad_request(
                    &Self::get_obj_name(),
                    "change_sk",
                    &format!("sk {input_sk} is not match sk rule"),
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
                    &format!("sk {input_sk} cannot be duplicated"),
                    "400-rbum-cert-ak-duplicate",
                ));
            }
            let end_time = Utc::now() + Duration::try_seconds(rbum_cert_conf.expire_sec).unwrap_or(TimeDelta::MAX);
            (new_sk, end_time)
        } else {
            if original_sk != stored_sk {
                return Err(funs.err().unauthorized(&Self::get_obj_name(), "change_sk", "sk not match", "401-rbum-cert-ori-sk-not-match"));
            }
            (input_sk.to_string(), rbum_cert.start_time + (rbum_cert.end_time - rbum_cert.start_time))
        };
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

    /// Encrypt sk
    ///
    /// 加密sk
    fn encrypt_sk(sk: &str, ak: &str, rbum_cert_conf_id: &str) -> TardisResult<String> {
        TardisFuns::crypto.digest.sha512(format!("{sk}-{ak}-{rbum_cert_conf_id}").as_str())
    }

    /// Processing logic after verification is successful
    ///
    /// 验证成功后的处理逻辑
    async fn after_validate_success(rbum_item_id: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        funs.cache().del(&format!("{}{}", funs.rbum_conf_cache_key_cert_err_times_(), rbum_item_id)).await?;
        Ok(())
    }

    /// Processing logic after verification fails
    ///
    /// 验证失败后的处理逻辑
    async fn after_validate_fail(rbum_item_id: &str, sk_lock_cycle_sec: i32, sk_lock_err_times: i16, sk_lock_duration_sec: i32, funs: &TardisFunsInst) -> TardisResult<()> {
        if sk_lock_cycle_sec == 0 || sk_lock_err_times == 0 || sk_lock_duration_sec == 0 {
            return Ok(());
        }
        let err_times = funs.cache().incr(&format!("{}{}", funs.rbum_conf_cache_key_cert_err_times_(), rbum_item_id), 1).await?;
        if sk_lock_err_times <= err_times as i16 {
            funs.cache().set_ex(&format!("{}{}", funs.rbum_conf_cache_key_cert_locked_(), rbum_item_id), "", sk_lock_duration_sec as u64).await?;
            funs.cache().del(&format!("{}{}", funs.rbum_conf_cache_key_cert_err_times_(), rbum_item_id)).await?;
        } else if err_times == 1 {
            funs.cache().expire(&format!("{}{}", funs.rbum_conf_cache_key_cert_err_times_(), rbum_item_id), sk_lock_cycle_sec as i64).await?;
        }
        Ok(())
    }

    /// Check whether the certificate is locked
    ///
    /// 检查证书是否被锁定
    pub async fn cert_is_locked(rel_rbum_id: &str, funs: &TardisFunsInst) -> TardisResult<bool> {
        let result = funs
            .cache()
            .exists(&format!("{}{}", funs.rbum_conf_cache_key_cert_locked_(), rel_rbum_id))
            .await
            .map_err(|e| funs.err().unauthorized(&Self::get_obj_name(), "cert_is_locked", &e.to_string(), "400-rbum-cert-lock"))?;
        Ok(result)
    }
}
