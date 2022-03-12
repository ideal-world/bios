pub mod cert {
    use tardis::basic::dto::TardisContext;
    use tardis::basic::error::TardisError;
    use tardis::basic::result::TardisResult;
    use tardis::chrono::Utc;
    use tardis::db::reldb_client::TardisRelDBlConnection;
    use tardis::db::sea_orm::*;
    use tardis::db::sea_query::*;
    use tardis::web::web_resp::TardisPage;

    use crate::rbum::domain::{rbum_cert, rbum_cert_conf, rbum_domain, rbum_item};
    use crate::rbum::dto::filer_dto::RbumBasicFilterReq;
    use crate::rbum::dto::rbum_cert_dto::{RbumCertAddReq, RbumCertDetailResp, RbumCertModifyReq, RbumCertSummaryResp};

    pub async fn add_rbum_cert<'a>(
        rel_rbum_cert_conf_id: Option<String>,
        rel_rbum_domain_id: Option<String>,
        rel_rbum_item_id: Option<String>,
        rbum_cert_add_req: &RbumCertAddReq,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<String> {
        let rbum_cert_id: String = db
            .insert_one(
                rbum_cert::ActiveModel {
                    name: Set(rbum_cert_add_req.name.to_string()),
                    ak: Set(rbum_cert_add_req.ak.as_ref().unwrap_or(&"".to_string()).to_string()),
                    sk: Set(rbum_cert_add_req.sk.as_ref().unwrap_or(&"".to_string()).to_string()),
                    ext: Set(rbum_cert_add_req.ext.as_ref().unwrap_or(&"".to_string()).to_string()),
                    start_time: Set(rbum_cert_add_req.start_time.unwrap_or(Utc::now()).naive_utc()),
                    end_time: Set(rbum_cert_add_req.end_time.unwrap_or(Utc::now()).naive_utc()),
                    coexist_flag: Set(rbum_cert_add_req.coexist_flag.as_ref().unwrap_or(&"".to_string()).to_string()),
                    status: Set(rbum_cert_add_req.status.to_string()),
                    rel_rbum_cert_conf_id: Set(rel_rbum_cert_conf_id.unwrap_or("".to_string())),
                    rel_rbum_domain_id: Set(rel_rbum_domain_id.unwrap_or("".to_string())),
                    rel_rbum_item_id: Set(rel_rbum_item_id.unwrap_or("".to_string())),
                    ..Default::default()
                },
                cxt,
            )
            .await?
            .last_insert_id;
        Ok(rbum_cert_id)
    }

    pub async fn modify_rbum_cert<'a>(id: &str, rbum_cert_modify_req: &RbumCertModifyReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        let mut rbum_cert = rbum_cert::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(name) = &rbum_cert_modify_req.name {
            rbum_cert.name = Set(name.to_string());
        }
        if let Some(ak) = &rbum_cert_modify_req.ak {
            rbum_cert.ak = Set(ak.to_string());
        }
        if let Some(sk) = &rbum_cert_modify_req.sk {
            rbum_cert.sk = Set(sk.to_string());
        }
        if let Some(ext) = &rbum_cert_modify_req.ext {
            rbum_cert.ext = Set(ext.to_string());
        }
        if let Some(start_time) = rbum_cert_modify_req.start_time {
            rbum_cert.start_time = Set(start_time.naive_utc());
        }
        if let Some(end_time) = rbum_cert_modify_req.end_time {
            rbum_cert.end_time = Set(end_time.naive_utc());
        }
        if let Some(coexist_flag) = &rbum_cert_modify_req.coexist_flag {
            rbum_cert.coexist_flag = Set(coexist_flag.to_string());
        }
        if let Some(status) = &rbum_cert_modify_req.status {
            rbum_cert.status = Set(status.to_string());
        }
        db.update_one(rbum_cert, cxt).await?;
        Ok(())
    }

    pub async fn delete_rbum_cert<'a>(id: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        db.soft_delete(rbum_cert::Entity::find().filter(rbum_cert::Column::Id.eq(id)), &cxt.account_id).await
    }

    pub async fn get_rbum_cert<'a>(id: &str, filter: &RbumBasicFilterReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<RbumCertDetailResp> {
        let mut query = package_rbum_cert_query(true, filter, cxt);
        query.and_where(Expr::tbl(rbum_cert::Entity, rbum_cert::Column::Id).eq(id.to_string()));
        let query = db.get_dto(&query).await?;
        match query {
            Some(rbum_cert) => Ok(rbum_cert),
            // TODO
            None => Err(TardisError::NotFound("".to_string())),
        }
    }

    pub async fn find_rbum_certs<'a>(
        filter: &RbumBasicFilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_update: Option<bool>,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumCertSummaryResp>> {
        let mut query = package_rbum_cert_query(false, filter, cxt);
        if let Some(sort) = desc_sort_by_update {
            query.order_by(rbum_cert::Column::UpdateTime, if sort { Order::Desc } else { Order::Asc });
        }
        let (records, total_size) = db.paginate_dtos(&query, page_number, page_size).await?;
        Ok(TardisPage {
            page_size,
            page_number,
            total_size,
            records,
        })
    }

    pub fn package_rbum_cert_query(is_detail: bool, filter: &RbumBasicFilterReq, cxt: &TardisContext) -> SelectStatement {
        let updater_table = Alias::new("updater");
        let rel_app_table = Alias::new("relApp");
        let rel_tenant_table = Alias::new("relTenant");
        let rel_rbum_item_table = Alias::new("relRbumItem");

        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_cert::Entity, rbum_cert::Column::Id),
                (rbum_cert::Entity, rbum_cert::Column::Name),
                (rbum_cert::Entity, rbum_cert::Column::Ak),
                (rbum_cert::Entity, rbum_cert::Column::Sk),
                (rbum_cert::Entity, rbum_cert::Column::Ext),
                (rbum_cert::Entity, rbum_cert::Column::StartTime),
                (rbum_cert::Entity, rbum_cert::Column::EndTime),
                (rbum_cert::Entity, rbum_cert::Column::CoexistFlag),
                (rbum_cert::Entity, rbum_cert::Column::Status),
                (rbum_cert::Entity, rbum_cert::Column::RelRbumCertConfId),
                (rbum_cert::Entity, rbum_cert::Column::RelRbumDomainId),
                (rbum_cert::Entity, rbum_cert::Column::RelRbumItemId),
                (rbum_cert::Entity, rbum_cert::Column::RelAppId),
                (rbum_cert::Entity, rbum_cert::Column::RelTenantId),
                (rbum_cert::Entity, rbum_cert::Column::UpdaterId),
                (rbum_cert::Entity, rbum_cert::Column::CreateTime),
                (rbum_cert::Entity, rbum_cert::Column::UpdateTime),
            ])
            .from(rbum_cert::Entity);

        if filter.rel_cxt_app {
            query.and_where(Expr::tbl(rbum_cert::Entity, rbum_cert::Column::RelAppId).eq(cxt.app_id.as_str()));
        }
        if filter.rel_cxt_tenant {
            query.and_where(Expr::tbl(rbum_cert::Entity, rbum_cert::Column::RelTenantId).eq(cxt.tenant_id.as_str()));
        }
        if filter.rel_cxt_updater {
            query.and_where(Expr::tbl(rbum_cert::Entity, rbum_cert::Column::UpdaterId).eq(cxt.account_id.as_str()));
        }

        if is_detail {
            query
                .expr_as(Expr::tbl(rbum_cert_conf::Entity, rbum_cert_conf::Column::Name), Alias::new("rel_rbum_cert_conf_name"))
                .expr_as(Expr::tbl(rbum_domain::Entity, rbum_domain::Column::Name), Alias::new("rel_rbum_domain_name"))
                .expr_as(Expr::tbl(rel_rbum_item_table.clone(), rbum_item::Column::Name), Alias::new("rel_rbum_item_name"))
                .expr_as(Expr::tbl(rel_app_table.clone(), rbum_item::Column::Name), Alias::new("rel_app_name"))
                .expr_as(Expr::tbl(rel_tenant_table.clone(), rbum_item::Column::Name), Alias::new("rel_tenant_name"))
                .expr_as(Expr::tbl(updater_table.clone(), rbum_item::Column::Name), Alias::new("updater_name"))
                .inner_join(
                    rbum_cert_conf::Entity,
                    Expr::tbl(rbum_cert_conf::Entity, rbum_cert_conf::Column::Id).equals(rbum_cert::Entity, rbum_cert::Column::RelRbumCertConfId),
                )
                .inner_join(
                    rbum_domain::Entity,
                    Expr::tbl(rbum_domain::Entity, rbum_domain::Column::Id).equals(rbum_cert::Entity, rbum_cert::Column::RelRbumDomainId),
                )
                .join_as(
                    JoinType::InnerJoin,
                    rbum_item::Entity,
                    rel_rbum_item_table.clone(),
                    Expr::tbl(rel_rbum_item_table, rbum_item::Column::Id).equals(rbum_cert::Entity, rbum_cert::Column::RelRbumItemId),
                )
                .join_as(
                    JoinType::InnerJoin,
                    rbum_item::Entity,
                    rel_app_table.clone(),
                    Expr::tbl(rel_app_table, rbum_item::Column::Id).equals(rbum_cert::Entity, rbum_cert::Column::RelAppId),
                )
                .join_as(
                    JoinType::InnerJoin,
                    rbum_item::Entity,
                    rel_tenant_table.clone(),
                    Expr::tbl(rel_tenant_table, rbum_item::Column::Id).equals(rbum_cert::Entity, rbum_cert::Column::RelTenantId),
                )
                .join_as(
                    JoinType::InnerJoin,
                    rbum_item::Entity,
                    updater_table.clone(),
                    Expr::tbl(updater_table, rbum_item::Column::Id).equals(rbum_cert::Entity, rbum_cert::Column::UpdaterId),
                );
        }

        query
    }
}

pub mod cert_conf {
    use tardis::basic::dto::TardisContext;
    use tardis::basic::error::TardisError;
    use tardis::basic::result::TardisResult;
    use tardis::db::reldb_client::TardisRelDBlConnection;
    use tardis::db::sea_orm::*;
    use tardis::db::sea_query::*;
    use tardis::web::web_resp::TardisPage;

    use crate::rbum::domain::{rbum_cert, rbum_cert_conf, rbum_domain, rbum_item};
    use crate::rbum::dto::filer_dto::RbumBasicFilterReq;
    use crate::rbum::dto::rbum_cert_conf_dto::{RbumCertConfAddReq, RbumCertConfDetailResp, RbumCertConfModifyReq, RbumCertConfSummaryResp};

    pub async fn add_rbum_cert_conf<'a>(
        rel_rbum_domain_id: &str,
        rbum_cert_conf_add_req: &RbumCertConfAddReq,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<String> {
        let rbum_cert_conf_id: String = db
            .insert_one(
                rbum_cert_conf::ActiveModel {
                    name: Set(rbum_cert_conf_add_req.name.to_string()),
                    note: Set(rbum_cert_conf_add_req.note.as_ref().unwrap_or(&"".to_string()).to_string()),
                    ak_note: Set(rbum_cert_conf_add_req.ak_note.as_ref().unwrap_or(&"".to_string()).to_string()),
                    ak_rule: Set(rbum_cert_conf_add_req.ak_rule.as_ref().unwrap_or(&"".to_string()).to_string()),
                    sk_note: Set(rbum_cert_conf_add_req.sk_note.as_ref().unwrap_or(&"".to_string()).to_string()),
                    sk_rule: Set(rbum_cert_conf_add_req.sk_rule.as_ref().unwrap_or(&"".to_string()).to_string()),
                    repeatable: Set(rbum_cert_conf_add_req.repeatable.unwrap_or(true)),
                    is_basic: Set(rbum_cert_conf_add_req.is_basic.unwrap_or(true)),
                    rest_by_kinds: Set(rbum_cert_conf_add_req.rest_by_kinds.as_ref().unwrap_or(&"".to_string()).to_string()),
                    expire_sec: Set(rbum_cert_conf_add_req.expire_sec.unwrap_or(0)),
                    coexist_num: Set(rbum_cert_conf_add_req.coexist_num.unwrap_or(1)),
                    rel_rbum_domain_id: Set(rel_rbum_domain_id.to_string()),
                    ..Default::default()
                },
                cxt,
            )
            .await?
            .last_insert_id;
        Ok(rbum_cert_conf_id)
    }

    pub async fn modify_rbum_cert_conf<'a>(id: &str, rbum_cert_conf_modify_req: &RbumCertConfModifyReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        let mut rbum_cert_conf = rbum_cert_conf::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(name) = &rbum_cert_conf_modify_req.name {
            rbum_cert_conf.name = Set(name.to_string());
        }
        if let Some(note) = &rbum_cert_conf_modify_req.note {
            rbum_cert_conf.note = Set(note.to_string());
        }
        if let Some(ak_note) = &rbum_cert_conf_modify_req.ak_note {
            rbum_cert_conf.ak_note = Set(ak_note.to_string());
        }
        if let Some(ak_rule) = &rbum_cert_conf_modify_req.ak_rule {
            rbum_cert_conf.ak_rule = Set(ak_rule.to_string());
        }
        if let Some(sk_note) = &rbum_cert_conf_modify_req.sk_note {
            rbum_cert_conf.sk_note = Set(sk_note.to_string());
        }
        if let Some(sk_rule) = &rbum_cert_conf_modify_req.sk_rule {
            rbum_cert_conf.sk_rule = Set(sk_rule.to_string());
        }
        if let Some(repeatable) = rbum_cert_conf_modify_req.repeatable {
            rbum_cert_conf.repeatable = Set(repeatable);
        }
        if let Some(is_basic) = rbum_cert_conf_modify_req.is_basic {
            rbum_cert_conf.is_basic = Set(is_basic);
        }
        if let Some(rest_by_kinds) = &rbum_cert_conf_modify_req.rest_by_kinds {
            rbum_cert_conf.rest_by_kinds = Set(rest_by_kinds.to_string());
        }
        if let Some(expire_sec) = rbum_cert_conf_modify_req.expire_sec {
            rbum_cert_conf.expire_sec = Set(expire_sec);
        }
        if let Some(coexist_num) = rbum_cert_conf_modify_req.coexist_num {
            rbum_cert_conf.coexist_num = Set(coexist_num);
        }
        db.update_one(rbum_cert_conf, cxt).await?;
        Ok(())
    }

    pub async fn delete_rbum_cert_conf<'a>(id: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        db.soft_delete(rbum_cert_conf::Entity::find().filter(rbum_cert_conf::Column::Id.eq(id)), &cxt.account_id).await
    }

    pub async fn get_rbum_cert_conf<'a>(id: &str, filter: &RbumBasicFilterReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<RbumCertConfDetailResp> {
        let mut query = package_rbum_cert_conf_query(true, filter, cxt);
        query.and_where(Expr::tbl(rbum_cert_conf::Entity, rbum_cert_conf::Column::Id).eq(id.to_string()));
        let query = db.get_dto(&query).await?;
        match query {
            Some(rbum_cert_conf) => Ok(rbum_cert_conf),
            // TODO
            None => Err(TardisError::NotFound("".to_string())),
        }
    }

    pub async fn find_rbum_cert_confs<'a>(
        filter: &RbumBasicFilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_update: Option<bool>,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumCertConfSummaryResp>> {
        let mut query = package_rbum_cert_conf_query(true, filter, cxt);
        if let Some(sort) = desc_sort_by_update {
            query.order_by(rbum_cert_conf::Column::UpdateTime, if sort { Order::Desc } else { Order::Asc });
        }
        let (records, total_size) = db.paginate_dtos(&query, page_number, page_size).await?;
        Ok(TardisPage {
            page_size,
            page_number,
            total_size,
            records,
        })
    }

    pub fn package_rbum_cert_conf_query(is_detail: bool, filter: &RbumBasicFilterReq, cxt: &TardisContext) -> SelectStatement {
        let updater_table = Alias::new("updater");
        let rel_app_table = Alias::new("relApp");
        let rel_tenant_table = Alias::new("relTenant");

        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::Id),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::Name),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::Note),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::AkNote),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::AkRule),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::SkNote),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::SkRule),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::Repeatable),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::IsBasic),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::RestByKinds),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::ExpireSec),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::CoexistNum),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::RelRbumDomainId),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::RelAppId),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::RelTenantId),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::UpdaterId),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::CreateTime),
                (rbum_cert_conf::Entity, rbum_cert_conf::Column::UpdateTime),
            ])
            .from(rbum_cert::Entity);

        if filter.rel_cxt_app {
            query.and_where(Expr::tbl(rbum_cert_conf::Entity, rbum_cert_conf::Column::RelAppId).eq(cxt.app_id.as_str()));
        }
        if filter.rel_cxt_tenant {
            query.and_where(Expr::tbl(rbum_cert_conf::Entity, rbum_cert_conf::Column::RelTenantId).eq(cxt.tenant_id.as_str()));
        }
        if filter.rel_cxt_updater {
            query.and_where(Expr::tbl(rbum_cert_conf::Entity, rbum_cert_conf::Column::UpdaterId).eq(cxt.account_id.as_str()));
        }

        if is_detail {
            query
                .expr_as(Expr::tbl(rbum_domain::Entity, rbum_domain::Column::Name), Alias::new("rel_rbum_domain_name"))
                .expr_as(Expr::tbl(rel_app_table.clone(), rbum_item::Column::Name), Alias::new("rel_app_name"))
                .expr_as(Expr::tbl(rel_tenant_table.clone(), rbum_item::Column::Name), Alias::new("rel_tenant_name"))
                .expr_as(Expr::tbl(updater_table.clone(), rbum_item::Column::Name), Alias::new("updater_name"))
                .inner_join(
                    rbum_domain::Entity,
                    Expr::tbl(rbum_domain::Entity, rbum_domain::Column::Id).equals(rbum_cert_conf::Entity, rbum_cert_conf::Column::RelRbumDomainId),
                )
                .join_as(
                    JoinType::InnerJoin,
                    rbum_item::Entity,
                    rel_app_table.clone(),
                    Expr::tbl(rel_app_table, rbum_item::Column::Id).equals(rbum_cert_conf::Entity, rbum_cert_conf::Column::RelAppId),
                )
                .join_as(
                    JoinType::InnerJoin,
                    rbum_item::Entity,
                    rel_tenant_table.clone(),
                    Expr::tbl(rel_tenant_table, rbum_item::Column::Id).equals(rbum_cert_conf::Entity, rbum_cert_conf::Column::RelTenantId),
                )
                .join_as(
                    JoinType::InnerJoin,
                    rbum_item::Entity,
                    updater_table.clone(),
                    Expr::tbl(updater_table, rbum_item::Column::Id).equals(rbum_cert_conf::Entity, rbum_cert_conf::Column::UpdaterId),
                );
        }

        query
    }
}
