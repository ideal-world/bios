use serde::{Deserialize, Serialize};
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::{TardisActiveModel, TardisRelDBlConnection};
use tardis::db::sea_orm::*;
use tardis::db::sea_query::*;
use tardis::TardisFuns;

use crate::rbum::domain::{
    rbum_cert, rbum_cert_conf, rbum_domain, rbum_item, rbum_item_attr, rbum_kind, rbum_kind_attr, rbum_rel, rbum_rel_attr, rbum_rel_env, rbum_set, rbum_set_cate, rbum_set_item,
};

pub async fn init_db() -> TardisResult<()> {
    let db_kind = TardisFuns::reldb().backend();
    let mut tx = TardisFuns::reldb().conn();
    tx.begin().await?;
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

pub async fn get_first_account_context<'a>(rbum_kind_uri_scheme: &str, rbum_domain_uri_authority: &str, db: &TardisRelDBlConnection<'a>) -> TardisResult<Option<TardisContext>> {
    #[derive(Deserialize, FromQueryResult, Serialize, Clone, Debug)]
    struct TmpContext {
        pub id: String,
        pub scope_ids: String,
    }

    let mut query = Query::select();
    query
        .column((rbum_item::Entity, rbum_item::Column::Id))
        .column((rbum_item::Entity, rbum_item::Column::ScopeIds))
        .from(rbum_item::Entity)
        .inner_join(
            rbum_kind::Entity,
            Expr::tbl(rbum_kind::Entity, rbum_kind::Column::Id).equals(rbum_item::Entity, rbum_item::Column::RelRbumKindId),
        )
        .inner_join(
            rbum_domain::Entity,
            Expr::tbl(rbum_domain::Entity, rbum_domain::Column::Id).equals(rbum_item::Entity, rbum_item::Column::RelRbumDomainId),
        )
        .and_where(Expr::tbl(rbum_kind::Entity, rbum_kind::Column::UriScheme).eq(rbum_kind_uri_scheme))
        .and_where(Expr::tbl(rbum_domain::Entity, rbum_domain::Column::UriAuthority).eq(rbum_domain_uri_authority))
        .order_by((rbum_item::Entity, rbum_item::Column::CreateTime), Order::Asc);

    let context: Option<TmpContext> = db.get_dto(&query).await?;

    if let Some(context) = context {
        Ok(Some(TardisContext {
            scope_ids: context.scope_ids.to_string(),
            ak: "_".to_string(),
            account_id: context.id,
            token: "_".to_string(),
            token_kind: "_".to_string(),
            roles: vec![],
            groups: vec![],
        }))
    } else {
        Ok(None)
    }
}
