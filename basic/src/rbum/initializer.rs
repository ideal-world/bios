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
use crate::rbum::get_tenant_code_from_app_code;

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
        pub app_code: String,
        pub account_code: String,
    }

    let app_table = Alias::new("app");

    let mut query = Query::select();
    query
        .expr_as(Expr::tbl(rbum_item::Entity, rbum_item::Column::Code), Alias::new("account_code"))
        .expr_as(Expr::tbl(app_table.clone(), rbum_item::Column::Code), Alias::new("app_code"))
        .from(rbum_item::Entity)
        .join_as(
            JoinType::InnerJoin,
            rbum_item::Entity,
            app_table.clone(),
            Expr::tbl(app_table, rbum_item::Column::Code).equals(rbum_item::Entity, rbum_item::Column::RelAppCode),
        )
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
            app_code: context.app_code.to_string(),
            tenant_code: get_tenant_code_from_app_code(&context.app_code),
            ak: "_".to_string(),
            account_code: context.account_code,
            token: "_".to_string(),
            token_kind: "_".to_string(),
            roles: vec![],
            groups: vec![],
        }))
    } else {
        Ok(None)
    }
}
