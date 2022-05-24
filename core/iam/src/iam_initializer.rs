use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::error::TardisError;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::log::info;
use tardis::web::web_server::TardisWebServer;
use tardis::TardisFuns;

use bios_basic::rbum::dto::rbum_domain_dto::RbumDomainAddReq;
use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::dto::rbum_kind_dto::RbumKindAddReq;
use bios_basic::rbum::rbum_initializer::get_first_account_context;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_domain_serv::RbumDomainServ;
use bios_basic::rbum::serv::rbum_item_serv::{RbumItemCrudOperation, RbumItemServ};
use bios_basic::rbum::serv::rbum_kind_serv::RbumKindServ;

use crate::basic::domain::{iam_account, iam_app, iam_res, iam_role, iam_tenant};
use crate::basic::dto::iam_account_dto::{IamAccountAggAddReq, IamAccountAggModifyReq};
use crate::basic::dto::iam_cert_conf_dto::{IamMailVCodeCertConfAddOrModifyReq, IamPhoneVCodeCertConfAddOrModifyReq, IamUserPwdCertConfAddOrModifyReq};
use crate::basic::dto::iam_role_dto::IamRoleAddReq;
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::console_app::api::iam_ca_app_api;
use crate::console_common::api::{iam_cc_account_api, iam_cc_role_api};
use crate::console_passport::api::{iam_cp_account_api, iam_cp_account_attr_api, iam_cp_cert_api, iam_cp_tenant_api};
use crate::console_system::api::{iam_cs_account_api, iam_cs_account_attr_api, iam_cs_cert_api, iam_cs_cert_conf_api, iam_cs_res_api, iam_cs_role_api, iam_cs_tenant_api};
use crate::console_tenant::api::{
    iam_ct_account_api, iam_ct_account_attr_api, iam_ct_app_api, iam_ct_cert_api, iam_ct_cert_conf_api, iam_ct_org_api, iam_ct_res_api, iam_ct_role_api, iam_ct_tenant_api,
};
use crate::iam_config::{BasicInfo, IamBasicInfoManager, IamConfig};
use crate::iam_constants;
use crate::iam_constants::{
    RBUM_ITEM_NAME_APP_ADMIN_ROLE, RBUM_ITEM_NAME_SYS_ADMIN_ACCOUNT, RBUM_ITEM_NAME_SYS_ADMIN_ROLE, RBUM_ITEM_NAME_TENANT_ADMIN_ROLE, RBUM_KIND_SCHEME_IAM_ACCOUNT,
    RBUM_KIND_SCHEME_IAM_APP, RBUM_KIND_SCHEME_IAM_RES, RBUM_KIND_SCHEME_IAM_ROLE, RBUM_KIND_SCHEME_IAM_TENANT, RBUM_SCOPE_LEVEL_GLOBAL,
};

pub async fn init(web_server: &TardisWebServer) -> TardisResult<()> {
    let funs = iam_constants::get_tardis_inst();
    init_db(funs).await?;
    init_api(web_server).await
}

async fn init_api(web_server: &TardisWebServer) -> TardisResult<()> {
    web_server
        .add_module(
            iam_constants::COMPONENT_CODE,
            (
                (iam_cc_account_api::IamCcAccountApi, iam_cc_role_api::IamCcRoleApi),
                (
                    iam_cp_account_api::IamCpAccountApi,
                    iam_cp_cert_api::IamCpCertApi,
                    iam_cp_tenant_api::IamCpTenantApi,
                    iam_cp_account_attr_api::IamCpAccountAttrApi,
                ),
                (
                    iam_cs_tenant_api::IamCsTenantApi,
                    iam_cs_account_api::IamCsAccountApi,
                    iam_cs_account_attr_api::IamCsAccountAttrApi,
                    iam_cs_cert_api::IamCsCertApi,
                    iam_cs_cert_conf_api::IamCsCertConfApi,
                    iam_cs_role_api::IamCsRoleApi,
                    iam_cs_res_api::IamCsResApi,
                ),
                (
                    iam_ct_tenant_api::IamCtTenantApi,
                    iam_ct_cert_conf_api::IamCtCertConfApi,
                    iam_ct_org_api::IamCtOrgApi,
                    iam_ct_account_api::IamCtAccountApi,
                    iam_ct_account_attr_api::IamCtAccountAttrApi,
                    iam_ct_app_api::IamCtAppApi,
                    iam_ct_cert_api::IamCtCertApi,
                    iam_ct_role_api::IamCtRoleApi,
                    iam_ct_res_api::IamCtResApi,
                ),
                (iam_ca_app_api::IamCaAppApi),
            ),
        )
        .await;
    Ok(())
}

pub async fn init_db(mut funs: TardisFunsInst<'_>) -> TardisResult<Option<(String, String)>> {
    bios_basic::rbum::rbum_initializer::init(funs.module_code(), funs.conf::<IamConfig>().rbum.clone()).await?;
    funs.begin().await?;
    let cxt = get_first_account_context(RBUM_KIND_SCHEME_IAM_ACCOUNT, iam_constants::COMPONENT_CODE, &funs).await?;
    let sysadmin_info = if let Some(cxt) = cxt {
        init_basic_info(&funs, &cxt).await?;
        None
    } else {
        funs.db().create_table_and_index(&iam_tenant::ActiveModel::create_table_and_index_statement(TardisFuns::reldb().backend())).await?;
        funs.db().create_table_and_index(&iam_app::ActiveModel::create_table_and_index_statement(TardisFuns::reldb().backend())).await?;
        funs.db().create_table_and_index(&iam_role::ActiveModel::create_table_and_index_statement(TardisFuns::reldb().backend())).await?;
        funs.db().create_table_and_index(&iam_account::ActiveModel::create_table_and_index_statement(TardisFuns::reldb().backend())).await?;
        funs.db().create_table_and_index(&iam_res::ActiveModel::create_table_and_index_statement(TardisFuns::reldb().backend())).await?;
        let (name, password) = init_rbum_data(&funs).await?;
        Some((name, password))
    };
    funs.commit().await?;
    Ok(sysadmin_info)
}

async fn init_basic_info<'a>(funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
    let kind_tenant_id = RbumKindServ::get_rbum_kind_id_by_code(RBUM_KIND_SCHEME_IAM_TENANT, funs)
        .await?
        .ok_or_else(|| TardisError::NotFound("Initialization error, tenant kind not found".to_string()))?;
    let kind_app_id = RbumKindServ::get_rbum_kind_id_by_code(RBUM_KIND_SCHEME_IAM_APP, funs)
        .await?
        .ok_or_else(|| TardisError::NotFound("Initialization error, app kind not found".to_string()))?;
    let kind_role_id = RbumKindServ::get_rbum_kind_id_by_code(RBUM_KIND_SCHEME_IAM_ROLE, funs)
        .await?
        .ok_or_else(|| TardisError::NotFound("Initialization error, role kind not found".to_string()))?;
    let kind_account_id = RbumKindServ::get_rbum_kind_id_by_code(RBUM_KIND_SCHEME_IAM_ACCOUNT, funs)
        .await?
        .ok_or_else(|| TardisError::NotFound("Initialization error, account kind not found".to_string()))?;
    let kind_res_id = RbumKindServ::get_rbum_kind_id_by_code(RBUM_KIND_SCHEME_IAM_RES, funs)
        .await?
        .ok_or_else(|| TardisError::NotFound("Initialization error, res kind not found".to_string()))?;

    let domain_iam_id = RbumDomainServ::get_rbum_domain_id_by_code(iam_constants::COMPONENT_CODE, funs)
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
        funs,
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

    IamBasicInfoManager::set(BasicInfo {
        kind_tenant_id,
        kind_app_id,
        kind_account_id,
        kind_role_id,
        kind_res_id,
        domain_iam_id,
        role_sys_admin_id,
        role_tenant_admin_id,
        role_app_admin_id,
    })?;
    Ok(())
}

async fn init_rbum_data(funs: &TardisFunsInst<'_>) -> TardisResult<(String, String)> {
    let default_account_id = TardisFuns::field.nanoid();

    let cxt = TardisContext {
        own_paths: "".to_string(),
        ak: "".to_string(),
        token: "".to_string(),
        token_kind: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: default_account_id.clone(),
    };

    let kind_tenant_id = add_kind(RBUM_KIND_SCHEME_IAM_TENANT, funs, &cxt).await?;
    let kind_app_id = add_kind(RBUM_KIND_SCHEME_IAM_APP, funs, &cxt).await?;
    let kind_role_id = add_kind(RBUM_KIND_SCHEME_IAM_ROLE, funs, &cxt).await?;
    let kind_account_id = add_kind(RBUM_KIND_SCHEME_IAM_ACCOUNT, funs, &cxt).await?;
    let kind_res_id = add_kind(RBUM_KIND_SCHEME_IAM_RES, funs, &cxt).await?;

    let domain_iam_id = add_domain(funs, &cxt).await?;

    IamBasicInfoManager::set(BasicInfo {
        kind_tenant_id: kind_tenant_id.to_string(),
        kind_app_id: kind_app_id.to_string(),
        kind_account_id: kind_account_id.to_string(),
        kind_role_id: kind_role_id.to_string(),
        kind_res_id: kind_res_id.to_string(),
        domain_iam_id: domain_iam_id.to_string(),
        role_sys_admin_id: "".to_string(),
        role_tenant_admin_id: "".to_string(),
        role_app_admin_id: "".to_string(),
    })?;

    IamSetServ::init_set(true, RBUM_SCOPE_LEVEL_GLOBAL, funs, &cxt).await?;
    IamSetServ::init_set(false, RBUM_SCOPE_LEVEL_GLOBAL, funs, &cxt).await?;
    IamCertServ::init_default_ident_conf(
        IamUserPwdCertConfAddOrModifyReq {
            ak_note: None,
            ak_rule: None,
            sk_note: None,
            sk_rule: None,
            repeatable: Some(true),
            expire_sec: None,
        },
        Some(IamPhoneVCodeCertConfAddOrModifyReq { ak_note: None, ak_rule: None }),
        Some(IamMailVCodeCertConfAddOrModifyReq { ak_note: None, ak_rule: None }),
        funs,
        &cxt,
    )
    .await?;

    let pwd = IamCertServ::get_new_pwd();
    IamAccountServ::add_account_agg(
        &IamAccountAggAddReq {
            id: Some(TrimString(default_account_id.clone())),
            name: TrimString(RBUM_ITEM_NAME_SYS_ADMIN_ACCOUNT.to_string()),
            cert_user_name: TrimString(RBUM_ITEM_NAME_SYS_ADMIN_ACCOUNT.to_string()),
            cert_password: TrimString(pwd.clone()),
            cert_phone: None,
            cert_mail: None,
            icon: None,
            disabled: None,
            scope_level: None,
            role_ids: None,
            exts: Default::default(),
        },
        funs,
        &cxt,
    )
    .await?;

    let role_sys_admin_id = IamRoleServ::add_item(
        &mut IamRoleAddReq {
            name: TrimString(RBUM_ITEM_NAME_SYS_ADMIN_ROLE.to_string()),
            icon: None,
            sort: None,
            scope_level: Some(iam_constants::RBUM_SCOPE_LEVEL_PRIVATE),
            disabled: None,
        },
        funs,
        &cxt,
    )
    .await?;

    IamAccountServ::modify_account_agg(
        &default_account_id,
        &IamAccountAggModifyReq {
            name: None,
            scope_level: None,
            disabled: None,
            icon: None,
            role_ids: Some(vec![role_sys_admin_id.clone()]),
            exts: Default::default(),
        },
        funs,
        &cxt,
    )
    .await?;

    let role_tenant_admin_id = IamRoleServ::add_item(
        &mut IamRoleAddReq {
            name: TrimString(RBUM_ITEM_NAME_TENANT_ADMIN_ROLE.to_string()),
            icon: None,
            sort: None,
            scope_level: Some(iam_constants::RBUM_SCOPE_LEVEL_TENANT),
            disabled: None,
        },
        funs,
        &cxt,
    )
    .await?;
    let role_app_admin_id = IamRoleServ::add_item(
        &mut IamRoleAddReq {
            name: TrimString(RBUM_ITEM_NAME_APP_ADMIN_ROLE.to_string()),
            icon: None,
            sort: None,
            scope_level: Some(iam_constants::RBUM_SCOPE_LEVEL_APP),
            disabled: None,
        },
        funs,
        &cxt,
    )
    .await?;

    IamBasicInfoManager::set(BasicInfo {
        kind_tenant_id,
        kind_app_id,
        kind_account_id,
        kind_role_id,
        kind_res_id,
        domain_iam_id,
        role_sys_admin_id: role_sys_admin_id.clone(),
        role_tenant_admin_id,
        role_app_admin_id,
    })?;

    info!(
        "Initialization is complete.
-----------
System administrator name: {} ,Initial password: {}
-----------",
        RBUM_ITEM_NAME_SYS_ADMIN_ACCOUNT, pwd
    );
    Ok((RBUM_ITEM_NAME_SYS_ADMIN_ACCOUNT.to_string(), pwd))
}

async fn add_kind<'a>(scheme: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<String> {
    RbumKindServ::add_rbum(
        &mut RbumKindAddReq {
            code: TrimString(scheme.to_string()),
            name: TrimString(scheme.to_string()),
            note: None,
            icon: None,
            sort: None,
            ext_table_name: Some(scheme.to_string().to_lowercase()),
            scope_level: Some(iam_constants::RBUM_SCOPE_LEVEL_GLOBAL),
        },
        funs,
        cxt,
    )
    .await
}

async fn add_domain<'a>(funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<String> {
    RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: TrimString(iam_constants::COMPONENT_CODE.to_string()),
            name: TrimString(iam_constants::COMPONENT_CODE.to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: Some(iam_constants::RBUM_SCOPE_LEVEL_GLOBAL),
        },
        funs,
        cxt,
    )
    .await
}
