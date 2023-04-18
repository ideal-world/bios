use serde::{Deserialize, Serialize};
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm;
use tardis::db::sea_orm::sea_query::*;
use tardis::TardisFuns;
use tardis::TardisFunsInst;

use crate::rbum::domain::{
    rbum_cert, rbum_cert_conf, rbum_domain, rbum_item, rbum_item_attr, rbum_kind, rbum_kind_attr, rbum_rel, rbum_rel_attr, rbum_rel_env, rbum_set, rbum_set_cate, rbum_set_item,
};
use crate::rbum::rbum_config::{RbumConfig, RbumConfigManager};

pub async fn init(code: &str, config: RbumConfig) -> TardisResult<()> {
    RbumConfigManager::add(code, config)?;
    TardisFuns::reldb_by_module_or_default(code).init_basic_tables().await?;
    let db_kind = TardisFuns::reldb_by_module_or_default(code).backend();
    let mut tx = TardisFuns::reldb_by_module_or_default(code).conn();
    let compatible_type = TardisFuns::reldb_by_module_or_default(code).compatible_type();
    if TardisFuns::dict.get("__RBUM_INIT__", &tx).await?.is_some() {
        return Ok(());
    }
    tx.begin().await?;
    TardisFuns::dict.add("__RBUM_INIT__", "", "", &tx).await?;
    tx.init(rbum_domain::ActiveModel::init(db_kind, Some("update_time"), compatible_type.clone())).await?;
    tx.init(rbum_kind::ActiveModel::init(db_kind, Some("update_time"), compatible_type.clone())).await?;
    tx.init(rbum_item::ActiveModel::init(db_kind, Some("update_time"), compatible_type.clone())).await?;
    tx.init(rbum_kind_attr::ActiveModel::init(db_kind, Some("update_time"), compatible_type.clone())).await?;
    tx.init(rbum_item_attr::ActiveModel::init(db_kind, Some("update_time"), compatible_type.clone())).await?;
    tx.init(rbum_rel::ActiveModel::init(db_kind, Some("update_time"), compatible_type.clone())).await?;
    tx.init(rbum_rel_attr::ActiveModel::init(db_kind, Some("update_time"), compatible_type.clone())).await?;
    tx.init(rbum_rel_env::ActiveModel::init(db_kind, Some("update_time"), compatible_type.clone())).await?;
    tx.init(rbum_cert_conf::ActiveModel::init(db_kind, Some("update_time"), compatible_type.clone())).await?;
    tx.init(rbum_cert::ActiveModel::init(db_kind, Some("update_time"), compatible_type.clone())).await?;
    tx.init(rbum_set::ActiveModel::init(db_kind, Some("update_time"), compatible_type.clone())).await?;
    tx.init(rbum_set_cate::ActiveModel::init(db_kind, Some("update_time"), compatible_type.clone())).await?;
    tx.init(rbum_set_item::ActiveModel::init(db_kind, Some("update_time"), compatible_type.clone())).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn get_first_account_context<'a>(rbum_kind_code: &str, rbum_domain_code: &str, funs: &TardisFunsInst) -> TardisResult<Option<TardisContext>> {
    #[derive(Deserialize, sea_orm::FromQueryResult, Serialize, Clone, Debug)]
    struct TmpContext {
        pub id: String,
        pub own_paths: String,
    }

    let mut query = Query::select();
    query
        .column((rbum_item::Entity, rbum_item::Column::Id))
        .column((rbum_item::Entity, rbum_item::Column::OwnPaths))
        .from(rbum_item::Entity)
        .inner_join(
            rbum_kind::Entity,
            Expr::col((rbum_kind::Entity, rbum_kind::Column::Id)).equals((rbum_item::Entity, rbum_item::Column::RelRbumKindId)),
        )
        .inner_join(
            rbum_domain::Entity,
            Expr::col((rbum_domain::Entity, rbum_domain::Column::Id)).equals((rbum_item::Entity, rbum_item::Column::RelRbumDomainId)),
        )
        .and_where(Expr::col((rbum_kind::Entity, rbum_kind::Column::Code)).eq(rbum_kind_code))
        .and_where(Expr::col((rbum_domain::Entity, rbum_domain::Column::Code)).eq(rbum_domain_code))
        .order_by((rbum_item::Entity, rbum_item::Column::CreateTime), Order::Asc);

    let context: Option<TmpContext> = funs.db().get_dto(&query).await?;

    if let Some(context) = context {
        Ok(Some(TardisContext {
            own_paths: context.own_paths.to_string(),
            owner: context.id,
            ak: "_".to_string(),
            roles: vec![],
            groups: vec![],
            ext: Default::default(),
        }))
    } else {
        Ok(None)
    }
}

pub async fn truncate_data<'a>(funs: &TardisFunsInst) -> TardisResult<()> {
    funs.db().execute(Table::truncate().table(rbum_cert::Entity)).await?;
    funs.db().execute(Table::truncate().table(rbum_cert_conf::Entity)).await?;
    funs.db().execute(Table::truncate().table(rbum_domain::Entity)).await?;
    funs.db().execute(Table::truncate().table(rbum_item::Entity)).await?;
    funs.db().execute(Table::truncate().table(rbum_item_attr::Entity)).await?;
    funs.db().execute(Table::truncate().table(rbum_kind::Entity)).await?;
    funs.db().execute(Table::truncate().table(rbum_kind_attr::Entity)).await?;
    funs.db().execute(Table::truncate().table(rbum_rel::Entity)).await?;
    funs.db().execute(Table::truncate().table(rbum_rel_attr::Entity)).await?;
    funs.db().execute(Table::truncate().table(rbum_rel_env::Entity)).await?;
    funs.db().execute(Table::truncate().table(rbum_set::Entity)).await?;
    funs.db().execute(Table::truncate().table(rbum_set_cate::Entity)).await?;
    funs.db().execute(Table::truncate().table(rbum_set_item::Entity)).await?;
    funs.cache().flushdb().await?;
    Ok(())
}
