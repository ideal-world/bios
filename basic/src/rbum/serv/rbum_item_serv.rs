use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisRelDBlConnection;
use tardis::db::sea_orm::*;
use tardis::db::sea_query::*;
use tardis::web::web_resp::TardisPage;

use crate::rbum::domain::{rbum_item, rbum_kind};
use crate::rbum::dto::filer_dto::RbumBasicFilterReq;
use crate::rbum::dto::rbum_item_dto::{RbumItemAddReq, RbumItemDetailResp, RbumItemModifyReq, RbumItemSummaryResp};

pub async fn add_rbum_item<'a>(
    id: &str,
    rbum_kind_id: &str,
    rbum_item_add_req: &RbumItemAddReq,
    rel_app_id: Option<String>,
    db: &TardisRelDBlConnection<'a>,
    cxt: &TardisContext,
) -> TardisResult<String> {
    // TODO 检查rel_rbum_kind_id是否存在且合法
    // TODO 检查rel_rbum_domain_id是否存在且合法
    // TODO 检查rel_app_id是否存在且合法
    let rbum_item_id: String = db
        .insert_one(
            rbum_item::ActiveModel {
                id: Set(id.to_string()),
                name: Set(rbum_item_add_req.name.to_string()),
                uri_part: Set(rbum_item_add_req.uri_part.to_string()),
                icon: Set(rbum_item_add_req.icon.to_string()),
                sort: Set(rbum_item_add_req.sort),
                rel_rbum_kind_id: Set(rbum_kind_id.to_string()),
                rel_rbum_domain_id: Set(rbum_item_add_req.rel_rbum_domain_id.to_string()),
                rel_app_id: if let Some(rel_app_id) = &rel_app_id { Set(rel_app_id.to_string()) } else { NotSet },
                scope_kind: Set(rbum_item_add_req.scope_kind.to_string()),
                disabled: Set(rbum_item_add_req.disabled),
                ..Default::default()
            },
            cxt,
        )
        .await?
        .last_insert_id;
    Ok(rbum_item_id)
}

pub async fn modify_rbum_item<'a>(id: &str, rbum_item_modify_req: &RbumItemModifyReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
    // TODO 检查id是否存在且合法
    let mut rbum_item = rbum_item::ActiveModel { ..Default::default() };
    rbum_item.id = Set(id.to_string());
    if let Some(scope_kind) = &rbum_item_modify_req.scope_kind {
        rbum_item.scope_kind = Set(scope_kind.to_string());
    }
    if let Some(disabled) = rbum_item_modify_req.disabled {
        rbum_item.disabled = Set(disabled);
    }
    if let Some(name) = &rbum_item_modify_req.name {
        rbum_item.name = Set(name.to_string());
    }
    if let Some(uri_part) = &rbum_item_modify_req.uri_part {
        rbum_item.uri_part = Set(uri_part.to_string());
    }
    if let Some(icon) = &rbum_item_modify_req.icon {
        rbum_item.icon = Set(icon.to_string());
    }
    if let Some(sort) = rbum_item_modify_req.sort {
        rbum_item.sort = Set(sort);
    }
    db.update_one(rbum_item, cxt).await?;
    Ok(())
}

pub async fn delete_rbum_item<'a>(id: &str, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<u64> {
    // TODO 检查id是否存在且合法
    db.soft_delete(rbum_item::Entity::find().filter(rbum_item::Column::Id.eq(id)), &cxt.account_id).await
}

pub async fn peek_rbum_item<'a>(id: &str, filter: &RbumBasicFilterReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<RbumItemSummaryResp> {
    let mut query = Query::select();
    query.columns(vec![rbum_item::Column::Id, rbum_item::Column::Name, rbum_item::Column::Icon, rbum_item::Column::Sort]);
    query.from(rbum_item::Entity).and_where(rbum_item::Column::Id.eq(id));
    if filter.rel_cxt_app {
        query.and_where(rbum_item::Column::RelAppId.eq(cxt.app_id.as_str()));
    }
    if filter.rel_cxt_tenant {
        query.and_where(rbum_item::Column::RelTenantId.eq(cxt.tenant_id.as_str()));
    }
    if filter.rel_cxt_creator {
        query.and_where(rbum_item::Column::CreatorId.eq(cxt.account_id.as_str()));
    }
    if filter.rel_cxt_updater {
        query.and_where(rbum_item::Column::UpdaterId.eq(cxt.account_id.as_str()));
    }
    if let Some(scope_kind) = &filter.scope_kind {
        query.and_where(rbum_item::Column::ScopeKind.eq(scope_kind.to_string()));
    }
    if let Some(kind_id) = &filter.kind_id {
        query.and_where(rbum_item::Column::RelRbumKindId.eq(kind_id.to_string()));
    }
    if let Some(domain_id) = &filter.domain_id {
        query.and_where(rbum_item::Column::RelRbumDomainId.eq(domain_id.to_string()));
    }
    let query = db.get_dto(&query).await?;
    match query {
        Some(rbum_item) => Ok(rbum_item),
        // TODO
        None => Err(TardisError::NotFound("".to_string())),
    }
}

pub async fn get_rbum_item<'a>(id: &str, filter: &RbumBasicFilterReq, db: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<RbumItemDetailResp> {
    let mut query = package_rbum_kind_query(filter, cxt);
    query.and_where(Expr::tbl(rbum_item::Entity, rbum_item::Column::Id).eq(id.to_string()));
    let query = db.get_dto(&query).await?;
    match query {
        Some(rbum_kind) => Ok(rbum_kind),
        // TODO
        None => Err(TardisError::NotFound("".to_string())),
    }
}

pub async fn find_rbum_items<'a>(
    filter: &RbumBasicFilterReq,
    page_number: u64,
    page_size: u64,
    desc_sort_by_update: Option<bool>,
    db: &TardisRelDBlConnection<'a>,
    cxt: &TardisContext,
) -> TardisResult<TardisPage<RbumItemDetailResp>> {
    let mut query = package_rbum_kind_query(filter, cxt);
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

fn package_rbum_kind_query(filter: &RbumBasicFilterReq, cxt: &TardisContext) -> SelectStatement {
    let creator_table = Alias::new("creator");
    let updater_table = Alias::new("updater");
    let rel_app_table = Alias::new("relApp");
    let rel_tenant_table = Alias::new("relTenant");

    let mut query = Query::select();
    query
        .columns(vec![
            (rbum_item::Entity, rbum_item::Column::Id),
            (rbum_item::Entity, rbum_item::Column::Name),
            (rbum_item::Entity, rbum_item::Column::UriPart),
            (rbum_item::Entity, rbum_item::Column::Icon),
            (rbum_item::Entity, rbum_item::Column::Sort),
            (rbum_item::Entity, rbum_item::Column::ScopeKind),
            (rbum_item::Entity, rbum_item::Column::Disabled),
            (rbum_item::Entity, rbum_item::Column::CreatorId),
            (rbum_item::Entity, rbum_item::Column::CreateTime),
            (rbum_item::Entity, rbum_item::Column::UpdaterId),
            (rbum_item::Entity, rbum_item::Column::UpdateTime),
            (rbum_item::Entity, rbum_item::Column::RelAppId),
            (rbum_item::Entity, rbum_item::Column::RelTenantId),
            (rbum_item::Entity, rbum_item::Column::RelRbumKindId),
            (rbum_item::Entity, rbum_item::Column::RelRbumDomainId),
        ])
        .expr_as(Expr::tbl(creator_table.clone(), rbum_item::Column::Name), Alias::new("creator_name"))
        .expr_as(Expr::tbl(updater_table.clone(), rbum_item::Column::Name), Alias::new("updater_name"))
        .expr_as(Expr::tbl(rel_app_table.clone(), rbum_item::Column::Name), Alias::new("rel_app_name"))
        .expr_as(Expr::tbl(rel_tenant_table.clone(), rbum_item::Column::Name), Alias::new("rel_tenant_name"))
        .expr_as(Expr::tbl(rbum_kind::Entity, rbum_kind::Column::Name), Alias::new("rel_rbum_kind_name"))
        // TODO RelRbumDomainId
        .from(rbum_item::Entity)
        .join_as(
            JoinType::InnerJoin,
            rbum_item::Entity,
            creator_table.clone(),
            Expr::tbl(creator_table, rbum_item::Column::Id).equals(rbum_item::Entity, rbum_item::Column::CreatorId),
        )
        .join_as(
            JoinType::InnerJoin,
            rbum_item::Entity,
            updater_table.clone(),
            Expr::tbl(updater_table, rbum_item::Column::Id).equals(rbum_item::Entity, rbum_item::Column::UpdaterId),
        )
        .join_as(
            JoinType::InnerJoin,
            rbum_item::Entity,
            rel_app_table.clone(),
            Expr::tbl(rel_app_table, rbum_item::Column::Id).equals(rbum_item::Entity, rbum_item::Column::RelAppId),
        )
        .join_as(
            JoinType::InnerJoin,
            rbum_item::Entity,
            rel_tenant_table.clone(),
            Expr::tbl(rel_tenant_table, rbum_item::Column::Id).equals(rbum_item::Entity, rbum_item::Column::RelTenantId),
        )
        .inner_join(
            rbum_kind::Entity,
            Expr::tbl(rbum_kind::Entity, rbum_kind::Column::Id).equals(rbum_item::Entity, rbum_item::Column::RelRbumKindId),
        );

    if filter.rel_cxt_app {
        query.and_where(Expr::tbl(rbum_item::Entity, rbum_item::Column::RelAppId).eq(cxt.app_id.as_str()));
    }
    if filter.rel_cxt_tenant {
        query.and_where(Expr::tbl(rbum_item::Entity, rbum_item::Column::RelTenantId).eq(cxt.tenant_id.as_str()));
    }
    if filter.rel_cxt_creator {
        query.and_where(Expr::tbl(rbum_item::Entity, rbum_item::Column::CreatorId).eq(cxt.account_id.as_str()));
    }
    if filter.rel_cxt_updater {
        query.and_where(Expr::tbl(rbum_item::Entity, rbum_item::Column::UpdaterId).eq(cxt.account_id.as_str()));
    }
    if let Some(scope_kind) = &filter.scope_kind {
        query.and_where(Expr::tbl(rbum_item::Entity, rbum_item::Column::ScopeKind).eq(scope_kind.to_string()));
    }
    if let Some(kind_id) = &filter.kind_id {
        query.and_where(Expr::tbl(rbum_item::Entity, rbum_item::Column::RelRbumKindId).eq(kind_id.to_string()));
    }
    if let Some(domain_id) = &filter.domain_id {
        query.and_where(Expr::tbl(rbum_item::Entity, rbum_item::Column::RelRbumDomainId).eq(domain_id.to_string()));
    }
    query
}
