pub mod domain {
    use tardis::basic::dto::TardisContext;
    use tardis::basic::error::TardisError;
    use tardis::basic::result::TardisResult;
    use tardis::db::reldb_client::TardisRelDBlConnection;
    use tardis::db::sea_orm::*;
    use tardis::db::sea_query::*;
    use tardis::web::web_resp::TardisPage;

    use crate::rbum::domain::{rbum_domain, rbum_item};
    use crate::rbum::dto::filer_dto::RbumBasicFilterReq;
    use crate::rbum::dto::rbum_domain_dto::{RbumDomainAddReq, RbumDomainDetailResp, RbumDomainModifyReq, RbumDomainSummaryResp};
    use crate::rbum::enumeration::RbumScopeKind;

    pub async fn add_rbum_domain<'a>(rbum_domain_add_req: &RbumDomainAddReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<String> {
        let rbum_domain_id: String = db
            .insert_one(
                rbum_domain::ActiveModel {
                    uri_authority: Set(rbum_domain_add_req.uri_authority.to_string()),
                    name: Set(rbum_domain_add_req.name.to_string()),
                    note: Set(rbum_domain_add_req.note.as_ref().unwrap_or(&"".to_string()).to_string()),
                    icon: Set(rbum_domain_add_req.icon.as_ref().unwrap_or(&"".to_string()).to_string()),
                    sort: Set(rbum_domain_add_req.sort.unwrap_or(0)),
                    scope_kind: Set(rbum_domain_add_req.scope_kind.as_ref().unwrap_or(&RbumScopeKind::App).to_string()),
                    ..Default::default()
                },
                cxt,
            )
            .await?
            .last_insert_id;
        Ok(rbum_domain_id)
    }

    pub async fn modify_rbum_domain<'a>(id: &str, rbum_domain_modify_req: &RbumDomainModifyReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        let mut rbum_domain = rbum_domain::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(uri_authority) = &rbum_domain_modify_req.uri_authority {
            rbum_domain.uri_authority = Set(uri_authority.to_string());
        }
        if let Some(name) = &rbum_domain_modify_req.name {
            rbum_domain.name = Set(name.to_string());
        }
        if let Some(note) = &rbum_domain_modify_req.note {
            rbum_domain.note = Set(note.to_string());
        }
        if let Some(icon) = &rbum_domain_modify_req.icon {
            rbum_domain.icon = Set(icon.to_string());
        }
        if let Some(sort) = rbum_domain_modify_req.sort {
            rbum_domain.sort = Set(sort);
        }
        if let Some(scope_kind) = &rbum_domain_modify_req.scope_kind {
            rbum_domain.scope_kind = Set(scope_kind.to_string());
        }
        db.update_one(rbum_domain, cxt).await?;
        Ok(())
    }

    pub async fn delete_rbum_domain<'a>(id: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        db.soft_delete(rbum_domain::Entity::find().filter(rbum_domain::Column::Id.eq(id)), &cxt.account_id).await
    }

    pub async fn get_rbum_domain<'a>(id: &str, filter: &RbumBasicFilterReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<RbumDomainDetailResp> {
        let mut query = package_rbum_domain_query(true, filter, cxt);
        query.and_where(Expr::tbl(rbum_domain::Entity, rbum_domain::Column::Id).eq(id.to_string()));
        let query = db.get_dto(&query).await?;
        match query {
            Some(rbum_domain) => Ok(rbum_domain),
            // TODO
            None => Err(TardisError::NotFound("".to_string())),
        }
    }

    pub async fn find_rbum_domains<'a>(
        filter: &RbumBasicFilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_update: Option<bool>,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumDomainSummaryResp>> {
        let mut query = package_rbum_domain_query(false, filter, cxt);
        if let Some(sort) = desc_sort_by_update {
            query.order_by(rbum_domain::Column::UpdateTime, if sort { Order::Desc } else { Order::Asc });
        }
        let (records, total_size) = db.paginate_dtos(&query, page_number, page_size).await?;
        Ok(TardisPage {
            page_size,
            page_number,
            total_size,
            records,
        })
    }

    pub fn package_rbum_domain_query(is_detail: bool, filter: &RbumBasicFilterReq, cxt: &TardisContext) -> SelectStatement {
        let updater_table = Alias::new("updater");
        let rel_app_table = Alias::new("relApp");
        let rel_tenant_table = Alias::new("relTenant");

        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_domain::Entity, rbum_domain::Column::Id),
                (rbum_domain::Entity, rbum_domain::Column::UriAuthority),
                (rbum_domain::Entity, rbum_domain::Column::Name),
                (rbum_domain::Entity, rbum_domain::Column::Note),
                (rbum_domain::Entity, rbum_domain::Column::Icon),
                (rbum_domain::Entity, rbum_domain::Column::Sort),
                (rbum_domain::Entity, rbum_domain::Column::RelAppId),
                (rbum_domain::Entity, rbum_domain::Column::RelTenantId),
                (rbum_domain::Entity, rbum_domain::Column::UpdaterId),
                (rbum_domain::Entity, rbum_domain::Column::CreateTime),
                (rbum_domain::Entity, rbum_domain::Column::UpdateTime),
                (rbum_domain::Entity, rbum_domain::Column::ScopeKind),
            ])
            .from(rbum_domain::Entity);

        if filter.rel_cxt_app {
            query.and_where(Expr::tbl(rbum_domain::Entity, rbum_domain::Column::RelAppId).eq(cxt.app_id.as_str()));
        }
        if filter.rel_cxt_tenant {
            query.and_where(Expr::tbl(rbum_domain::Entity, rbum_domain::Column::RelTenantId).eq(cxt.tenant_id.as_str()));
        }
        if filter.rel_cxt_updater {
            query.and_where(Expr::tbl(rbum_domain::Entity, rbum_domain::Column::UpdaterId).eq(cxt.account_id.as_str()));
        }
        if let Some(scope_kind) = &filter.scope_kind {
            query.and_where(Expr::tbl(rbum_domain::Entity, rbum_domain::Column::ScopeKind).eq(scope_kind.to_string()));
        }

        if is_detail {
            query
                .expr_as(Expr::tbl(rel_app_table.clone(), rbum_item::Column::Name), Alias::new("rel_app_name"))
                .expr_as(Expr::tbl(rel_tenant_table.clone(), rbum_item::Column::Name), Alias::new("rel_tenant_name"))
                .expr_as(Expr::tbl(updater_table.clone(), rbum_item::Column::Name), Alias::new("updater_name"))
                .join_as(
                    JoinType::InnerJoin,
                    rbum_item::Entity,
                    rel_app_table.clone(),
                    Expr::tbl(rel_app_table, rbum_item::Column::Id).equals(rbum_domain::Entity, rbum_domain::Column::RelAppId),
                )
                .join_as(
                    JoinType::InnerJoin,
                    rbum_item::Entity,
                    rel_tenant_table.clone(),
                    Expr::tbl(rel_tenant_table, rbum_item::Column::Id).equals(rbum_domain::Entity, rbum_domain::Column::RelTenantId),
                )
                .join_as(
                    JoinType::InnerJoin,
                    rbum_item::Entity,
                    updater_table.clone(),
                    Expr::tbl(updater_table, rbum_item::Column::Id).equals(rbum_domain::Entity, rbum_domain::Column::UpdaterId),
                );
        }

        query
    }
}
