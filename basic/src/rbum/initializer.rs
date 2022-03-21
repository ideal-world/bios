use serde::{Deserialize, Serialize};
use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm::*;
use tardis::db::sea_query::*;
use tardis::TardisFuns;

use crate::rbum::constants::{RBUM_ITEM_APP_CODE_LEN, RBUM_ITEM_TENANT_CODE_LEN};
use crate::rbum::domain::{
    rbum_cert, rbum_cert_conf, rbum_domain, rbum_item, rbum_item_attr, rbum_kind, rbum_kind_attr, rbum_rel, rbum_rel_attr, rbum_rel_env, rbum_set, rbum_set_cate, rbum_set_item,
};
use crate::rbum::dto::rbum_domain_dto::RbumDomainAddReq;
use crate::rbum::dto::rbum_item_dto::RbumItemAddReq;
use crate::rbum::dto::rbum_kind_dto::RbumKindAddReq;
use crate::rbum::enumeration::RbumScopeKind;
use crate::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use crate::rbum::serv::rbum_domain_serv::RbumDomainServ;
use crate::rbum::serv::rbum_item_serv::RbumItemServ;
use crate::rbum::serv::rbum_kind_serv::RbumKindServ;
use crate::rbum::{constants, get_tenant_code_from_app_code};

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

    let default_tenant_code = TardisFuns::field.nanoid_len(RBUM_ITEM_TENANT_CODE_LEN);
    let default_app_code = format!("{}{}", default_tenant_code, TardisFuns::field.nanoid_len(RBUM_ITEM_APP_CODE_LEN));
    let default_account_code = format!("{}{}", default_tenant_code, TardisFuns::field.nanoid());

    let cxt = TardisContext {
        app_code: default_app_code.clone(),
        tenant_code: default_tenant_code.clone(),
        ak: "".to_string(),
        account_code: default_account_code.clone(),
        token: "".to_string(),
        token_kind: "".to_string(),
        roles: vec![],
        groups: vec![],
    };

    let kind_tenant_id = RbumKindServ::add_rbum(
        &mut RbumKindAddReq {
            uri_scheme: TrimString(constants::RBUM_KIND_SCHEME_IAM_TENANT.to_string()),
            name: TrimString(constants::RBUM_KIND_SCHEME_IAM_TENANT.to_string()),
            note: None,
            icon: None,
            sort: None,
            ext_table_name: Some(constants::RBUM_KIND_SCHEME_IAM_TENANT.to_string().to_lowercase()),
            scope_kind: Some(RbumScopeKind::Global),
        },
        &tx,
        &cxt,
    )
    .await?;

    let kind_app_id = RbumKindServ::add_rbum(
        &mut RbumKindAddReq {
            uri_scheme: TrimString(constants::RBUM_KIND_SCHEME_IAM_APP.to_string()),
            name: TrimString(constants::RBUM_KIND_SCHEME_IAM_APP.to_string()),
            note: None,
            icon: None,
            sort: None,
            ext_table_name: Some(constants::RBUM_KIND_SCHEME_IAM_APP.to_string().to_lowercase()),
            scope_kind: Some(RbumScopeKind::Global),
        },
        &tx,
        &cxt,
    )
    .await?;

    let kind_account_id = RbumKindServ::add_rbum(
        &mut RbumKindAddReq {
            uri_scheme: TrimString(constants::RBUM_KIND_SCHEME_IAM_ACCOUNT.to_string()),
            name: TrimString(constants::RBUM_KIND_SCHEME_IAM_ACCOUNT.to_string()),
            note: None,
            icon: None,
            sort: None,
            ext_table_name: Some(constants::RBUM_KIND_SCHEME_IAM_ACCOUNT.to_string().to_lowercase()),
            scope_kind: Some(RbumScopeKind::Global),
        },
        &tx,
        &cxt,
    )
    .await?;

    let domain_iam_id = RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            uri_authority: TrimString(crate::Components::Iam.to_string()),
            name: TrimString(crate::Components::Iam.to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_kind: Some(RbumScopeKind::Global),
        },
        &tx,
        &cxt,
    )
    .await?;

    RbumItemServ::add_rbum(
        &mut RbumItemAddReq {
            code: Some(TrimString(default_tenant_code.clone())),
            uri_path: None,
            name: TrimString(constants::RBUM_ITEM_NAME_DEFAULT_TENANT.to_string()),
            icon: None,
            sort: None,
            scope_kind: None,
            disabled: None,
            rel_rbum_kind_id: kind_tenant_id.clone(),
            rel_rbum_domain_id: domain_iam_id.clone(),
        },
        &tx,
        &cxt,
    )
    .await?;

    RbumItemServ::add_rbum(
        &mut RbumItemAddReq {
            code: Some(TrimString(default_app_code.clone())),
            uri_path: None,
            name: TrimString(constants::RBUM_ITEM_NAME_DEFAULT_APP.to_string()),
            icon: None,
            sort: None,
            scope_kind: None,
            disabled: None,
            rel_rbum_kind_id: kind_app_id.clone(),
            rel_rbum_domain_id: domain_iam_id.clone(),
        },
        &tx,
        &cxt,
    )
    .await?;

    RbumItemServ::add_rbum(
        &mut RbumItemAddReq {
            code: Some(TrimString(default_account_code.clone())),
            uri_path: None,
            name: TrimString(constants::RBUM_ITEM_NAME_DEFAULT_ACCOUNT.to_string()),
            icon: None,
            sort: None,
            scope_kind: None,
            disabled: None,
            rel_rbum_kind_id: kind_account_id.clone(),
            rel_rbum_domain_id: domain_iam_id.clone(),
        },
        &tx,
        &cxt,
    )
    .await?;

    tx.commit().await?;
    Ok(())
}

pub async fn get_sys_admin_context() -> TardisResult<TardisContext> {
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
        .and_where(Expr::tbl(rbum_kind::Entity, rbum_kind::Column::UriScheme).eq(constants::RBUM_KIND_SCHEME_IAM_ACCOUNT))
        .order_by((rbum_item::Entity, rbum_item::Column::CreateTime), Order::Asc);

    let context: TmpContext =
        TardisFuns::reldb().conn().get_dto(&query).await?.ok_or_else(|| TardisError::NotFound("Initialization error, system object not found".to_string()))?;

    Ok(TardisContext {
        app_code: context.app_code.to_string(),
        tenant_code: get_tenant_code_from_app_code(&context.app_code),
        ak: "_".to_string(),
        account_code: context.account_code,
        token: "_".to_string(),
        token_kind: "_".to_string(),
        roles: vec![],
        groups: vec![],
    })
}

#[derive(Deserialize, FromQueryResult, Serialize, Clone, Debug)]
struct TmpContext {
    pub app_code: String,
    pub account_code: String,
}
