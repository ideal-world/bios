pub mod set {
    use tardis::basic::dto::TardisContext;
    use tardis::basic::error::TardisError;
    use tardis::basic::result::TardisResult;
    use tardis::db::reldb_client::TardisRelDBlConnection;
    use tardis::db::sea_orm::*;
    use tardis::db::sea_query::*;
    use tardis::web::web_resp::TardisPage;

    use crate::rbum::domain::{rbum_cert, rbum_item, rbum_set};
    use crate::rbum::dto::filer_dto::RbumBasicFilterReq;
    use crate::rbum::dto::rbum_set_dto::{RbumSetAddReq, RbumSetDetailResp, RbumSetModifyReq, RbumSetSummaryResp};
    use crate::rbum::enumeration::RbumScopeKind;

    pub async fn add_rbum_set<'a>(rbum_set_add_req: &RbumSetAddReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<String> {
        let rbum_set_id: String = db
            .insert_one(
                rbum_set::ActiveModel {
                    name: Set(rbum_set_add_req.name.to_string()),
                    note: Set(rbum_set_add_req.note.as_ref().unwrap_or(&"".to_string()).to_string()),
                    icon: Set(rbum_set_add_req.icon.as_ref().unwrap_or(&"".to_string()).to_string()),
                    sort: Set(rbum_set_add_req.sort.unwrap_or(0)),
                    tags: Set(rbum_set_add_req.tags.as_ref().unwrap_or(&"".to_string()).to_string()),
                    scope_kind: Set(rbum_set_add_req.scope_kind.as_ref().unwrap_or(&RbumScopeKind::App).to_string()),
                    ..Default::default()
                },
                cxt,
            )
            .await?
            .last_insert_id;
        Ok(rbum_set_id)
    }

    pub async fn modify_rbum_set<'a>(id: &str, rbum_set_modify_req: &RbumSetModifyReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        let mut rbum_set = rbum_set::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(name) = &rbum_set_modify_req.name {
            rbum_set.name = Set(name.to_string());
        }
        if let Some(note) = &rbum_set_modify_req.note {
            rbum_set.note = Set(note.to_string());
        }
        if let Some(icon) = &rbum_set_modify_req.icon {
            rbum_set.icon = Set(icon.to_string());
        }
        if let Some(sort) = rbum_set_modify_req.sort {
            rbum_set.sort = Set(sort);
        }
        if let Some(tags) = &rbum_set_modify_req.tags {
            rbum_set.tags = Set(tags.to_string());
        }
        if let Some(scope_kind) = &rbum_set_modify_req.scope_kind {
            rbum_set.scope_kind = Set(scope_kind.to_string());
        }
        db.update_one(rbum_set, cxt).await?;
        Ok(())
    }

    pub async fn delete_rbum_set<'a>(id: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        db.soft_delete(rbum_set::Entity::find().filter(rbum_set::Column::Id.eq(id)), &cxt.account_id).await
    }

    pub async fn get_rbum_set<'a>(id: &str, filter: &RbumBasicFilterReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<RbumSetDetailResp> {
        let mut query = package_rbum_set_query(true, filter, cxt);
        query.and_where(Expr::tbl(rbum_set::Entity, rbum_set::Column::Id).eq(id.to_string()));
        let query = db.get_dto(&query).await?;
        match query {
            Some(rbum_set) => Ok(rbum_set),
            // TODO
            None => Err(TardisError::NotFound("".to_string())),
        }
    }

    pub async fn find_rbum_sets<'a>(
        filter: &RbumBasicFilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_update: Option<bool>,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumSetSummaryResp>> {
        let mut query = package_rbum_set_query(false, filter, cxt);
        if let Some(sort) = desc_sort_by_update {
            query.order_by(rbum_set::Column::UpdateTime, if sort { Order::Desc } else { Order::Asc });
        }
        let (records, total_size) = db.paginate_dtos(&query, page_number, page_size).await?;
        Ok(TardisPage {
            page_size,
            page_number,
            total_size,
            records,
        })
    }

    pub fn package_rbum_set_query(is_detail: bool, filter: &RbumBasicFilterReq, cxt: &TardisContext) -> SelectStatement {
        let updater_table = Alias::new("updater");
        let rel_app_table = Alias::new("relApp");
        let rel_tenant_table = Alias::new("relTenant");

        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_set::Entity, rbum_set::Column::Id),
                (rbum_set::Entity, rbum_set::Column::Name),
                (rbum_set::Entity, rbum_set::Column::Note),
                (rbum_set::Entity, rbum_set::Column::Icon),
                (rbum_set::Entity, rbum_set::Column::Sort),
                (rbum_set::Entity, rbum_set::Column::Tags),
                (rbum_set::Entity, rbum_set::Column::RelAppId),
                (rbum_set::Entity, rbum_set::Column::RelTenantId),
                (rbum_set::Entity, rbum_set::Column::UpdaterId),
                (rbum_set::Entity, rbum_set::Column::CreateTime),
                (rbum_set::Entity, rbum_set::Column::UpdateTime),
                (rbum_set::Entity, rbum_set::Column::ScopeKind),
            ])
            .from(rbum_cert::Entity);

        if filter.rel_cxt_app {
            query.and_where(Expr::tbl(rbum_set::Entity, rbum_set::Column::RelAppId).eq(cxt.app_id.as_str()));
        }
        if filter.rel_cxt_tenant {
            query.and_where(Expr::tbl(rbum_set::Entity, rbum_set::Column::RelTenantId).eq(cxt.tenant_id.as_str()));
        }
        if filter.rel_cxt_updater {
            query.and_where(Expr::tbl(rbum_set::Entity, rbum_set::Column::UpdaterId).eq(cxt.account_id.as_str()));
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
                    Expr::tbl(rel_app_table, rbum_item::Column::Id).equals(rbum_set::Entity, rbum_set::Column::RelAppId),
                )
                .join_as(
                    JoinType::InnerJoin,
                    rbum_item::Entity,
                    rel_tenant_table.clone(),
                    Expr::tbl(rel_tenant_table, rbum_item::Column::Id).equals(rbum_set::Entity, rbum_set::Column::RelTenantId),
                )
                .join_as(
                    JoinType::InnerJoin,
                    rbum_item::Entity,
                    updater_table.clone(),
                    Expr::tbl(updater_table, rbum_item::Column::Id).equals(rbum_set::Entity, rbum_set::Column::UpdaterId),
                );
        }

        query
    }
}

pub mod set_cate {
    use tardis::basic::dto::TardisContext;
    use tardis::basic::error::TardisError;
    use tardis::basic::result::TardisResult;
    use tardis::db::reldb_client::TardisRelDBlConnection;
    use tardis::db::sea_orm::*;
    use tardis::db::sea_query::*;
    use tardis::web::web_resp::TardisPage;

    use crate::rbum::domain::{rbum_cert, rbum_item, rbum_set_cate};
    use crate::rbum::dto::filer_dto::RbumBasicFilterReq;
    use crate::rbum::dto::rbum_set_cate_dto::{RbumSetCateAddReq, RbumSetCateDetailResp, RbumSetCateModifyReq, RbumSetCateSummaryResp};
    use crate::rbum::enumeration::RbumScopeKind;

    pub async fn add_rbum_set_cate<'a>(
        rel_rbum_set_id: &str,
        rbum_set_cate_add_req: &RbumSetCateAddReq,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<String> {
        // TODO set sys_code
        let rbum_set_cate_id: String = db
            .insert_one(
                rbum_set_cate::ActiveModel {
                    bus_code: Set(rbum_set_cate_add_req.bus_code.to_string()),
                    name: Set(rbum_set_cate_add_req.name.to_string()),
                    sort: Set(rbum_set_cate_add_req.sort.unwrap_or(0)),
                    scope_kind: Set(rbum_set_cate_add_req.scope_kind.as_ref().unwrap_or(&RbumScopeKind::App).to_string()),
                    rel_rbum_set_id: Set(rel_rbum_set_id.to_string()),
                    ..Default::default()
                },
                cxt,
            )
            .await?
            .last_insert_id;
        Ok(rbum_set_cate_id)
    }

    pub async fn modify_rbum_set_cate<'a>(id: &str, rbum_set_cate_modify_req: &RbumSetCateModifyReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        let mut rbum_set_cate = rbum_set_cate::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(bus_code) = &rbum_set_cate_modify_req.bus_code {
            rbum_set_cate.bus_code = Set(bus_code.to_string());
        }
        if let Some(name) = &rbum_set_cate_modify_req.name {
            rbum_set_cate.name = Set(name.to_string());
        }
        if let Some(sort) = rbum_set_cate_modify_req.sort {
            rbum_set_cate.sort = Set(sort);
        }
        if let Some(scope_kind) = &rbum_set_cate_modify_req.scope_kind {
            rbum_set_cate.scope_kind = Set(scope_kind.to_string());
        }
        db.update_one(rbum_set_cate, cxt).await?;
        Ok(())
    }

    pub async fn delete_rbum_set_cate<'a>(id: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        db.soft_delete(rbum_set_cate::Entity::find().filter(rbum_set_cate::Column::Id.eq(id)), &cxt.account_id).await
    }

    pub async fn get_rbum_set_cate<'a>(id: &str, filter: &RbumBasicFilterReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<RbumSetCateDetailResp> {
        let mut query = package_rbum_set_cate_query(true, filter, cxt);
        query.and_where(Expr::tbl(rbum_set_cate::Entity, rbum_set_cate::Column::Id).eq(id.to_string()));
        let query = db.get_dto(&query).await?;
        match query {
            Some(rbum_set_cate) => Ok(rbum_set_cate),
            // TODO
            None => Err(TardisError::NotFound("".to_string())),
        }
    }

    pub async fn find_rbum_set_cates<'a>(
        filter: &RbumBasicFilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_update: Option<bool>,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumSetCateSummaryResp>> {
        let mut query = package_rbum_set_cate_query(false, filter, cxt);
        if let Some(sort) = desc_sort_by_update {
            query.order_by(rbum_set_cate::Column::UpdateTime, if sort { Order::Desc } else { Order::Asc });
        }
        let (records, total_size) = db.paginate_dtos(&query, page_number, page_size).await?;
        Ok(TardisPage {
            page_size,
            page_number,
            total_size,
            records,
        })
    }

    pub fn package_rbum_set_cate_query(is_detail: bool, filter: &RbumBasicFilterReq, cxt: &TardisContext) -> SelectStatement {
        let updater_table = Alias::new("updater");
        let rel_app_table = Alias::new("relApp");
        let rel_tenant_table = Alias::new("relTenant");

        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_set_cate::Entity, rbum_set_cate::Column::Id),
                (rbum_set_cate::Entity, rbum_set_cate::Column::BusCode),
                (rbum_set_cate::Entity, rbum_set_cate::Column::Name),
                (rbum_set_cate::Entity, rbum_set_cate::Column::Sort),
                (rbum_set_cate::Entity, rbum_set_cate::Column::RelAppId),
                (rbum_set_cate::Entity, rbum_set_cate::Column::RelTenantId),
                (rbum_set_cate::Entity, rbum_set_cate::Column::UpdaterId),
                (rbum_set_cate::Entity, rbum_set_cate::Column::CreateTime),
                (rbum_set_cate::Entity, rbum_set_cate::Column::UpdateTime),
                (rbum_set_cate::Entity, rbum_set_cate::Column::ScopeKind),
            ])
            .from(rbum_cert::Entity);

        if filter.rel_cxt_app {
            query.and_where(Expr::tbl(rbum_set_cate::Entity, rbum_set_cate::Column::RelAppId).eq(cxt.app_id.as_str()));
        }
        if filter.rel_cxt_tenant {
            query.and_where(Expr::tbl(rbum_set_cate::Entity, rbum_set_cate::Column::RelTenantId).eq(cxt.tenant_id.as_str()));
        }
        if filter.rel_cxt_updater {
            query.and_where(Expr::tbl(rbum_set_cate::Entity, rbum_set_cate::Column::UpdaterId).eq(cxt.account_id.as_str()));
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
                    Expr::tbl(rel_app_table, rbum_item::Column::Id).equals(rbum_set_cate::Entity, rbum_set_cate::Column::RelAppId),
                )
                .join_as(
                    JoinType::InnerJoin,
                    rbum_item::Entity,
                    rel_tenant_table.clone(),
                    Expr::tbl(rel_tenant_table, rbum_item::Column::Id).equals(rbum_set_cate::Entity, rbum_set_cate::Column::RelTenantId),
                )
                .join_as(
                    JoinType::InnerJoin,
                    rbum_item::Entity,
                    updater_table.clone(),
                    Expr::tbl(updater_table, rbum_item::Column::Id).equals(rbum_set_cate::Entity, rbum_set_cate::Column::UpdaterId),
                );
        }

        query
    }
}

pub mod set_item {
    use tardis::basic::dto::TardisContext;
    use tardis::basic::error::TardisError;
    use tardis::basic::result::TardisResult;
    use tardis::db::reldb_client::TardisRelDBlConnection;
    use tardis::db::sea_orm::*;
    use tardis::db::sea_query::*;
    use tardis::web::web_resp::TardisPage;

    use crate::rbum::domain::{rbum_cert, rbum_item, rbum_set_cate, rbum_set_item};
    use crate::rbum::dto::filer_dto::RbumBasicFilterReq;
    use crate::rbum::dto::rbum_set_item_dto::{RbumSetItemAddReq, RbumSetItemDetailResp, RbumSetItemModifyReq};

    pub async fn add_rbum_set_item<'a>(
        rel_rbum_set_id: &str,
        rel_rbum_set_cate_code: &str,
        rel_rbum_item_id: &str,
        rbum_set_item_add_req: &RbumSetItemAddReq,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<String> {
        let rbum_set_item_id: String = db
            .insert_one(
                rbum_set_item::ActiveModel {
                    rel_rbum_set_id: Set(rel_rbum_set_id.to_string()),
                    rel_rbum_set_cate_code: Set(rel_rbum_set_cate_code.to_string()),
                    rel_rbum_item_id: Set(rel_rbum_item_id.to_string()),
                    sort: Set(rbum_set_item_add_req.sort),
                    ..Default::default()
                },
                cxt,
            )
            .await?
            .last_insert_id;
        Ok(rbum_set_item_id)
    }

    pub async fn modify_rbum_set_item<'a>(id: &str, rbum_set_item_modify_req: &RbumSetItemModifyReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        let rbum_set_item = rbum_set_item::ActiveModel {
            id: Set(id.to_string()),
            sort: Set(rbum_set_item_modify_req.sort),
            ..Default::default()
        };
        db.update_one(rbum_set_item, cxt).await?;
        Ok(())
    }

    pub async fn delete_rbum_set_item<'a>(id: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        db.soft_delete(rbum_set_item::Entity::find().filter(rbum_set_item::Column::Id.eq(id)), &cxt.account_id).await
    }

    pub async fn get_rbum_set_item<'a>(id: &str, filter: &RbumBasicFilterReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<RbumSetItemDetailResp> {
        let mut query = package_rbum_set_item_query(true, filter, cxt);
        query.and_where(Expr::tbl(rbum_set_item::Entity, rbum_set_item::Column::Id).eq(id.to_string()));
        let query = db.get_dto(&query).await?;
        match query {
            Some(rbum_set_item) => Ok(rbum_set_item),
            // TODO
            None => Err(TardisError::NotFound("".to_string())),
        }
    }

    pub async fn find_rbum_set_items<'a>(
        filter: &RbumBasicFilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_update: Option<bool>,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumSetItemDetailResp>> {
        let mut query = package_rbum_set_item_query(false, filter, cxt);
        if let Some(sort) = desc_sort_by_update {
            query.order_by(rbum_set_item::Column::UpdateTime, if sort { Order::Desc } else { Order::Asc });
        }
        let (records, total_size) = db.paginate_dtos(&query, page_number, page_size).await?;
        Ok(TardisPage {
            page_size,
            page_number,
            total_size,
            records,
        })
    }

    pub fn package_rbum_set_item_query(is_detail: bool, filter: &RbumBasicFilterReq, cxt: &TardisContext) -> SelectStatement {
        let updater_table = Alias::new("updater");
        let rel_app_table = Alias::new("relApp");
        let rel_tenant_table = Alias::new("relTenant");
        let rel_item_table = Alias::new("relItem");

        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_set_item::Entity, rbum_set_item::Column::Id),
                (rbum_set_item::Entity, rbum_set_item::Column::Sort),
                (rbum_set_item::Entity, rbum_set_item::Column::RelRbumSetCateCode),
                (rbum_set_item::Entity, rbum_set_item::Column::RelRbumItemId),
                (rbum_set_item::Entity, rbum_set_item::Column::RelAppId),
                (rbum_set_item::Entity, rbum_set_item::Column::RelTenantId),
                (rbum_set_item::Entity, rbum_set_item::Column::UpdaterId),
                (rbum_set_item::Entity, rbum_set_item::Column::CreateTime),
                (rbum_set_item::Entity, rbum_set_item::Column::UpdateTime),
            ])
            .from(rbum_cert::Entity);

        if filter.rel_cxt_app {
            query.and_where(Expr::tbl(rbum_set_item::Entity, rbum_set_item::Column::RelAppId).eq(cxt.app_id.as_str()));
        }
        if filter.rel_cxt_tenant {
            query.and_where(Expr::tbl(rbum_set_item::Entity, rbum_set_item::Column::RelTenantId).eq(cxt.tenant_id.as_str()));
        }
        if filter.rel_cxt_updater {
            query.and_where(Expr::tbl(rbum_set_item::Entity, rbum_set_item::Column::UpdaterId).eq(cxt.account_id.as_str()));
        }

        if is_detail {
            query
                .expr_as(Expr::tbl(rbum_set_cate::Entity, rbum_set_cate::Column::Name), Alias::new("rel_rbum_set_cate_name"))
                .expr_as(Expr::tbl(rel_item_table.clone(), rbum_item::Column::Name), Alias::new("rel_rbum_item_name"))
                .expr_as(Expr::tbl(rel_app_table.clone(), rbum_item::Column::Name), Alias::new("rel_app_name"))
                .expr_as(Expr::tbl(rel_tenant_table.clone(), rbum_item::Column::Name), Alias::new("rel_tenant_name"))
                .expr_as(Expr::tbl(updater_table.clone(), rbum_item::Column::Name), Alias::new("updater_name"))
                .inner_join(
                    rbum_set_cate::Entity,
                    Expr::tbl(rbum_set_cate::Entity, rbum_set_cate::Column::SysCode).equals(rbum_set_item::Entity, rbum_set_item::Column::RelRbumSetCateCode),
                )
                .join_as(
                    JoinType::InnerJoin,
                    rbum_item::Entity,
                    rel_item_table.clone(),
                    Expr::tbl(rel_item_table, rbum_item::Column::Id).equals(rbum_set_item::Entity, rbum_set_item::Column::RelRbumItemId),
                )
                .join_as(
                    JoinType::InnerJoin,
                    rbum_item::Entity,
                    rel_app_table.clone(),
                    Expr::tbl(rel_app_table, rbum_item::Column::Id).equals(rbum_set_item::Entity, rbum_set_item::Column::RelAppId),
                )
                .join_as(
                    JoinType::InnerJoin,
                    rbum_item::Entity,
                    rel_tenant_table.clone(),
                    Expr::tbl(rel_tenant_table, rbum_item::Column::Id).equals(rbum_set_item::Entity, rbum_set_item::Column::RelTenantId),
                )
                .join_as(
                    JoinType::InnerJoin,
                    rbum_item::Entity,
                    updater_table.clone(),
                    Expr::tbl(updater_table, rbum_item::Column::Id).equals(rbum_set_item::Entity, rbum_set_item::Column::UpdaterId),
                );
        }

        query
    }
}
