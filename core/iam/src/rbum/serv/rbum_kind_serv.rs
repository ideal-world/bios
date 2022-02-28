use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::{TardisActiveModel, TardisSeaORMExtend};
use tardis::db::sea_orm::*;
use tardis::db::sea_query::*;
use tardis::TardisFuns;
use tardis::web::web_resp::TardisPage;

use crate::rbum::domain::{rbum_item, rbum_kind};
use crate::rbum::dto::filer_dto::RbumBasicFilterReq;
use crate::rbum::dto::rbum_kind_dto::{RbumKindAddReq, RbumKindDetailResp, RbumKindModifyReq, RbumKindSummaryResp};

pub async fn add_rbum_kind<'a, C: ConnectionTrait>(rbum_kind_add_req: &RbumKindAddReq, tx: &'a C, cxt: &TardisContext) -> TardisResult<String> {
    let rbum_kind = rbum_kind::ActiveModel {
        id: Set(rbum_kind_add_req.id.to_string()),
        name: Set(rbum_kind_add_req.name.to_string()),
        note: Set(rbum_kind_add_req.note.to_string()),
        icon: Set(rbum_kind_add_req.icon.to_string()),
        sort: Set(rbum_kind_add_req.sort),
        ext_table_name: Set(rbum_kind_add_req.ext_table_name.to_string()),
        rel_app_id: if let Some(rel_app_id) = &rbum_kind_add_req.rel_app_id {
            Set(rel_app_id.to_string())
        } else {
            NotSet
        },
        scope_kind: Set(rbum_kind_add_req.scope_kind.to_string()),
        ..Default::default()
    }
    .insert_cust(tx, cxt)
    .await?;
    Ok(rbum_kind.id)
}

pub async fn modify_rbum_kind<'a, C: ConnectionTrait>(id: &str, rbum_kind_modify_req: &RbumKindModifyReq, tx: &'a C, cxt: &TardisContext) -> TardisResult<()> {
    let mut rbum_kind = rbum_kind::ActiveModel { ..Default::default() };
    rbum_kind.id = Set(id.to_string());
    if let Some(rel_app_id) = &rbum_kind_modify_req.rel_app_id {
        rbum_kind.rel_app_id = Set(rel_app_id.to_string());
    }
    if let Some(scope_kind) = &rbum_kind_modify_req.scope_kind {
        rbum_kind.scope_kind = Set(scope_kind.to_string());
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
    rbum_kind.update_cust(tx, cxt).await?;
    Ok(())
}

pub async fn delete_rbum_kind<'a, C: ConnectionTrait>(id: &str, tx: &'a C, cxt: &TardisContext) -> TardisResult<usize> {
    rbum_kind::Entity::find().filter(rbum_kind::Column::Id.eq(id)).soft_delete(tx, &cxt.account_id).await
}

pub async fn peek_rbum_kind<'a, C: ConnectionTrait>(id: &str, filter: &RbumBasicFilterReq, tx: &'a C, cxt: &TardisContext) -> TardisResult<RbumKindSummaryResp> {
    let mut query = rbum_kind::Entity::find_by_id(id.to_string());
    if filter.rel_cxt_app {
        query = query.filter(rbum_kind::Column::RelAppId.eq(cxt.app_id.as_str()));
    }
    if filter.rel_cxt_tenant {
        query = query.filter(rbum_kind::Column::RelTenantId.eq(cxt.tenant_id.as_str()));
    }
    if filter.rel_cxt_creator {
        query = query.filter(rbum_kind::Column::CreatorId.eq(cxt.account_id.as_str()));
    }
    if filter.rel_cxt_updater {
        query = query.filter(rbum_kind::Column::UpdaterId.eq(cxt.account_id.as_str()));
    }
    if let Some(scope_kind) = &filter.scope_kind {
        query = query.filter(rbum_kind::Column::ScopeKind.eq(scope_kind.to_string()));
    }
    let query = query.into_model::<RbumKindSummaryResp>().one(tx).await?;
    match query {
        Some(rbum_kind) => Ok(rbum_kind),
        // TODO
        None => Err(TardisError::NotFound("".to_string())),
    }
}

pub async fn get_rbum_kind<'a, C: ConnectionTrait>(id: &str, filter: &RbumBasicFilterReq, tx: &'a C, cxt: &TardisContext) -> TardisResult<RbumKindDetailResp> {
    let mut query = package_rbum_kind_query(filter, cxt);
    query.and_where(Expr::tbl(rbum_kind::Entity, rbum_kind::Column::Id).eq(id.to_string()));
    let query = TardisFuns::reldb().get_dto(&query, tx).await?;
    match query {
        Some(rbum_kind) => Ok(rbum_kind),
        // TODO
        None => Err(TardisError::NotFound("".to_string())),
    }
}

pub async fn find_rbum_kinds<'a, C: ConnectionTrait>(
    filter: &RbumBasicFilterReq,
    page_number: u64,
    page_size: u64,
    tx: &'a C,
    cxt: &TardisContext,
) -> TardisResult<TardisPage<RbumKindDetailResp>> {
    let query = package_rbum_kind_query(filter, cxt);
    let (records, total_size) = TardisFuns::reldb().paginate_dtos(&query, page_number, page_size, tx).await?;
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
            (rbum_kind::Entity, rbum_kind::Column::Id),
            (rbum_kind::Entity, rbum_kind::Column::Name),
            (rbum_kind::Entity, rbum_kind::Column::Note),
            (rbum_kind::Entity, rbum_kind::Column::Icon),
            (rbum_kind::Entity, rbum_kind::Column::Sort),
            (rbum_kind::Entity, rbum_kind::Column::ScopeKind),
            (rbum_kind::Entity, rbum_kind::Column::ExtTableName),
            (rbum_kind::Entity, rbum_kind::Column::CreatorId),
            (rbum_kind::Entity, rbum_kind::Column::CreateTime),
            (rbum_kind::Entity, rbum_kind::Column::UpdaterId),
            (rbum_kind::Entity, rbum_kind::Column::UpdateTime),
            (rbum_kind::Entity, rbum_kind::Column::RelAppId),
            (rbum_kind::Entity, rbum_kind::Column::RelTenantId),
        ])
        .expr_as(Expr::tbl(creator_table.clone(), rbum_item::Column::Name), Alias::new("creator_name"))
        .expr_as(Expr::tbl(updater_table.clone(), rbum_item::Column::Name), Alias::new("updater_name"))
        .expr_as(Expr::tbl(rel_app_table.clone(), rbum_item::Column::Name), Alias::new("rel_app_name"))
        .expr_as(Expr::tbl(rel_tenant_table.clone(), rbum_item::Column::Name), Alias::new("rel_tenant_name"))
        .from(rbum_kind::Entity)
        .join_as(
            JoinType::InnerJoin,
            rbum_item::Entity,
            creator_table.clone(),
            Expr::tbl(creator_table, rbum_item::Column::Id).equals(rbum_kind::Entity, rbum_kind::Column::CreatorId),
        )
        .join_as(
            JoinType::InnerJoin,
            rbum_item::Entity,
            updater_table.clone(),
            Expr::tbl(updater_table, rbum_item::Column::Id).equals(rbum_kind::Entity, rbum_kind::Column::UpdaterId),
        )
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
        );

    if filter.rel_cxt_app {
        query.and_where(Expr::tbl(rbum_kind::Entity, rbum_kind::Column::RelAppId).eq(cxt.app_id.as_str()));
    }
    if filter.rel_cxt_tenant {
        query.and_where(Expr::tbl(rbum_kind::Entity, rbum_kind::Column::RelTenantId).eq(cxt.tenant_id.as_str()));
    }
    if filter.rel_cxt_creator {
        query.and_where(Expr::tbl(rbum_kind::Entity, rbum_kind::Column::CreatorId).eq(cxt.account_id.as_str()));
    }
    if filter.rel_cxt_updater {
        query.and_where(Expr::tbl(rbum_kind::Entity, rbum_kind::Column::UpdaterId).eq(cxt.account_id.as_str()));
    }
    if let Some(scope_kind) = &filter.scope_kind {
        query.and_where(Expr::tbl(rbum_kind::Entity, rbum_kind::Column::ScopeKind).eq(scope_kind.to_string()));
    }
    query
}
