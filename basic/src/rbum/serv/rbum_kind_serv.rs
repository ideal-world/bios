pub mod kind {
    use tardis::basic::dto::TardisContext;
    use tardis::basic::error::TardisError;
    use tardis::basic::result::TardisResult;
    use tardis::db::reldb_client::TardisRelDBlConnection;
    use tardis::db::sea_orm::*;
    use tardis::db::sea_query::*;
    use tardis::web::web_resp::TardisPage;

    use crate::rbum::domain::{rbum_item, rbum_kind};
    use crate::rbum::dto::filer_dto::RbumBasicFilterReq;
    use crate::rbum::dto::rbum_kind_dto::{RbumKindAddReq, RbumKindDetailResp, RbumKindModifyReq, RbumKindSummaryResp};
    use crate::rbum::enumeration::RbumScopeKind;

    pub async fn add_rbum_kind<'a>(rbum_kind_add_req: &RbumKindAddReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<String> {
        let rbum_kind_id: String = db
            .insert_one(
                rbum_kind::ActiveModel {
                    uri_scheme: Set(rbum_kind_add_req.uri_scheme.to_string()),
                    name: Set(rbum_kind_add_req.name.to_string()),
                    note: Set(rbum_kind_add_req.note.as_ref().unwrap_or(&"".to_string()).to_string()),
                    icon: Set(rbum_kind_add_req.icon.as_ref().unwrap_or(&"".to_string()).to_string()),
                    sort: Set(rbum_kind_add_req.sort.unwrap_or(0)),
                    ext_table_name: Set(rbum_kind_add_req.ext_table_name.as_ref().unwrap_or(&"".to_string()).to_string()),
                    scope_kind: Set(rbum_kind_add_req.scope_kind.as_ref().unwrap_or(&RbumScopeKind::App).to_string()),
                    ..Default::default()
                },
                cxt,
            )
            .await?
            .last_insert_id;
        Ok(rbum_kind_id)
    }

    pub async fn modify_rbum_kind<'a>(id: &str, rbum_kind_modify_req: &RbumKindModifyReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        let mut rbum_kind = rbum_kind::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(uri_scheme) = &rbum_kind_modify_req.uri_scheme {
            rbum_kind.uri_scheme = Set(uri_scheme.to_string());
        }
        if let Some(name) = &rbum_kind_modify_req.name {
            rbum_kind.name = Set(name.to_string());
        }
        if let Some(note) = &rbum_kind_modify_req.note {
            rbum_kind.note = Set(note.to_string());
        }
        if let Some(icon) = &rbum_kind_modify_req.icon {
            rbum_kind.icon = Set(icon.to_string());
        }
        if let Some(sort) = rbum_kind_modify_req.sort {
            rbum_kind.sort = Set(sort);
        }
        if let Some(ext_table_name) = &rbum_kind_modify_req.ext_table_name {
            rbum_kind.ext_table_name = Set(ext_table_name.to_string());
        }
        if let Some(scope_kind) = &rbum_kind_modify_req.scope_kind {
            rbum_kind.scope_kind = Set(scope_kind.to_string());
        }
        db.update_one(rbum_kind, cxt).await?;
        Ok(())
    }

    pub async fn delete_rbum_kind<'a>(id: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        db.soft_delete(rbum_kind::Entity::find().filter(rbum_kind::Column::Id.eq(id)), &cxt.account_id).await
    }

    pub async fn get_rbum_kind<'a>(id: &str, filter: &RbumBasicFilterReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<RbumKindDetailResp> {
        let mut query = package_rbum_kind_query(true, filter, cxt);
        query.and_where(Expr::tbl(rbum_kind::Entity, rbum_kind::Column::Id).eq(id.to_string()));
        let query = db.get_dto(&query).await?;
        match query {
            Some(rbum_kind) => Ok(rbum_kind),
            // TODO
            None => Err(TardisError::NotFound("".to_string())),
        }
    }

    pub async fn find_rbum_kinds<'a>(
        filter: &RbumBasicFilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_update: Option<bool>,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumKindSummaryResp>> {
        let mut query = package_rbum_kind_query(false, filter, cxt);
        if let Some(sort) = desc_sort_by_update {
            query.order_by(rbum_kind::Column::UpdateTime, if sort { Order::Desc } else { Order::Asc });
        }
        let (records, total_size) = db.paginate_dtos(&query, page_number, page_size).await?;
        Ok(TardisPage {
            page_size,
            page_number,
            total_size,
            records,
        })
    }

    fn package_rbum_kind_query(is_detail: bool, filter: &RbumBasicFilterReq, cxt: &TardisContext) -> SelectStatement {
        let updater_table = Alias::new("updater");
        let rel_app_table = Alias::new("relApp");
        let rel_tenant_table = Alias::new("relTenant");

        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_kind::Entity, rbum_kind::Column::Id),
                (rbum_kind::Entity, rbum_kind::Column::Name),
                (rbum_kind::Entity, rbum_kind::Column::Note),
                (rbum_kind::Entity, rbum_kind::Column::Icon),
                (rbum_kind::Entity, rbum_kind::Column::Sort),
                (rbum_kind::Entity, rbum_kind::Column::ExtTableName),
                (rbum_kind::Entity, rbum_kind::Column::RelAppId),
                (rbum_kind::Entity, rbum_kind::Column::RelTenantId),
                (rbum_kind::Entity, rbum_kind::Column::UpdaterId),
                (rbum_kind::Entity, rbum_kind::Column::CreateTime),
                (rbum_kind::Entity, rbum_kind::Column::UpdateTime),
                (rbum_kind::Entity, rbum_kind::Column::ScopeKind),
            ])
            .from(rbum_kind::Entity);

        if filter.rel_cxt_app {
            query.and_where(Expr::tbl(rbum_kind::Entity, rbum_kind::Column::RelAppId).eq(cxt.app_id.as_str()));
        }
        if filter.rel_cxt_tenant {
            query.and_where(Expr::tbl(rbum_kind::Entity, rbum_kind::Column::RelTenantId).eq(cxt.tenant_id.as_str()));
        }
        if filter.rel_cxt_updater {
            query.and_where(Expr::tbl(rbum_kind::Entity, rbum_kind::Column::UpdaterId).eq(cxt.account_id.as_str()));
        }
        if let Some(scope_kind) = &filter.scope_kind {
            query.and_where(Expr::tbl(rbum_kind::Entity, rbum_kind::Column::ScopeKind).eq(scope_kind.to_string()));
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
                    Expr::tbl(rel_app_table, rbum_item::Column::Id).equals(rbum_kind::Entity, rbum_kind::Column::RelAppId),
                )
                .join_as(
                    JoinType::InnerJoin,
                    rbum_item::Entity,
                    rel_tenant_table.clone(),
                    Expr::tbl(rel_tenant_table, rbum_item::Column::Id).equals(rbum_kind::Entity, rbum_kind::Column::RelTenantId),
                )
                .join_as(
                    JoinType::InnerJoin,
                    rbum_item::Entity,
                    updater_table.clone(),
                    Expr::tbl(updater_table, rbum_item::Column::Id).equals(rbum_kind::Entity, rbum_kind::Column::UpdaterId),
                );
        }

        query
    }
}

pub mod kind_attr {
    use tardis::basic::dto::TardisContext;
    use tardis::basic::error::TardisError;
    use tardis::basic::result::TardisResult;
    use tardis::db::reldb_client::TardisRelDBlConnection;
    use tardis::db::sea_orm::*;
    use tardis::db::sea_query::*;
    use tardis::web::web_resp::TardisPage;

    use crate::rbum::domain::{rbum_item, rbum_kind_attr};
    use crate::rbum::dto::filer_dto::RbumBasicFilterReq;
    use crate::rbum::dto::rbum_kind_attr_dto::{RbumKindAttrAddReq, RbumKindAttrDetailResp, RbumKindAttrModifyReq, RbumKindAttrSummaryResp};
    use crate::rbum::enumeration::RbumScopeKind;

    pub async fn add_rbum_kind_attr<'a>(
        rel_rbum_kind_id: &str,
        rbum_kind_attr_add_req: &RbumKindAttrAddReq,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<String> {
        let rbum_kind_attr_id: String = db
            .insert_one(
                rbum_kind_attr::ActiveModel {
                    name: Set(rbum_kind_attr_add_req.name.to_string()),
                    label: Set(rbum_kind_attr_add_req.label.to_string()),
                    data_type_kind: Set(rbum_kind_attr_add_req.data_type_kind.to_string()),
                    widget_type: Set(rbum_kind_attr_add_req.widget_type.to_string()),
                    note: Set(rbum_kind_attr_add_req.note.as_ref().unwrap_or(&"".to_string()).to_string()),
                    sort: Set(rbum_kind_attr_add_req.sort.unwrap_or(0)),
                    main_column: Set(rbum_kind_attr_add_req.main_column.unwrap_or(false)),
                    position: Set(rbum_kind_attr_add_req.position.unwrap_or(false)),
                    capacity: Set(rbum_kind_attr_add_req.capacity.unwrap_or(false)),
                    overload: Set(rbum_kind_attr_add_req.overload.unwrap_or(false)),
                    default_value: Set(rbum_kind_attr_add_req.default_value.as_ref().unwrap_or(&"".to_string()).to_string()),
                    options: Set(rbum_kind_attr_add_req.options.as_ref().unwrap_or(&"".to_string()).to_string()),
                    required: Set(rbum_kind_attr_add_req.required.unwrap_or(false)),
                    min_length: Set(rbum_kind_attr_add_req.min_length.unwrap_or(0)),
                    max_length: Set(rbum_kind_attr_add_req.max_length.unwrap_or(0)),
                    action: Set(rbum_kind_attr_add_req.action.as_ref().unwrap_or(&"".to_string()).to_string()),
                    rel_rbum_kind_id: Set(rel_rbum_kind_id.to_string()),
                    scope_kind: Set(rbum_kind_attr_add_req.scope_kind.as_ref().unwrap_or(&RbumScopeKind::App).to_string()),
                    ..Default::default()
                },
                cxt,
            )
            .await?
            .last_insert_id;
        Ok(rbum_kind_attr_id)
    }

    pub async fn modify_rbum_kind_attr<'a>(id: &str, rbum_kind_attr_modify_req: &RbumKindAttrModifyReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
        let mut rbum_kind_attr = rbum_kind_attr::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(name) = &rbum_kind_attr_modify_req.name {
            rbum_kind_attr.name = Set(name.to_string());
        }
        if let Some(label) = &rbum_kind_attr_modify_req.label {
            rbum_kind_attr.label = Set(label.to_string());
        }
        if let Some(data_type_kind) = &rbum_kind_attr_modify_req.data_type_kind {
            rbum_kind_attr.data_type_kind = Set(data_type_kind.to_string());
        }
        if let Some(widget_type) = &rbum_kind_attr_modify_req.widget_type {
            rbum_kind_attr.widget_type = Set(widget_type.to_string());
        }
        if let Some(note) = &rbum_kind_attr_modify_req.note {
            rbum_kind_attr.note = Set(note.to_string());
        }
        if let Some(sort) = rbum_kind_attr_modify_req.sort {
            rbum_kind_attr.sort = Set(sort);
        }
        if let Some(main_column) = rbum_kind_attr_modify_req.main_column {
            rbum_kind_attr.main_column = Set(main_column);
        }
        if let Some(position) = rbum_kind_attr_modify_req.position {
            rbum_kind_attr.position = Set(position);
        }
        if let Some(capacity) = rbum_kind_attr_modify_req.capacity {
            rbum_kind_attr.capacity = Set(capacity);
        }
        if let Some(overload) = rbum_kind_attr_modify_req.overload {
            rbum_kind_attr.overload = Set(overload);
        }
        if let Some(default_value) = &rbum_kind_attr_modify_req.default_value {
            rbum_kind_attr.default_value = Set(default_value.to_string());
        }
        if let Some(options) = &rbum_kind_attr_modify_req.options {
            rbum_kind_attr.options = Set(options.to_string());
        }
        if let Some(required) = rbum_kind_attr_modify_req.required {
            rbum_kind_attr.required = Set(required);
        }
        if let Some(min_length) = rbum_kind_attr_modify_req.min_length {
            rbum_kind_attr.min_length = Set(min_length);
        }
        if let Some(max_length) = rbum_kind_attr_modify_req.max_length {
            rbum_kind_attr.max_length = Set(max_length);
        }
        if let Some(action) = &rbum_kind_attr_modify_req.action {
            rbum_kind_attr.default_value = Set(action.to_string());
        }
        if let Some(scope_kind) = &rbum_kind_attr_modify_req.scope_kind {
            rbum_kind_attr.scope_kind = Set(scope_kind.to_string());
        }
        db.update_one(rbum_kind_attr, cxt).await?;
        Ok(())
    }

    pub async fn delete_rbum_kind_attr<'a>(id: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        db.soft_delete(rbum_kind_attr::Entity::find().filter(rbum_kind_attr::Column::Id.eq(id)), &cxt.account_id).await
    }

    pub async fn get_rbum_kind_attr<'a>(id: &str, filter: &RbumBasicFilterReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<RbumKindAttrDetailResp> {
        let mut query = package_rbum_kind_attr_query(true, filter, cxt);
        query.and_where(Expr::tbl(rbum_kind_attr::Entity, rbum_kind_attr::Column::Id).eq(id.to_string()));
        let query = db.get_dto(&query).await?;
        match query {
            Some(rbum_kind_attr) => Ok(rbum_kind_attr),
            // TODO
            None => Err(TardisError::NotFound("".to_string())),
        }
    }

    pub async fn find_rbum_kind_attrs<'a>(
        filter: &RbumBasicFilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_update: Option<bool>,
        db: &TardisRelDBlConnection<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumKindAttrSummaryResp>> {
        let mut query = package_rbum_kind_attr_query(false, filter, cxt);
        if let Some(sort) = desc_sort_by_update {
            query.order_by(rbum_kind_attr::Column::UpdateTime, if sort { Order::Desc } else { Order::Asc });
        }
        let (records, total_size) = db.paginate_dtos(&query, page_number, page_size).await?;
        Ok(TardisPage {
            page_size,
            page_number,
            total_size,
            records,
        })
    }

    pub fn package_rbum_kind_attr_query(is_detail: bool, filter: &RbumBasicFilterReq, cxt: &TardisContext) -> SelectStatement {
        let updater_table = Alias::new("updater");
        let rel_app_table = Alias::new("relApp");
        let rel_tenant_table = Alias::new("relTenant");

        let mut query = Query::select();
        query
            .columns(vec![
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Id),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Name),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Label),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Note),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Sort),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::MainColumn),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Position),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Capacity),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Overload),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::DataTypeKind),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::WidgetType),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::DefaultValue),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Options),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Required),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::MinLength),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::MaxLength),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::Action),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::RelRbumKindId),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::RelAppId),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::RelTenantId),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::UpdaterId),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::CreateTime),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::UpdateTime),
                (rbum_kind_attr::Entity, rbum_kind_attr::Column::ScopeKind),
            ])
            .from(rbum_kind_attr::Entity);

        if filter.rel_cxt_app {
            query.and_where(Expr::tbl(rbum_kind_attr::Entity, rbum_kind_attr::Column::RelAppId).eq(cxt.app_id.as_str()));
        }
        if filter.rel_cxt_tenant {
            query.and_where(Expr::tbl(rbum_kind_attr::Entity, rbum_kind_attr::Column::RelTenantId).eq(cxt.tenant_id.as_str()));
        }
        if filter.rel_cxt_updater {
            query.and_where(Expr::tbl(rbum_kind_attr::Entity, rbum_kind_attr::Column::UpdaterId).eq(cxt.account_id.as_str()));
        }
        if let Some(scope_kind) = &filter.scope_kind {
            query.and_where(Expr::tbl(rbum_kind_attr::Entity, rbum_kind_attr::Column::ScopeKind).eq(scope_kind.to_string()));
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
                    Expr::tbl(rel_app_table, rbum_item::Column::Id).equals(rbum_kind_attr::Entity, rbum_kind_attr::Column::RelAppId),
                )
                .join_as(
                    JoinType::InnerJoin,
                    rbum_item::Entity,
                    rel_tenant_table.clone(),
                    Expr::tbl(rel_tenant_table, rbum_item::Column::Id).equals(rbum_kind_attr::Entity, rbum_kind_attr::Column::RelTenantId),
                )
                .join_as(
                    JoinType::InnerJoin,
                    rbum_item::Entity,
                    updater_table.clone(),
                    Expr::tbl(updater_table, rbum_item::Column::Id).equals(rbum_kind_attr::Entity, rbum_kind_attr::Column::UpdaterId),
                );
        }

        query
    }
}
