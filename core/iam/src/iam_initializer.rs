use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::{TardisActiveModel, TardisRelDBlConnection};
use tardis::log::info;
use tardis::web::web_server::TardisWebServer;
use tardis::TardisFuns;

use bios_basic::rbum::dto::filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::dto::rbum_domain_dto::RbumDomainAddReq;
use bios_basic::rbum::dto::rbum_kind_dto::RbumKindAddReq;
use bios_basic::rbum::rbum_initializer::get_first_account_context;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_domain_serv::RbumDomainServ;
use bios_basic::rbum::serv::rbum_item_serv::{RbumItemCrudOperation, RbumItemServ};
use bios_basic::rbum::serv::rbum_kind_serv::RbumKindServ;
use bios_basic::rbum::serv::rbum_rel_serv::RbumRelServ;

use crate::basic::constants;
use crate::basic::constants::*;
use crate::basic::domain::{iam_account, iam_app, iam_http_res, iam_role, iam_tenant};
use crate::basic::dto::iam_account_dto::IamAccountAddReq;
use crate::basic::dto::iam_cert_dto::IamUserPwdCertAddReq;
use crate::basic::dto::iam_role_dto::IamRoleAddReq;
use crate::basic::enumeration::IAMRelKind;
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::console_passport::api::{iam_cp_account_api, iam_cp_cert_api};
use crate::console_system::api::{iam_cs_account_api, iam_cs_tenant_api};
use crate::console_tenant::api::{iam_ct_account_api, iam_ct_app_api, iam_ct_cert_conf_api, iam_ct_http_res_api, iam_ct_role_api, iam_ct_tenant_api};

pub async fn init_api(web_server: &mut TardisWebServer) -> TardisResult<()> {
    web_server.add_module(
        "iam",
        (
            iam_cp_account_api::IamCpAccountApi,
            iam_cp_cert_api::IamCpAccountApi,
            iam_cs_tenant_api::IamCsTenantApi,
            iam_cs_account_api::IamCsAccountApi,
            iam_ct_tenant_api::IamCtTenantApi,
            iam_ct_account_api::IamCtAccountApi,
            iam_ct_app_api::IamCtAppApi,
            iam_ct_role_api::IamCtRoleApi,
            iam_ct_cert_conf_api::IamCtCertConfApi,
            iam_ct_http_res_api::IamCtHttpResApi,
        ),
    );
    Ok(())
}

pub async fn init_db() -> TardisResult<()> {
    bios_basic::rbum::rbum_initializer::init_db().await?;
    let mut tx = TardisFuns::reldb().conn();
    tx.begin().await?;
    let cxt = get_first_account_context(RBUM_KIND_SCHEME_IAM_ACCOUNT, &bios_basic::Components::Iam.to_string(), &tx).await?;
    if let Some(cxt) = cxt {
        init_basic_info(&tx, &cxt).await?;
    } else {
        tx.create_table_and_index(&iam_tenant::ActiveModel::create_table_and_index_statement(TardisFuns::reldb().backend())).await?;
        tx.create_table_and_index(&iam_app::ActiveModel::create_table_and_index_statement(TardisFuns::reldb().backend())).await?;
        tx.create_table_and_index(&iam_role::ActiveModel::create_table_and_index_statement(TardisFuns::reldb().backend())).await?;
        tx.create_table_and_index(&iam_account::ActiveModel::create_table_and_index_statement(TardisFuns::reldb().backend())).await?;
        tx.create_table_and_index(&iam_http_res::ActiveModel::create_table_and_index_statement(TardisFuns::reldb().backend())).await?;
        init_rbum_data(&tx).await?;
    }
    tx.commit().await?;
    Ok(())
}

async fn init_basic_info<'a>(tx: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<()> {
    let kind_tenant_id = RbumKindServ::get_rbum_kind_id_by_code(RBUM_KIND_SCHEME_IAM_TENANT, tx)
        .await?
        .ok_or_else(|| TardisError::NotFound("Initialization error, tenant kind not found".to_string()))?;
    let kind_app_id =
        RbumKindServ::get_rbum_kind_id_by_code(RBUM_KIND_SCHEME_IAM_APP, tx).await?.ok_or_else(|| TardisError::NotFound("Initialization error, app kind not found".to_string()))?;
    let kind_role_id = RbumKindServ::get_rbum_kind_id_by_code(RBUM_KIND_SCHEME_IAM_ROLE, tx)
        .await?
        .ok_or_else(|| TardisError::NotFound("Initialization error, role kind not found".to_string()))?;
    let kind_account_id = RbumKindServ::get_rbum_kind_id_by_code(RBUM_KIND_SCHEME_IAM_ACCOUNT, tx)
        .await?
        .ok_or_else(|| TardisError::NotFound("Initialization error, account kind not found".to_string()))?;
    let kind_http_res_id = RbumKindServ::get_rbum_kind_id_by_code(RBUM_KIND_SCHEME_IAM_RES_HTTP, tx)
        .await?
        .ok_or_else(|| TardisError::NotFound("Initialization error, http res kind not found".to_string()))?;

    let domain_iam_id = RbumDomainServ::get_rbum_domain_id_by_code(&bios_basic::Components::Iam.to_string(), tx)
        .await?
        .ok_or_else(|| TardisError::NotFound("Initialization error, iam domain not found".to_string()))?;

    let roles = RbumItemServ::paginate_rbums(
        &RbumBasicFilterReq {
            rbum_kind_id: Some(kind_role_id.clone()),
            rbum_domain_id: Some(domain_iam_id.clone()),
            ..Default::default()
        },
        1,
        3,
        Some(false),
        None,
        tx,
        cxt,
    )
    .await?
    .records;

    let role_sys_admin_id = roles
        .iter()
        .find(|r| r.name == RBUM_ITEM_NAME_SYS_ADMIN_ROLE)
        .map(|r| r.id.clone())
        .ok_or_else(|| TardisError::NotFound("Initialization error, sys admin role not found".to_string()))?;

    let role_tenant_admin_id = roles
        .iter()
        .find(|r| r.name == RBUM_ITEM_NAME_TENANT_ADMIN_ROLE)
        .map(|r| r.id.clone())
        .ok_or_else(|| TardisError::NotFound("Initialization error, tenant admin role not found".to_string()))?;

    let role_app_admin_id = roles
        .iter()
        .find(|r| r.name == RBUM_ITEM_NAME_APP_ADMIN_ROLE)
        .map(|r| r.id.clone())
        .ok_or_else(|| TardisError::NotFound("Initialization error, app admin role not found".to_string()))?;

    set_basic_info(BasicInfoPub {
        kind_tenant_id,
        kind_app_id,
        kind_account_id,
        kind_role_id,
        kind_http_res_id,
        domain_iam_id,
        role_sys_admin_id,
        role_tenant_admin_id,
        role_app_admin_id,
    })?;
    Ok(())
}

async fn init_rbum_data<'a>(tx: &TardisRelDBlConnection<'a>) -> TardisResult<()> {
    let default_account_id = TardisFuns::field.nanoid();

    let cxt = TardisContext {
        own_paths: "".to_string(),
        ak: "".to_string(),
        token: "".to_string(),
        token_kind: "".to_string(),
        roles: vec![],
        groups: vec![],
        account_id: default_account_id.to_string(),
    };

    let kind_tenant_id = add_kind(RBUM_KIND_SCHEME_IAM_TENANT, tx, &cxt).await?;
    let kind_app_id = add_kind(RBUM_KIND_SCHEME_IAM_APP, tx, &cxt).await?;
    let kind_role_id = add_kind(RBUM_KIND_SCHEME_IAM_ROLE, tx, &cxt).await?;
    let kind_account_id = add_kind(RBUM_KIND_SCHEME_IAM_ACCOUNT, tx, &cxt).await?;
    let kind_http_res_id = add_kind(RBUM_KIND_SCHEME_IAM_RES_HTTP, tx, &cxt).await?;

    let domain_iam_id = add_domain(tx, &cxt).await?;

    set_basic_info(BasicInfoPub {
        kind_tenant_id: kind_tenant_id.to_string(),
        kind_app_id: kind_app_id.to_string(),
        kind_account_id: kind_account_id.to_string(),
        kind_role_id: kind_role_id.to_string(),
        kind_http_res_id: kind_http_res_id.to_string(),
        domain_iam_id: domain_iam_id.to_string(),
        role_sys_admin_id: "".to_string(),
        role_tenant_admin_id: "".to_string(),
        role_app_admin_id: "".to_string(),
    })?;

    let role_sys_admin_id = IamRoleServ::add_item(
        &mut IamRoleAddReq {
            name: TrimString(RBUM_ITEM_NAME_SYS_ADMIN_ROLE.to_string()),
            icon: None,
            sort: None,
            scope_level: constants::RBUM_SCOPE_LEVEL_GLOBAL,
            disabled: None,
        },
        tx,
        &cxt,
    )
    .await?;
    let role_tenant_admin_id = IamRoleServ::add_item(
        &mut IamRoleAddReq {
            name: TrimString(RBUM_ITEM_NAME_TENANT_ADMIN_ROLE.to_string()),
            icon: None,
            sort: None,
            scope_level: constants::RBUM_SCOPE_LEVEL_GLOBAL,
            disabled: None,
        },
        tx,
        &cxt,
    )
    .await?;
    let role_app_admin_id = IamRoleServ::add_item(
        &mut IamRoleAddReq {
            name: TrimString(RBUM_ITEM_NAME_APP_ADMIN_ROLE.to_string()),
            icon: None,
            sort: None,
            scope_level: constants::RBUM_SCOPE_LEVEL_GLOBAL,
            disabled: None,
        },
        tx,
        &cxt,
    )
    .await?;

    set_basic_info(BasicInfoPub {
        kind_tenant_id,
        kind_app_id,
        kind_account_id,
        kind_role_id,
        kind_http_res_id,
        domain_iam_id,
        role_sys_admin_id: role_sys_admin_id.clone(),
        role_tenant_admin_id,
        role_app_admin_id,
    })?;

    IamCertServ::init_global_ident_conf(tx, &cxt).await?;

    let account_sys_admin_id = IamAccountServ::add_item(
        &mut IamAccountAddReq {
            id: Some(TrimString(default_account_id)),
            name: TrimString(RBUM_ITEM_NAME_SYS_ADMIN_ACCOUNT.to_string()),
            icon: None,
            disabled: None,
            scope_level: constants::RBUM_SCOPE_LEVEL_GLOBAL,
        },
        tx,
        &cxt,
    )
    .await?;

    RbumRelServ::add_simple_rel(&IAMRelKind::IamRoleAccount.to_string(), &role_sys_admin_id, &account_sys_admin_id, tx, &cxt).await?;
    let pwd = IamCertServ::get_new_pwd();
    IamCertUserPwdServ::add_cert(
        &mut IamUserPwdCertAddReq {
            ak: TrimString(RBUM_ITEM_NAME_SYS_ADMIN_ACCOUNT.to_string()),
            sk: TrimString(pwd.clone()),
        },
        &account_sys_admin_id,
        None,
        tx,
        &cxt,
    )
    .await?;

    info!(
        "Initialization is complete.
-----------
System administrator name: {} ,Initial password: {}
-----------",
        RBUM_ITEM_NAME_SYS_ADMIN_ACCOUNT, pwd
    );
    Ok(())
}

async fn add_kind<'a>(scheme: &str, tx: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<String> {
    RbumKindServ::add_rbum(
        &mut RbumKindAddReq {
            code: TrimString(scheme.to_string()),
            name: TrimString(scheme.to_string()),
            note: None,
            icon: None,
            sort: None,
            ext_table_name: Some(scheme.to_string().to_lowercase()),
            scope_level: constants::RBUM_SCOPE_LEVEL_GLOBAL,
        },
        tx,
        cxt,
    )
    .await
}

async fn add_domain<'a>(tx: &TardisRelDBlConnection<'a>, cxt: &TardisContext) -> TardisResult<String> {
    RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: TrimString(bios_basic::Components::Iam.to_string()),
            name: TrimString(bios_basic::Components::Iam.to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: constants::RBUM_SCOPE_LEVEL_GLOBAL,
        },
        tx,
        cxt,
    )
    .await
}
