use chrono::Utc;
use sea_orm::sea_query::Query;
use sea_orm::ActiveValue::Set;
use sea_orm::*;
use sea_query::{Alias, Expr};
use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisSeaORMExtend;
use tardis::TardisFuns;

use crate::domain::{rbum_item, rbum_kind, BiosRelDBClient, BiosSeaORMExtend};
use crate::dto::rbum_kind_dto::{RbumKindAddReq, RbumKindDetailResp, RbumKindModifyReq, RbumKindSummaryResp};

pub async fn add_rbum_kind(rbum_kind_add_req: &RbumKindAddReq, cxt: &TardisContext) -> TardisResult<String> {
    let tx = TardisFuns::reldb().conn().begin().await?;
    let rbum_kind = rbum_kind::ActiveModel {
        code: Set(rbum_kind_add_req.code.to_string()),
        name: Set(rbum_kind_add_req.name.to_string()),
        note: Set(rbum_kind_add_req.note.to_string()),
        icon: Set(rbum_kind_add_req.icon.to_string()),
        sort: Set(rbum_kind_add_req.sort),
        ext_table_name: Set(rbum_kind_add_req.ext_table_name.to_string()),
        ..Default::default()
    }
    .insert_with_cxt(&tx, cxt)
    .await
    .unwrap();
    tx.commit().await?;
    Ok(rbum_kind.id)
}

pub async fn modify_rbum_kind(id: &str, rbum_kind_modify_req: &RbumKindModifyReq, cxt: &TardisContext) -> TardisResult<()> {
    let mut rbum_kind = rbum_kind::ActiveModel { ..Default::default() };
    rbum_kind.id = Set(id.to_string());
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
    if let Some(scope_kind) = &rbum_kind_modify_req.scope_kind {
        rbum_kind.scope_kind = Set(scope_kind.to_string());
    }
    if let Some(ext_table_name) = &rbum_kind_modify_req.ext_table_name {
        rbum_kind.ext_table_name = Set(ext_table_name.to_string());
    }
    let tx = TardisFuns::reldb().conn().begin().await?;
    rbum_kind.update_with_cxt(&tx, cxt).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn delete_rbum_kind(id: &str, cxt: &TardisContext) -> TardisResult<()> {
    let tx = TardisFuns::reldb().conn().begin().await?;
    rbum_kind::Entity::find().filter(rbum_kind::Column::Id.eq(id)).soft_delete(&tx, &cxt.account_id).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn peek_rbum_kind(id: &str, cxt: &TardisContext) -> TardisResult<RbumKindSummaryResp> {
    match rbum_kind::Entity::find_by_id(id.to_string()).into_model::<RbumKindSummaryResp>().one(TardisFuns::reldb().conn()).await? {
        Some(rbum_kind) => Ok(rbum_kind),
        // TODO
        None => Err(TardisError::NotFound("".to_string())),
    }
}

pub async fn get_rbum_kind(id: &str, cxt: &TardisContext) -> TardisResult<RbumKindDetailResp> {
    let creator_table = Alias::new("creator");
    let updater_table = Alias::new("updater");
    let rel_app_table = Alias::new("relApp");
    let rel_tenant_table = Alias::new("relTenant");

    // let dd = BiosRelDBClient::build_detail_resp(rbum_kind::Entity);

    let rbum_kind = BiosRelDBClient::get(
        Query::select()
            .columns(vec![
                (rbum_kind::Entity, rbum_kind::Column::Id),
                (rbum_kind::Entity, rbum_kind::Column::Code),
                (rbum_kind::Entity, rbum_kind::Column::Name),
                (rbum_kind::Entity, rbum_kind::Column::Note),
                (rbum_kind::Entity, rbum_kind::Column::Icon),
                (rbum_kind::Entity, rbum_kind::Column::Sort),
                (rbum_kind::Entity, rbum_kind::Column::ScopeKind),
                (rbum_kind::Entity, rbum_kind::Column::ExtTableName),
                (rbum_kind::Entity, rbum_kind::Column::CreateTime),
                (rbum_kind::Entity, rbum_kind::Column::UpdateTime),
            ])
            .expr_as(Expr::tbl(creator_table.clone(), rbum_item::Column::Name), Alias::new("creator_name"))
            .expr_as(Expr::tbl(updater_table.clone(), rbum_item::Column::Name), Alias::new("updater_name"))
            .expr_as(Expr::tbl(rel_app_table.clone(), rbum_item::Column::Name), Alias::new("rel_app_name"))
            .expr_as(Expr::tbl(rel_tenant_table.clone(), rbum_item::Column::Name), Alias::new("rel_tenant_name"))
            .from(rbum_kind::Entity)
            .join_as(
                JoinType::LeftJoin,
                rbum_item::Entity,
                creator_table.clone(),
                Expr::tbl(creator_table, rbum_item::Column::Id).equals(rbum_kind::Entity, rbum_kind::Column::CreatorId),
            )
            .join_as(
                JoinType::LeftJoin,
                rbum_item::Entity,
                updater_table.clone(),
                Expr::tbl(updater_table, rbum_item::Column::Id).equals(rbum_kind::Entity, rbum_kind::Column::UpdaterId),
            )
            .join_as(
                JoinType::LeftJoin,
                rbum_item::Entity,
                rel_app_table.clone(),
                Expr::tbl(rel_app_table, rbum_item::Column::Id).equals(rbum_kind::Entity, rbum_kind::Column::RelAppId),
            )
            .join_as(
                JoinType::LeftJoin,
                rbum_item::Entity,
                rel_tenant_table.clone(),
                Expr::tbl(rel_tenant_table, rbum_item::Column::Id).equals(rbum_kind::Entity, rbum_kind::Column::RelTenantId),
            )
            .and_where(Expr::tbl(rbum_kind::Entity, rbum_kind::Column::Id).eq(id.to_string())),
        TardisFuns::reldb().conn(),
        cxt,
    )
    .await?;
    match rbum_kind {
        Some(rbum_kind) => Ok(rbum_kind),
        // TODO
        None => Err(TardisError::NotFound("".to_string())),
    }
}
