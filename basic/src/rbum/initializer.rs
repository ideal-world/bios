use serde::{Deserialize, Serialize};
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm::*;
use tardis::db::sea_query::*;
use tardis::TardisFuns;

use crate::rbum::constants;
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

    // let tenant_id = TardisFuns::field.uuid_str();
    // let app_id = TardisFuns::field.uuid_str();
    // let sys_admin_id = TardisFuns::field.uuid_str();
    // let context = TardisContext {
    //     app_id: app_id.clone(),
    //     tenant_id: tenant_id.clone(),
    //     ak: "_".to_string(),
    //     account_id: sys_admin_id.clone(),
    //     token: "_".to_string(),
    //     token_kind: "".to_string(),
    //     roles: vec![],
    //     groups: vec![],
    // };
    //
    // rbum_kind_serv::kind::add_rbum_kind(
    //     &RbumKindAddReq {
    //         id: constants::RBUM_KIND_ID_IAM_TENANT.to_string(),
    //         rel_app_id: None,
    //         scope_kind: RbumScopeKind::Global,
    //         name: "Tenant".to_string(),
    //         note: "".to_string(),
    //         icon: "".to_string(),
    //         sort: 0,
    //         ext_table_name: constants::RBUM_KIND_ID_IAM_TENANT.to_string(),
    //     },
    //     &tx,
    //     &context,
    // )
    // .await?;
    //
    // rbum_kind_serv::kind::add_rbum_kind(
    //     &RbumKindAddReq {
    //         id: constants::RBUM_KIND_ID_IAM_APP.to_string(),
    //         rel_app_id: None,
    //         scope_kind: RbumScopeKind::Global,
    //         name: "App".to_string(),
    //         note: "".to_string(),
    //         icon: "".to_string(),
    //         sort: 0,
    //         ext_table_name: constants::RBUM_KIND_ID_IAM_APP.to_string(),
    //     },
    //     &tx,
    //     &context,
    // )
    // .await?;
    //
    // rbum_kind_serv::kind::add_rbum_kind(
    //     &RbumKindAddReq {
    //         id: constants::RBUM_KIND_ID_IAM_ACCOUNT.to_string(),
    //         rel_app_id: None,
    //         scope_kind: RbumScopeKind::Global,
    //         name: "Account".to_string(),
    //         note: "".to_string(),
    //         icon: "".to_string(),
    //         sort: 0,
    //         ext_table_name: constants::RBUM_KIND_ID_IAM_ACCOUNT.to_string(),
    //     },
    //     &tx,
    //     &context,
    // )
    // .await?;
    //
    // rbum_item_serv::item::add_rbum_item(
    //     tenant_id.as_str(),
    //     constants::RBUM_KIND_ID_IAM_TENANT,
    //     &RbumItemAddReq {
    //         scope_kind: RbumScopeKind::Tenant,
    //         disabled: false,
    //         name: "System Tenant".to_string(),
    //         uri_path: "".to_string(),
    //         icon: "".to_string(),
    //         sort: 0,
    //         rel_rbum_domain_id: crate::Components::Iam.to_string(),
    //     },
    //     None,
    //     &tx,
    //     &context,
    // )
    // .await?;
    //
    // rbum_item_serv::item::add_rbum_item(
    //     app_id.as_str(),
    //     constants::RBUM_KIND_ID_IAM_APP,
    //     &RbumItemAddReq {
    //         scope_kind: RbumScopeKind::App,
    //         disabled: false,
    //         name: "IAM".to_string(),
    //         uri_path: "".to_string(),
    //         icon: "".to_string(),
    //         sort: 0,
    //         rel_rbum_domain_id: crate::Components::Iam.to_string(),
    //     },
    //     None,
    //     &tx,
    //     &context,
    // )
    // .await?;
    //
    // rbum_item_serv::item::add_rbum_item(
    //     sys_admin_id.as_str(),
    //     constants::RBUM_KIND_ID_IAM_ACCOUNT,
    //     &RbumItemAddReq {
    //         scope_kind: RbumScopeKind::App,
    //         disabled: false,
    //         name: "SysAdmin".to_string(),
    //         uri_path: "".to_string(),
    //         icon: "".to_string(),
    //         sort: 0,
    //         rel_rbum_domain_id: crate::Components::Iam.to_string(),
    //     },
    //     None,
    //     &tx,
    //     &context,
    // )
    // .await?;

    tx.commit().await?;
    Ok(())
}

pub async fn get_sys_admin_context() -> TardisContext {
    let app_table = Alias::new("app");
    let tenant_table = Alias::new("tenant");

    let mut query = Query::select();
    query
        .expr_as(Expr::value("_"), Alias::new("ak"))
        .expr_as(Expr::value("_"), Alias::new("token"))
        .expr_as(Expr::value("_"), Alias::new("token_kind"))
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
        .and_where(Expr::tbl(rbum_item::Entity, rbum_item::Column::RelRbumKindId).eq(constants::RBUM_KIND_ID_IAM_ACCOUNT));
    // TODO remove unwrap
    let context: TmpContext = TardisFuns::reldb().conn().get_dto(&query).await.unwrap().unwrap();

    TardisContext {
        app_id: context.app_id.to_string(),
        tenant_id: context.tenant_id.to_string(),
        ak: "ak1".to_string(),
        account_id: context.account_id.to_string(),
        token: "token1".to_string(),
        token_kind: "default".to_string(),
        roles: vec![],
        groups: vec![],
    }
}

#[derive(Deserialize, FromQueryResult, Serialize, Clone, Debug)]
struct TmpContext {
    pub app_id: String,
    pub tenant_id: String,
    pub account_id: String,
}
