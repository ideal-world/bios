use async_trait::async_trait;
use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::error::TardisError;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::chrono::{Duration, Utc};
use tardis::db::reldb_client::IdResp;
use tardis::db::sea_orm::*;
use tardis::db::sea_query::*;
use tardis::regex::Regex;
use tardis::{log, TardisFuns};

use crate::rbum::domain::{rbum_cert, rbum_cert_conf, rbum_domain, rbum_item};
use crate::rbum::dto::rbum_cert_conf_dto::{RbumCertConfAddReq, RbumCertConfDetailResp, RbumCertConfModifyReq, RbumCertConfSummaryResp};
use crate::rbum::dto::rbum_cert_dto::{RbumCertAddReq, RbumCertDetailResp, RbumCertModifyReq, RbumCertSummaryResp};
use crate::rbum::dto::rbum_filer_dto::{RbumCertConfFilterReq, RbumCertFilterReq};
use crate::rbum::rbum_enumeration::{RbumCertRelKind, RbumCertStatusKind};
use crate::rbum::serv::rbum_crud_serv::{RbumCrudOperation, RbumCrudQueryPackage};
use crate::rbum::serv::rbum_domain_serv::RbumDomainServ;
use crate::rbum::serv::rbum_item_serv::RbumItemServ;
use crate::rbum::serv::rbum_rel_serv::RbumRelServ;
use crate::rbum::serv::rbum_set_serv::RbumSetServ;

pub struct RbumCertConfServ;
pub struct RbumCertServ;

#[async_trait]
impl<'a> RbumCrudOperation<'a, rbum_cert_conf::ActiveModel, RbumCertConfAddReq, RbumCertConfModifyReq, RbumCertConfSummaryResp, RbumCertConfDetailResp, RbumCertConfFilterReq>
    for RbumCertConfServ
{
    fn get_table_name() -> &'static str {
        rbum_cert_conf::Entity.table_name()
    }

    async fn package_add(add_req: &RbumCertConfAddReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<rbum_cert_conf::ActiveModel> {
        Ok(rbum_cert_conf::ActiveModel {
            id: Set(TardisFuns::field.nanoid()),
            code: Set(add_req.code.to_string()),
            name: Set(add_req.name.to_string()),
            note: Set(add_req.note.as_ref().unwrap_or(&"".to_string()).to_string()),
            ak_note: Set(add_req.ak_note.as_ref().unwrap_or(&"".to_string()).to_string()),
            ak_rule: Set(add_req.ak_rule.as_ref().unwrap_or(&"".to_string()).to_string()),
            sk_note: Set(add_req.sk_note.as_ref().unwrap_or(&"".to_string()).to_string()),
            sk_rule: Set(add_req.sk_rule.as_ref().unwrap_or(&"".to_string()).to_string()),
            sk_need: Set(add_req.sk_need.unwrap_or(true)),
            sk_encrypted: Set(add_req.sk_encrypted.unwrap_or(false)),
            repeatable: Set(add_req.repeatable.unwrap_or(true)),
            is_basic: Set(add_req.is_basic.unwrap_or(true)),
            rest_by_kinds: Set(add_req.rest_by_kinds.as_ref().unwrap_or(&"".to_string()).to_string()),
            expire_sec: Set(add_req.expire_sec.unwrap_or(u32::MAX)),
            coexist_num: Set(add_req.coexist_num.unwrap_or(1)),
            conn_uri: Set(add_req.conn_uri.as_ref().unwrap_or(&"".to_string()).to_string()),
            rel_rbum_domain_id: Set(add_req.rel_rbum_domain_id.to_string()),
            rel_rbum_item_id: Set(add_req.rel_rbum_item_id.as_ref().unwrap_or(&"".to_string()).to_string()),
            ..Default::default()
        })
    }

    async fn before_add_rbum(add_req: &mut RbumCertConfAddReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::check_scope(&add_req.rel_rbum_domain_id, RbumDomainServ::get_table_name(), funs, cxt).await?;
        if let Some(rel_rbum_item_id) = &add_req.rel_rbum_item_id {
            Self::check_scope(rel_rbum_item_id, RbumItemServ::get_table_name(), funs, cxt).await?;
        }
        if let Some(ak_rule) = &add_req.ak_rule {
            Regex::new(ak_rule).map_err(|e| TardisError::BadRequest(format!("ak rule is invalid:{}", e)))?;
        }
        if let Some(sk_rule) = &add_req.sk_rule {
            Regex::new(sk_rule).map_err(|e| TardisError::BadRequest(format!("sk rule is invalid:{}", e)))?;
        }
        if funs
            .db()
            .count(
                Query::select()
                    .column(rbum_cert_conf::Column::Id)
                    .from(rbum_cert_conf::Entity)
                    .and_where(Expr::col(rbum_cert_conf::Column::Code).eq(add_req.code.0.as_str()))
                    .and_where(Expr::col(rbum_cert_conf::Column::RelRbumDomainId).eq(add_req.rel_rbum_domain_id.as_str()))
                    .and_where(Expr::col(rbum_cert_conf::Column::RelRbumDomainId).eq(add_req.rel_rbum_item_id.as_ref().unwrap_or(&"".to_string()).as_str())),
            )
            .await?
            > 0
        {
            return Err(TardisError::BadRequest(format!("Code {} already exists", add_req.code)));
        }
        Ok(())
    }

    async fn package_modify(id: &str, modify_req: &RbumCertConfModifyReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<rbum_cert_conf::ActiveModel> {
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
        }
        if let Some(rest_by_kinds) = &modify_req.rest_by_kinds {
            rbum_cert_conf.rest_by_kinds = Set(rest_by_kinds.to_string());
        }
        if let Some(expire_sec) = modify_req.expire_sec {
            rbum_cert_conf.expire_sec = Set(expire_sec);
        }
        if let Some(coexist_num) = modify_req.coexist_num {
            rbum_cert_conf.coexist_num = Set(coexist_num);
        }
        if let Some(conn_uri) = &modify_req.conn_uri {
            rbum_cert_conf.conn_uri = Set(conn_uri.to_string());
        }
        Ok(rbum_cert_conf)
    }

    async fn before_delete_rbum(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        Self::check_ownership(id, funs, cxt).await?;
        Self::check_exist_before_delete(id, RbumCertServ::get_table_name(), rbum_cert::Column::RelRbumCertConfId.as_str(), funs).await?;
        Ok(())
    }

    async fn package_query(is_detail: bool, filter: &RbumCertConfFilterReq, _: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::Id),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::Code),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::Name),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::Note),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::AkNote),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::AkRule),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::SkNote),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::SkRule),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::SkNeed),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::SkEncrypted),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::Repeatable),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::IsBasic),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::RestByKinds),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::ExpireSec),
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
        query.with_filter(Self::get_table_name(), &filter.basic, is_detail, false, cxt);
        Ok(query)
    }
}

impl RbumCertConfServ {
    pub async fn get_rbum_cert_conf_id_by_code<'a>(code: &str, rbum_domain_id: &str, rbum_item_id: &str, funs: &TardisFunsInst<'a>) -> TardisResult<Option<String>> {
        let resp = funs
            .db()
            .get_dto::<IdResp>(
                Query::select()
                    .column(rbum_cert_conf::Column::Id)
                    .from(rbum_cert_conf::Entity)
                    .and_where(Expr::col(rbum_cert_conf::Column::Code).eq(code))
                    .and_where(Expr::col(rbum_cert_conf::Column::RelRbumDomainId).eq(rbum_domain_id))
                    .and_where(Expr::col(rbum_cert_conf::Column::RelRbumItemId).eq(rbum_item_id)),
            )
            .await?
            .map(|r| r.id);
        Ok(resp)
    }
}

#[async_trait]
impl<'a> RbumCrudOperation<'a, rbum_cert::ActiveModel, RbumCertAddReq, RbumCertModifyReq, RbumCertSummaryResp, RbumCertDetailResp, RbumCertFilterReq> for RbumCertServ {
    fn get_table_name() -> &'static str {
        rbum_cert::Entity.table_name()
    }

    async fn package_add(add_req: &RbumCertAddReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<rbum_cert::ActiveModel> {
        Ok(rbum_cert::ActiveModel {
            id: Set(TardisFuns::field.nanoid()),
            ak: Set(add_req.ak.to_string()),
            sk: Set(add_req.sk.as_ref().unwrap_or(&TrimString("".to_string())).to_string()),
            ext: Set(add_req.ext.as_ref().unwrap_or(&"".to_string()).to_string()),
            start_time: Set(add_req.start_time.unwrap_or_else(Utc::now).naive_utc()),
            end_time: Set(add_req.end_time.unwrap_or_else(Utc::now).naive_utc()),
            status: Set(add_req.status.to_string()),
            rel_rbum_cert_conf_id: Set(add_req.rel_rbum_cert_conf_id.as_ref().unwrap_or(&"".to_string()).to_string()),
            rel_rbum_kind: Set(add_req.rel_rbum_kind.to_int()),
            rel_rbum_id: Set(add_req.rel_rbum_kind.to_string()),
            ..Default::default()
        })
    }

    async fn before_add_rbum(add_req: &mut RbumCertAddReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        if let Some(rel_rbum_cert_conf_id) = &add_req.rel_rbum_cert_conf_id {
            Self::check_ownership_with_table_name(rel_rbum_cert_conf_id, RbumCertConfServ::get_table_name(), funs, cxt).await?;
        }
        match add_req.rel_rbum_kind {
            RbumCertRelKind::Item => Self::check_scope(&add_req.rel_rbum_id, RbumItemServ::get_table_name(), funs, cxt).await?,
            RbumCertRelKind::Set => Self::check_scope(&add_req.rel_rbum_id, RbumSetServ::get_table_name(), funs, cxt).await?,
            RbumCertRelKind::Rel => Self::check_ownership_with_table_name(&add_req.rel_rbum_id, RbumRelServ::get_table_name(), funs, cxt).await?,
        }

        if let Some(rel_rbum_cert_conf_id) = &add_req.rel_rbum_cert_conf_id {
            let rbum_cert_conf = RbumCertConfServ::get_rbum(rel_rbum_cert_conf_id, &RbumCertConfFilterReq::default(), funs, cxt).await?;
            // Encrypt Sk
            RbumCertServ::check_cert_conf_constraint_by_add(add_req, &rbum_cert_conf, funs, cxt).await?;
            if rbum_cert_conf.sk_encrypted {
                if let Some(sk) = &add_req.sk {
                    let sk = Self::encrypt_sk(sk.0.as_str(), add_req.ak.0.as_str())?;
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
            // Delete Old Certs
            if rbum_cert_conf.coexist_num != 0 {
                let need_delete_rbum_certs = Self::paginate_rbums(
                    &RbumCertFilterReq {
                        rel_rbum_kind: Some(add_req.rel_rbum_kind.clone()),
                        rel_rbum_id: Some(add_req.rel_rbum_id.clone()),
                        rel_rbum_cert_conf_id: Some(rel_rbum_cert_conf_id.clone()),
                        ..Default::default()
                    },
                    // Skip normal records
                    2,
                    rbum_cert_conf.coexist_num as u64,
                    Some(true),
                    None,
                    funs,
                    cxt,
                )
                .await?;
                for need_delete_rbum_cert in need_delete_rbum_certs.records {
                    Self::delete_rbum(&need_delete_rbum_cert.id, funs, cxt).await?;
                }
            }
        }
        Ok(())
    }

    async fn package_modify(id: &str, modify_req: &RbumCertModifyReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<rbum_cert::ActiveModel> {
        let mut rbum_cert = rbum_cert::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(ext) = &modify_req.ext {
            rbum_cert.ext = Set(ext.to_string());
        }
        if let Some(start_time) = modify_req.start_time {
            rbum_cert.start_time = Set(start_time.naive_utc());
        }
        if let Some(end_time) = modify_req.end_time {
            rbum_cert.end_time = Set(end_time.naive_utc());
        }
        if let Some(status) = &modify_req.status {
            rbum_cert.status = Set(status.to_string());
        }
        Ok(rbum_cert)
    }

    async fn package_query(is_detail: bool, filter: &RbumCertFilterReq, _: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_cert::Entity, rbum_cert::Column::Id),
                (rbum_cert::Entity, rbum_cert::Column::Ak),
                (rbum_cert::Entity, rbum_cert::Column::Ext),
                (rbum_cert::Entity, rbum_cert::Column::StartTime),
                (rbum_cert::Entity, rbum_cert::Column::EndTime),
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
            .from(rbum_cert::Entity)
            .left_join(
                rbum_cert_conf::Entity,
                Expr::tbl(rbum_cert_conf::Entity, rbum_cert_conf::Column::Id).equals(rbum_cert::Entity, rbum_cert::Column::RelRbumCertConfId),
            );
        if let Some(rel_rbum_cert_conf_id) = &filter.rel_rbum_cert_conf_id {
            query.and_where(Expr::tbl(rbum_cert::Entity, rbum_cert::Column::RelRbumCertConfId).eq(rel_rbum_cert_conf_id.to_string()));
        }
        if let Some(rel_rbum_kind) = &filter.rel_rbum_kind {
            query.and_where(Expr::tbl(rbum_cert::Entity, rbum_cert::Column::RelRbumKind).eq(rel_rbum_kind.to_int()));
        }
        if let Some(rel_rbum_id) = &filter.rel_rbum_id {
            query.and_where(Expr::tbl(rbum_cert::Entity, rbum_cert::Column::RelRbumId).eq(rel_rbum_id.to_string()));
        }
        query.with_filter(Self::get_table_name(), &filter.basic, is_detail, false, cxt);
        Ok(query)
    }
}

impl<'a> RbumCertServ {
    // TODO find cert

    pub async fn validate(ak: &str, sk: &str, rbum_cert_conf_id: &str, own_paths: &str, funs: &TardisFunsInst<'a>) -> TardisResult<(String, RbumCertRelKind, String)> {
        #[derive(Debug, FromQueryResult)]
        struct IdAndSkResp {
            pub id: String,
            pub sk: String,
            pub rel_rbum_kind: RbumCertRelKind,
            pub rel_rbum_id: String,
        }

        #[derive(Debug, FromQueryResult)]
        struct SkEncryptedResp {
            pub sk_encrypted: bool,
        }

        let mut query = Query::select();
        query
            .column(rbum_cert::Column::Id)
            .column(rbum_cert::Column::Sk)
            .column(rbum_cert::Column::RelRbumKind)
            .column(rbum_cert::Column::RelRbumId)
            .from(rbum_cert::Entity)
            .and_where(Expr::col(rbum_cert::Column::Ak).eq(ak))
            .and_where(Expr::col(rbum_cert::Column::RelRbumCertConfId).eq(rbum_cert_conf_id))
            .and_where(Expr::col(rbum_cert::Column::OwnPaths).like(format!("{}%", own_paths).as_str()))
            .and_where(Expr::col(rbum_cert::Column::Status).eq(RbumCertStatusKind::Enabled.to_string()))
            .and_where(Expr::col(rbum_cert::Column::StartTime).lte(Utc::now().naive_utc()))
            .and_where(Expr::col(rbum_cert::Column::EndTime).gte(Utc::now().naive_utc()));
        let rbum_cert = funs.db().get_dto::<IdAndSkResp>(&query).await?;
        if let Some(rbum_cert) = rbum_cert {
            let sk_encrypted_resp = funs
                .db()
                .get_dto::<SkEncryptedResp>(
                    Query::select().column(rbum_cert_conf::Column::SkEncrypted).from(rbum_cert_conf::Entity).and_where(Expr::col(rbum_cert_conf::Column::Id).eq(rbum_cert_conf_id)),
                )
                .await?
                .ok_or_else(|| TardisError::NotFound("cert conf not found".to_string()))?;
            let sk = if sk_encrypted_resp.sk_encrypted { Self::encrypt_sk(sk, ak)? } else { sk.to_string() };
            if rbum_cert.sk == sk {
                Ok((rbum_cert.id, rbum_cert.rel_rbum_kind, rbum_cert.rel_rbum_id))
            } else {
                tardis::log::warn!(
                    "validation error [sk is not match] by ak {},rbum_cert_conf_id {}, own_paths {}",
                    ak,
                    rbum_cert_conf_id,
                    own_paths
                );
                Err(TardisError::Unauthorized("validation error".to_string()))
            }
        } else {
            log::warn!("validation error by ak {},rbum_cert_conf_id {}, own_paths {}", ak, rbum_cert_conf_id, own_paths);
            Err(TardisError::Unauthorized("validation error".to_string()))
        }
    }

    pub async fn show_sk(id: &str, filter: &RbumCertFilterReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<String> {
        #[derive(FromQueryResult)]
        struct SkResp {
            pub sk: String,
        }
        let mut query = Query::select();
        query.column((rbum_cert::Entity, rbum_cert::Column::Sk)).from(rbum_cert::Entity).and_where(Expr::tbl(rbum_cert::Entity, rbum_cert::Column::Id).eq(id)).with_filter(
            Self::get_table_name(),
            &filter.basic,
            false,
            false,
            cxt,
        );
        let sk_resp = funs.db().get_dto::<SkResp>(&query).await?;
        if let Some(sk_resp) = sk_resp {
            Ok(sk_resp.sk)
        } else {
            Err(TardisError::NotFound("cert record not found".to_string()))
        }
    }

    pub async fn reset_sk(id: &str, new_sk: &str, filter: &RbumCertFilterReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        let rbum_cert = Self::get_rbum(id, filter, funs, cxt).await?;
        let new_sk = if let Some(rel_rbum_cert_conf_id) = &rbum_cert.rel_rbum_cert_conf_id {
            let rbum_cert_conf = RbumCertConfServ::get_rbum(
                rel_rbum_cert_conf_id,
                &RbumCertConfFilterReq {
                    basic: filter.basic.clone(),
                    ..Default::default()
                },
                funs,
                cxt,
            )
            .await?;
            if !rbum_cert_conf.sk_rule.is_empty() && !Regex::new(&rbum_cert_conf.sk_rule)?.is_match(new_sk) {
                return Err(TardisError::BadRequest(format!("sk {} is not match sk rule", new_sk)));
            }
            if rbum_cert_conf.sk_encrypted {
                Self::encrypt_sk(new_sk, rbum_cert.ak.as_str())?
            } else {
                new_sk.to_string()
            }
        } else {
            new_sk.to_string()
        };
        funs.db()
            .update_one(
                rbum_cert::ActiveModel {
                    id: Set(id.to_string()),
                    sk: Set(new_sk),
                    ..Default::default()
                },
                cxt,
            )
            .await?;
        Ok(())
    }

    pub async fn change_sk(id: &str, original_sk: &str, new_sk: &str, filter: &RbumCertFilterReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        let rbum_cert = Self::get_rbum(id, filter, funs, cxt).await?;
        let stored_sk = Self::show_sk(id, filter, funs, cxt).await?;
        let (new_sk, end_time) = if let Some(rel_rbum_cert_conf_id) = &rbum_cert.rel_rbum_cert_conf_id {
            let rbum_cert_conf = RbumCertConfServ::get_rbum(rel_rbum_cert_conf_id, &RbumCertConfFilterReq::default(), funs, cxt).await?;
            let original_sk = if rbum_cert_conf.sk_encrypted {
                Self::encrypt_sk(original_sk, rbum_cert.ak.as_str())?
            } else {
                original_sk.to_string()
            };
            if original_sk != stored_sk {
                return Err(TardisError::Unauthorized("sk not match".to_string()));
            }
            if !rbum_cert_conf.sk_rule.is_empty() && !Regex::new(&rbum_cert_conf.sk_rule)?.is_match(new_sk) {
                return Err(TardisError::BadRequest(format!("sk {} is not match sk rule", new_sk)));
            }
            let end_time = Utc::now() + Duration::seconds(rbum_cert_conf.expire_sec as i64);
            if rbum_cert_conf.sk_encrypted {
                (Self::encrypt_sk(new_sk, rbum_cert.ak.as_str())?, end_time)
            } else {
                (new_sk.to_string(), end_time)
            }
        } else {
            if original_sk != stored_sk {
                return Err(TardisError::Unauthorized("sk not match".to_string()));
            }
            (new_sk.to_string(), rbum_cert.start_time + (rbum_cert.end_time - rbum_cert.start_time))
        };
        funs.db()
            .update_one(
                rbum_cert::ActiveModel {
                    id: Set(id.to_string()),
                    sk: Set(new_sk),
                    end_time: Set(end_time.naive_utc()),
                    ..Default::default()
                },
                cxt,
            )
            .await?;
        Ok(())
    }

    async fn check_cert_conf_constraint_by_add(
        add_req: &RbumCertAddReq,
        rbum_cert_conf: &RbumCertConfDetailResp,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<()> {
        if rbum_cert_conf.sk_need && add_req.sk.is_none() {
            return Err(TardisError::BadRequest("sk is required".to_string()));
        }
        if !rbum_cert_conf.ak_rule.is_empty() && !Regex::new(&rbum_cert_conf.ak_rule)?.is_match(&add_req.ak.to_string()) {
            return Err(TardisError::BadRequest(format!("ak {} is not match ak rule", add_req.ak)));
        }
        if rbum_cert_conf.sk_need && !rbum_cert_conf.sk_rule.is_empty() {
            let sk = add_req.sk.as_ref().ok_or_else(|| TardisError::BadRequest("sk is required".to_string()))?.to_string();
            if !Regex::new(&rbum_cert_conf.sk_rule)?.is_match(&sk) {
                return Err(TardisError::BadRequest(format!("sk {} is not match sk rule", &sk)));
            }
        }
        if funs
            .db()
            .count(
                Query::select()
                    .column(rbum_cert::Column::Id)
                    .from(rbum_cert::Entity)
                    .and_where(Expr::col(rbum_cert::Column::RelRbumCertConfId).eq(rbum_cert_conf.id.as_str()))
                    .and_where(Expr::col(rbum_cert::Column::Ak).eq(add_req.ak.0.as_str()))
                    .and_where(Expr::col(rbum_cert::Column::OwnPaths).like(format!("{}%", cxt.own_paths).as_str())),
            )
            .await?
            > 0
        {
            return Err(TardisError::BadRequest("ak is used".to_string()));
        }
        Ok(())
    }

    fn encrypt_sk(sk: &str, ak: &str) -> TardisResult<String> {
        TardisFuns::crypto.digest.sha512(format!("{}-{}", sk, ak).as_str())
    }
}
