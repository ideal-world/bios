use serde::{Deserialize, Serialize};
use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm::*;
use tardis::db::sea_query::*;
use tardis::TardisFuns;

use crate::rbum::constants;
use crate::rbum::domain::{
    rbum_cert, rbum_cert_conf, rbum_domain, rbum_item, rbum_item_attr, rbum_kind, rbum_kind_attr, rbum_rel, rbum_rel_attr, rbum_rel_env, rbum_set, rbum_set_cate, rbum_set_item,
};
use crate::rbum::enumeration::RbumScopeKind;

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

    let kind_tenant_id = TardisFuns::field.uuid_str();
    let kind_app_id = TardisFuns::field.uuid_str();
    let kind_account_id = TardisFuns::field.uuid_str();
    let domain_iam_id = TardisFuns::field.uuid_str();
    let item_default_tenant_id = TardisFuns::field.uuid_str();
    let item_iam_app_id = TardisFuns::field.uuid_str();
    let item_sys_admin_id = TardisFuns::field.uuid_str();

    rbum_kind::ActiveModel {
        id: Set(kind_tenant_id.clone()),
        uri_scheme: Set(constants::RBUM_KIND_SCHEME_IAM_TENANT.to_string()),
        name: Set(constants::RBUM_KIND_SCHEME_IAM_TENANT.to_string()),
        note: Set("".to_string()),
        icon: Set("".to_string()),
        sort: Set(0),
        ext_table_name: Set(constants::RBUM_KIND_SCHEME_IAM_TENANT.to_string().to_lowercase()),
        rel_app_id: Set(item_iam_app_id.clone()),
        rel_tenant_id: Set(item_default_tenant_id.clone()),
        updater_id: Set(item_sys_admin_id.clone()),
        scope_kind: Set(RbumScopeKind::Global.to_string()),
        ..Default::default()
    }
    .insert(tx.raw_tx()?)
    .await?;

    rbum_kind::ActiveModel {
        id: Set(kind_app_id.clone()),
        uri_scheme: Set(constants::RBUM_KIND_SCHEME_IAM_APP.to_string()),
        name: Set(constants::RBUM_KIND_SCHEME_IAM_APP.to_string()),
        note: Set("".to_string()),
        icon: Set("".to_string()),
        sort: Set(0),
        ext_table_name: Set(constants::RBUM_KIND_SCHEME_IAM_APP.to_string().to_lowercase()),
        rel_app_id: Set(item_iam_app_id.clone()),
        rel_tenant_id: Set(item_default_tenant_id.clone()),
        updater_id: Set(item_sys_admin_id.clone()),
        scope_kind: Set(RbumScopeKind::Global.to_string()),
        ..Default::default()
    }
    .insert(tx.raw_tx()?)
    .await?;

    rbum_kind::ActiveModel {
        id: Set(kind_account_id.clone()),
        uri_scheme: Set(constants::RBUM_KIND_SCHEME_IAM_ACCOUNT.to_string()),
        name: Set(constants::RBUM_KIND_SCHEME_IAM_ACCOUNT.to_string()),
        note: Set("".to_string()),
        icon: Set("".to_string()),
        sort: Set(0),
        ext_table_name: Set(constants::RBUM_KIND_SCHEME_IAM_ACCOUNT.to_string().to_lowercase()),
        rel_app_id: Set(item_iam_app_id.clone()),
        rel_tenant_id: Set(item_default_tenant_id.clone()),
        updater_id: Set(item_sys_admin_id.clone()),
        scope_kind: Set(RbumScopeKind::Global.to_string()),
        ..Default::default()
    }
    .insert(tx.raw_tx()?)
    .await?;

    rbum_domain::ActiveModel {
        id: Set(domain_iam_id.clone()),
        uri_authority: Set(crate::Components::Iam.to_string()),
        name: Set(crate::Components::Iam.to_string()),
        note: Set("".to_string()),
        icon: Set("".to_string()),
        sort: Set(0),
        rel_app_id: Set(item_iam_app_id.clone()),
        rel_tenant_id: Set(item_default_tenant_id.clone()),
        updater_id: Set(item_sys_admin_id.clone()),
        scope_kind: Set(RbumScopeKind::Global.to_string()),
        ..Default::default()
    }
    .insert(tx.raw_tx()?)
    .await?;

    rbum_item::ActiveModel {
        id: Set(item_default_tenant_id.clone()),
        code: Set(constants::RBUM_ITEM_CODE_DEFAULT_TENANT.to_string()),
        uri_path: Set(constants::RBUM_ITEM_CODE_DEFAULT_TENANT.to_string()),
        name: Set(constants::RBUM_ITEM_CODE_DEFAULT_TENANT.to_string()),
        icon: Set("".to_string()),
        sort: Set(0),
        rel_rbum_kind_id: Set(kind_tenant_id.clone()),
        rel_rbum_domain_id: Set(domain_iam_id.clone()),
        disabled: Set(false),
        rel_app_id: Set(item_iam_app_id.clone()),
        rel_tenant_id: Set(item_default_tenant_id.clone()),
        updater_id: Set(item_sys_admin_id.clone()),
        scope_kind: Set(RbumScopeKind::App.to_string()),
        ..Default::default()
    }
    .insert(tx.raw_tx()?)
    .await?;

    rbum_item::ActiveModel {
        id: Set(item_iam_app_id.clone()),
        code: Set(constants::RBUM_ITEM_CODE_DEFAULT_APP.to_string()),
        uri_path: Set(constants::RBUM_ITEM_CODE_DEFAULT_APP.to_string()),
        name: Set(constants::RBUM_ITEM_CODE_DEFAULT_APP.to_string()),
        icon: Set("".to_string()),
        sort: Set(0),
        rel_rbum_kind_id: Set(kind_app_id.clone()),
        rel_rbum_domain_id: Set(domain_iam_id.clone()),
        disabled: Set(false),
        rel_app_id: Set(item_iam_app_id.clone()),
        rel_tenant_id: Set(item_default_tenant_id.clone()),
        updater_id: Set(item_sys_admin_id.clone()),
        scope_kind: Set(RbumScopeKind::App.to_string()),
        ..Default::default()
    }
    .insert(tx.raw_tx()?)
    .await?;

    rbum_item::ActiveModel {
        id: Set(item_sys_admin_id.clone()),
        code: Set(constants::RBUM_ITEM_CODE_DEFAULT_ACCOUNT.to_string()),
        uri_path: Set(constants::RBUM_ITEM_CODE_DEFAULT_ACCOUNT.to_string()),
        name: Set(constants::RBUM_ITEM_CODE_DEFAULT_ACCOUNT.to_string()),
        icon: Set("".to_string()),
        sort: Set(0),
        rel_rbum_kind_id: Set(kind_account_id.clone()),
        rel_rbum_domain_id: Set(domain_iam_id.clone()),
        disabled: Set(false),
        rel_app_id: Set(item_iam_app_id.clone()),
        rel_tenant_id: Set(item_default_tenant_id.clone()),
        updater_id: Set(item_sys_admin_id.clone()),
        scope_kind: Set(RbumScopeKind::App.to_string()),
        ..Default::default()
    }
    .insert(tx.raw_tx()?)
    .await?;

    tx.commit().await?;
    Ok(())
}

pub async fn get_sys_admin_context() -> TardisResult<TardisContext> {
    let app_table = Alias::new("app");
    let tenant_table = Alias::new("tenant");

    let mut query = Query::select();
    query
        .expr_as(Expr::tbl(rbum_item::Entity, rbum_item::Column::Id), Alias::new("account_id"))
        .expr_as(Expr::tbl(app_table.clone(), rbum_item::Column::Id), Alias::new("app_id"))
        .expr_as(Expr::tbl(tenant_table.clone(), rbum_item::Column::Id), Alias::new("tenant_id"))
        .from(rbum_item::Entity)
        .join_as(
            JoinType::InnerJoin,
            rbum_item::Entity,
            app_table.clone(),
            Expr::tbl(app_table, rbum_item::Column::Id).equals(rbum_item::Entity, rbum_item::Column::RelAppId),
        )
        .join_as(
            JoinType::InnerJoin,
            rbum_item::Entity,
            tenant_table.clone(),
            Expr::tbl(tenant_table, rbum_item::Column::Id).equals(rbum_item::Entity, rbum_item::Column::RelTenantId),
        )
        .inner_join(
            rbum_kind::Entity,
            Expr::tbl(rbum_kind::Entity, rbum_kind::Column::Id).equals(rbum_item::Entity, rbum_item::Column::RelRbumKindId),
        )
        .and_where(Expr::tbl(rbum_kind::Entity, rbum_kind::Column::UriScheme).eq(constants::RBUM_KIND_SCHEME_IAM_ACCOUNT))
        .order_by((rbum_item::Entity, rbum_item::Column::CreateTime), Order::Asc);

    let context: TmpContext =
        TardisFuns::reldb().conn().get_dto(&query).await?.ok_or_else(|| TardisError::NotFound("Initialization error, system object not found".to_string()))?;

    Ok(TardisContext {
        app_id: context.app_id.to_string(),
        tenant_id: context.tenant_id.to_string(),
        ak: "_".to_string(),
        account_id: context.account_id,
        token: "_".to_string(),
        token_kind: "_".to_string(),
        roles: vec![],
        groups: vec![],
    })
}

#[derive(Deserialize, FromQueryResult, Serialize, Clone, Debug)]
struct TmpContext {
    pub app_id: String,
    pub tenant_id: String,
    pub account_id: String,
}
