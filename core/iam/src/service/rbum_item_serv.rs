use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::{TardisActiveModel, TardisSeaORMExtend};
use tardis::db::sea_orm::*;
use tardis::db::sea_query::*;
use tardis::TardisFuns;

use crate::domain::rbum_item;
use crate::dto::filer_dto::RbumBasicFilterReq;
use crate::dto::rbum_item_dto::{RbumItemAddReq, RbumItemDetailResp, RbumItemModifyReq, RbumItemSummaryResp};

pub async fn add_rbum_item(rbum_item_add_req: &RbumItemAddReq, cxt: &TardisContext) -> TardisResult<String> {
    let tx = TardisFuns::reldb().conn().begin().await?;
    let rbum_item = rbum_item::ActiveModel {
        code: Set(rbum_item_add_req.code.to_string()),
        name: Set(rbum_item_add_req.name.to_string()),
        uri_part: Set(rbum_item_add_req.uri_part.to_string()),
        icon: Set(rbum_item_add_req.icon.to_string()),
        sort: Set(rbum_item_add_req.sort),
        rel_rbum_kind_id: Set(rbum_item_add_req.rel_rbum_kind_id.to_string()),
        rel_rbum_domain_id: Set(rbum_item_add_req.rel_rbum_domain_id.to_string()),
        ..Default::default()
    }
    .insert_cust(&tx, cxt)
    .await
    .unwrap();
    tx.commit().await?;
    Ok(rbum_item.id)
}

pub async fn modify_rbum_item(id: &str, rbum_item_modify_req: &RbumItemModifyReq, cxt: &TardisContext) -> TardisResult<()> {
    let mut rbum_item = rbum_item::ActiveModel { ..Default::default() };
    rbum_item.id = Set(id.to_string());
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
    if let Some(scope_kind) = &rbum_item_modify_req.scope_kind {
        rbum_item.scope_kind = Set(scope_kind.to_string());
    }
    if let Some(disabled) = &rbum_item_modify_req.disabled {
        rbum_item.disabled = Set(disabled);
    }
    let tx = TardisFuns::reldb().conn().begin().await?;
    rbum_item.update_cust(&tx, cxt).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn delete_rbum_item(id: &str, cxt: &TardisContext) -> TardisResult<()> {
    let tx = TardisFuns::reldb().conn().begin().await?;
    rbum_item::Entity::find().filter(rbum_item::Column::Id.eq(id)).soft_delete(&tx, &cxt.account_id).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn peek_rbum_item(id: &str, filter: &RbumBasicFilterReq, cxt: &TardisContext) -> TardisResult<RbumItemSummaryResp> {
    let mut query = rbum_item::Entity::find_by_id(id.to_string());
    if filter.rel_cxt_app {
        query = query.filter(rbum_item::Column::RelAppId.eq(cxt.app_id.as_str()));
    }
    if filter.rel_cxt_tenant {
        query = query.filter(rbum_item::Column::RelTenantId.eq(cxt.tenant_id.as_str()));
    }
    if filter.rel_cxt_creator {
        query = query.filter(rbum_item::Column::CreatorId.eq(cxt.account_id.as_str()));
    }
    if filter.rel_cxt_updater {
        query = query.filter(rbum_item::Column::UpdaterId.eq(cxt.account_id.as_str()));
    }
    if let Some(scope_kind) = &filter.scope_kind {
        query = query.filter(rbum_item::Column::ScopeKind.eq(scope_kind.as_str()));
    }
    let query = query.into_model::<RbumItemSummaryResp>().one(TardisFuns::reldb().conn()).await?;
    match query {
        Some(rbum_item) => Ok(rbum_item),
        // TODO
        None => Err(TardisError::NotFound("".to_string())),
    }
}

pub async fn get_rbum_item(id: &str, filter: &RbumBasicFilterReq, cxt: &TardisContext) -> TardisResult<RbumItemDetailResp> {
    let creator_table = Alias::new("creator");
    let updater_table = Alias::new("updater");
    let rel_app_table = Alias::new("relApp");
    let rel_tenant_table = Alias::new("relTenant");

    let mut query = Query::select();
    query
        .columns(vec![
            (rbum_item::Entity, rbum_item::Column::Id),
            (rbum_item::Entity, rbum_item::Column::Code),
            (rbum_item::Entity, rbum_item::Column::Name),
            (rbum_item::Entity, rbum_item::Column::UriPart),
            (rbum_item::Entity, rbum_item::Column::Icon),
            (rbum_item::Entity, rbum_item::Column::Sort),
            (rbum_item::Entity, rbum_item::Column::ScopeKind),
            (rbum_item::Entity, rbum_item::Column::Disabled),
            (rbum_item::Entity, rbum_item::Column::CreateTime),
            (rbum_item::Entity, rbum_item::Column::UpdateTime),
        ])
        .expr_as(Expr::tbl(creator_table.clone(), rbum_item::Column::Name), Alias::new("creator_name"))
        .expr_as(Expr::tbl(updater_table.clone(), rbum_item::Column::Name), Alias::new("updater_name"))
        .expr_as(Expr::tbl(rel_app_table.clone(), rbum_item::Column::Name), Alias::new("rel_app_name"))
        .expr_as(Expr::tbl(rel_tenant_table.clone(), rbum_item::Column::Name), Alias::new("rel_tenant_name"))
        // TODO  rel_rbum_kind_id rel_rbum_domain_id
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
        .and_where(Expr::tbl(rbum_item::Entity, rbum_item::Column::Id).eq(id.to_string()));

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
        query.and_where(Expr::tbl(rbum_item::Entity, rbum_item::Column::ScopeKind).eq(scope_kind.as_str()));
    }

    let query = TardisFuns::reldb().get_dto(&query, TardisFuns::reldb().conn()).await?;
    match query {
        Some(rbum_item) => Ok(rbum_item),
        // TODO
        None => Err(TardisError::NotFound("".to_string())),
    }
}
