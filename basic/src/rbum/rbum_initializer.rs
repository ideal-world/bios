use serde::{Deserialize, Serialize};
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm::*;
use tardis::db::sea_query::*;
use tardis::TardisFuns;
use tardis::TardisFunsInst;

use crate::rbum::domain::{
    rbum_cert, rbum_cert_conf, rbum_domain, rbum_item, rbum_item_attr, rbum_kind, rbum_kind_attr, rbum_rel, rbum_rel_attr, rbum_rel_env, rbum_set, rbum_set_cate, rbum_set_item,
};
use crate::rbum::rbum_config::{RbumConfig, RbumConfigManager};

pub async fn init(code: &str, config: RbumConfig) -> TardisResult<()> {
    RbumConfigManager::add(code, config)?;
    let db_kind = TardisFuns::reldb().backend();
    let mut tx = TardisFuns::reldb().conn();
    if TardisFuns::dict.get("__RBUM_INIT__", &tx).await?.is_some() {
        return Ok(());
    }
    tx.begin().await?;
    TardisFuns::dict.add("__RBUM_INIT__", "", "", &tx).await?;
    tx.create_table_and_index(&rbum_domain::ActiveModel::create_table_and_index_statement(db_kind)).await?;
    tx.create_table_and_index(&rbum_kind::ActiveModel::create_table_and_index_statement(db_kind)).await?;
    tx.create_table_and_index(&rbum_item::ActiveModel::create_table_and_index_statement(db_kind)).await?;
    tx.create_table_and_index(&rbum_kind_attr::ActiveModel::create_table_and_index_statement(db_kind)).await?;
    tx.create_table_and_index(&rbum_item_attr::ActiveModel::create_table_and_index_statement(db_kind)).await?;
    tx.create_table_and_index(&rbum_rel::ActiveModel::create_table_and_index_statement(db_kind)).await?;
    tx.create_table_and_index(&rbum_rel_attr::ActiveModel::create_table_and_index_statement(db_kind)).await?;
    tx.create_table_and_index(&rbum_rel_env::ActiveModel::create_table_and_index_statement(db_kind)).await?;
    tx.create_table_and_index(&rbum_cert_conf::ActiveModel::create_table_and_index_statement(db_kind)).await?;
    tx.create_table_and_index(&rbum_cert::ActiveModel::create_table_and_index_statement(db_kind)).await?;
    tx.create_table_and_index(&rbum_set::ActiveModel::create_table_and_index_statement(db_kind)).await?;
    tx.create_table_and_index(&rbum_set_cate::ActiveModel::create_table_and_index_statement(db_kind)).await?;
    tx.create_table_and_index(&rbum_set_item::ActiveModel::create_table_and_index_statement(db_kind)).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn get_first_account_context<'a>(rbum_kind_code: &str, rbum_domain_code: &str, funs: &TardisFunsInst<'a>) -> TardisResult<Option<TardisContext>> {
    #[derive(Deserialize, FromQueryResult, Serialize, Clone, Debug)]
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
            Expr::tbl(rbum_kind::Entity, rbum_kind::Column::Id).equals(rbum_item::Entity, rbum_item::Column::RelRbumKindId),
        )
        .inner_join(
            rbum_domain::Entity,
            Expr::tbl(rbum_domain::Entity, rbum_domain::Column::Id).equals(rbum_item::Entity, rbum_item::Column::RelRbumDomainId),
        )
        .and_where(Expr::tbl(rbum_kind::Entity, rbum_kind::Column::Code).eq(rbum_kind_code))
        .and_where(Expr::tbl(rbum_domain::Entity, rbum_domain::Column::Code).eq(rbum_domain_code))
        .order_by((rbum_item::Entity, rbum_item::Column::CreateTime), Order::Asc);

    let context: Option<TmpContext> = funs.db().get_dto(&query).await?;

    if let Some(context) = context {
        Ok(Some(TardisContext {
            own_paths: context.own_paths.to_string(),
            owner: context.id,
            ak: "_".to_string(),
            roles: vec![],
            groups: vec![],
        }))
    } else {
        Ok(None)
    }
}

pub async fn truncate_data<'a>(funs: &TardisFunsInst<'a>) -> TardisResult<()> {
    funs.db()
        .execute(
            Table::truncate()
                .table(rbum_cert::Entity)
                .table(rbum_cert_conf::Entity)
                .table(rbum_domain::Entity)
                .table(rbum_item::Entity)
                .table(rbum_item_attr::Entity)
                .table(rbum_kind::Entity)
                .table(rbum_kind_attr::Entity)
                .table(rbum_rel::Entity)
                .table(rbum_rel_attr::Entity)
                .table(rbum_rel_env::Entity)
                .table(rbum_set::Entity)
                .table(rbum_set_cate::Entity)
                .table(rbum_set_item::Entity),
        )
        .await?;
    Ok(())
}
