use bios_basic::rbum::rbum_enumeration::{RbumCertStatusKind, RbumScopeLevelKind};
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm::sea_query::Table;
use tardis::log::info;
use tardis::web::web_server::{TardisWebServer, WebServerModule};
use tardis::{TardisFuns, TardisFunsInst};

use bios_basic::rbum::dto::rbum_domain_dto::RbumDomainAddReq;
use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::dto::rbum_kind_dto::RbumKindAddReq;
use bios_basic::rbum::rbum_initializer;
use bios_basic::rbum::rbum_initializer::get_first_account_context;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_domain_serv::RbumDomainServ;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemServ;
use bios_basic::rbum::serv::rbum_kind_serv::RbumKindServ;

use crate::basic::domain::{iam_account, iam_app, iam_config, iam_res, iam_role, iam_tenant};
use crate::basic::dto::iam_account_dto::{IamAccountAggAddReq, IamAccountAggModifyReq};
use crate::basic::dto::iam_cert_conf_dto::{IamCertConfMailVCodeAddOrModifyReq, IamCertConfPhoneVCodeAddOrModifyReq, IamCertConfUserPwdAddOrModifyReq};
use crate::basic::dto::iam_res_dto::{IamResAddReq, IamResAggAddReq, JsonMenu};
use crate::basic::dto::iam_role_dto::{IamRoleAddReq, IamRoleAggAddReq};
use crate::basic::dto::iam_set_dto::IamSetItemAggAddReq;
use crate::basic::middleware::encrypt_mw::EncryptMW;
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_res_serv::{IamMenuServ, IamResServ};
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::console_app::api::{iam_ca_account_api, iam_ca_app_api, iam_ca_cert_manage_api, iam_ca_res_api, iam_ca_role_api};
use crate::console_common::api::{iam_cc_account_api, iam_cc_app_api, iam_cc_org_api, iam_cc_res_api, iam_cc_role_api, iam_cc_system_api, iam_cc_tenant_api};
use crate::console_interface::api::{iam_ci_account_api, iam_ci_app_api, iam_ci_cert_api, iam_ci_res_api, iam_ci_role_api};
use crate::console_passport::api::{iam_cp_account_api, iam_cp_app_api, iam_cp_cert_api, iam_cp_tenant_api};
use crate::console_system::api::{
    iam_cs_account_api, iam_cs_account_attr_api, iam_cs_cert_api, iam_cs_org_api, iam_cs_platform_api, iam_cs_res_api, iam_cs_role_api, iam_cs_spi_data_api, iam_cs_tenant_api,
};
use crate::console_tenant::api::{
    iam_ct_account_api, iam_ct_account_attr_api, iam_ct_app_api, iam_ct_app_set_api, iam_ct_cert_api, iam_ct_cert_manage_api, iam_ct_org_api, iam_ct_res_api, iam_ct_role_api,
    iam_ct_tenant_api,
};
use crate::iam_config::{BasicInfo, IamBasicInfoManager, IamConfig};
use crate::iam_constants;
use crate::iam_constants::RBUM_SCOPE_LEVEL_GLOBAL;
use crate::iam_enumeration::{IamResKind, IamRoleKind, IamSetKind};

pub async fn init(web_server: &TardisWebServer) -> TardisResult<()> {
    let funs = iam_constants::get_tardis_inst();
    init_db(funs).await?;
    init_api(web_server).await
}

async fn init_api(web_server: &TardisWebServer) -> TardisResult<()> {
    web_server
        .add_module(
            iam_constants::COMPONENT_CODE,
            WebServerModule::from((
                (
                    iam_cc_account_api::IamCcAccountApi,
                    iam_cc_app_api::IamCcAppApi,
                    #[cfg(feature = "ldap_client")]
                    iam_cc_account_api::IamCcAccountLdapApi,
                    iam_cc_role_api::IamCcRoleApi,
                    iam_cc_org_api::IamCcOrgApi,
                    iam_cc_res_api::IamCcResApi,
                    iam_cc_system_api::IamCcSystemApi,
                    iam_cc_tenant_api::IamCcTenantApi,
                ),
                (
                    iam_cp_account_api::IamCpAccountApi,
                    iam_cp_app_api::IamCpAppApi,
                    iam_cp_cert_api::IamCpCertApi,
                    #[cfg(feature = "ldap_client")]
                    iam_cp_cert_api::IamCpCertLdapApi,
                    iam_cp_tenant_api::IamCpTenantApi,
                ),
                (
                    iam_cs_tenant_api::IamCsTenantApi,
                    iam_cs_account_api::IamCsAccountApi,
                    iam_cs_account_attr_api::IamCsAccountAttrApi,
                    iam_cs_cert_api::IamCsCertApi,
                    iam_cs_cert_api::IamCsCertConfigLdapApi,
                    iam_cs_platform_api::IamCsPlatformApi,
                    iam_cs_org_api::IamCsOrgApi,
                    iam_cs_org_api::IamCsOrgItemApi,
                    iam_cs_role_api::IamCsRoleApi,
                    iam_cs_res_api::IamCsResApi,
                    iam_cs_spi_data_api::IamCsSpiDataApi,
                ),
                (
                    iam_ct_tenant_api::IamCtTenantApi,
                    iam_ct_org_api::IamCtOrgApi,
                    iam_ct_account_api::IamCtAccountApi,
                    iam_ct_account_attr_api::IamCtAccountAttrApi,
                    iam_ct_app_api::IamCtAppApi,
                    iam_ct_app_set_api::IamCtAppSetApi,
                    iam_ct_cert_api::IamCtCertApi,
                    iam_ct_cert_manage_api::IamCtCertManageApi,
                    iam_ct_role_api::IamCtRoleApi,
                    iam_ct_res_api::IamCtResApi,
                ),
                (
                    iam_ca_account_api::IamCaAccountApi,
                    iam_ca_app_api::IamCaAppApi,
                    iam_ca_role_api::IamCaRoleApi,
                    iam_ca_cert_manage_api::IamCaCertManageApi,
                    iam_ca_res_api::IamCaResApi,
                ),
                (
                    iam_ci_cert_api::IamCiCertManageApi,
                    iam_ci_cert_api::IamCiCertApi,
                    iam_ci_cert_api::IamCiLdapCertApi,
                    iam_ci_app_api::IamCiAppApi,
                    iam_ci_res_api::IamCiResApi,
                    iam_ci_role_api::IamCiRoleApi,
                    iam_ci_account_api::IamCiAccountApi,
                ),
            ))
            .middleware(EncryptMW),
        )
        .await;
    Ok(())
}

pub async fn init_db(mut funs: TardisFunsInst) -> TardisResult<Option<(String, String)>> {
    bios_basic::rbum::rbum_initializer::init(funs.module_code(), funs.conf::<IamConfig>().rbum.clone()).await?;
    funs.begin().await?;
    let ctx = get_first_account_context(iam_constants::RBUM_KIND_CODE_IAM_ACCOUNT, iam_constants::COMPONENT_CODE, &funs).await?;
    let sysadmin_info = if let Some(ctx) = ctx {
        init_basic_info(&funs, &ctx).await?;
        None
    } else {
        let db_kind = TardisFuns::reldb().backend();
        let compatible_type = TardisFuns::reldb().compatible_type();
        funs.db().init(iam_tenant::ActiveModel::init(db_kind, None, compatible_type.clone())).await?;
        funs.db().init(iam_app::ActiveModel::init(db_kind, None, compatible_type.clone())).await?;
        funs.db().init(iam_role::ActiveModel::init(db_kind, None, compatible_type.clone())).await?;
        funs.db().init(iam_account::ActiveModel::init(db_kind, None, compatible_type.clone())).await?;
        funs.db().init(iam_res::ActiveModel::init(db_kind, None, compatible_type.clone())).await?;
        funs.db().init(iam_config::ActiveModel::init(db_kind, None, compatible_type.clone())).await?;
        let (name, password) = init_rbum_data(&funs).await?;
        Some((name, password))
    };
    funs.commit().await?;
    Ok(sysadmin_info)
}

async fn init_basic_info<'a>(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let kind_tenant_id = RbumKindServ::get_rbum_kind_id_by_code(iam_constants::RBUM_KIND_CODE_IAM_TENANT, funs)
        .await?
        .ok_or_else(|| funs.err().not_found("iam", "init", "not found tenant kind", ""))?;
    let kind_app_id =
        RbumKindServ::get_rbum_kind_id_by_code(iam_constants::RBUM_KIND_CODE_IAM_APP, funs).await?.ok_or_else(|| funs.err().not_found("iam", "init", "not found app kind", ""))?;
    let kind_role_id = RbumKindServ::get_rbum_kind_id_by_code(iam_constants::RBUM_KIND_CODE_IAM_ROLE, funs)
        .await?
        .ok_or_else(|| funs.err().not_found("iam", "init", "not found role kind", ""))?;
    let kind_account_id = RbumKindServ::get_rbum_kind_id_by_code(iam_constants::RBUM_KIND_CODE_IAM_ACCOUNT, funs)
        .await?
        .ok_or_else(|| funs.err().not_found("iam", "init", "not found account kind", ""))?;
    let kind_res_id =
        RbumKindServ::get_rbum_kind_id_by_code(iam_constants::RBUM_KIND_CODE_IAM_RES, funs).await?.ok_or_else(|| funs.err().not_found("iam", "init", "not found res kind", ""))?;

    let domain_iam_id =
        RbumDomainServ::get_rbum_domain_id_by_code(iam_constants::COMPONENT_CODE, funs).await?.ok_or_else(|| funs.err().not_found("iam", "init", "not found iam domain", ""))?;

    let roles = RbumItemServ::paginate_rbums(
        &RbumBasicFilterReq {
            rbum_kind_id: Some(kind_role_id.clone()),
            rbum_domain_id: Some(domain_iam_id.clone()),
            ..Default::default()
        },
        1,
        4,
        Some(false),
        None,
        funs,
        ctx,
    )
    .await?
    .records;

    let role_sys_admin_id = roles
        .iter()
        .find(|r| r.code == iam_constants::RBUM_ITEM_NAME_SYS_ADMIN_ROLE)
        .map(|r| r.id.clone())
        .ok_or_else(|| funs.err().not_found("iam", "init", "not found sys admin role", ""))?;

    let role_tenant_admin_id = roles
        .iter()
        .find(|r| r.code == iam_constants::RBUM_ITEM_NAME_TENANT_ADMIN_ROLE)
        .map(|r| r.id.clone())
        .ok_or_else(|| funs.err().not_found("iam", "init", "not found tenant admin role", ""))?;

    let role_tenant_audit_id = roles
        .iter()
        .find(|r| r.code == iam_constants::RBUM_ITEM_NAME_TENANT_AUDIT_ROLE)
        .map(|r| r.id.clone())
        .ok_or_else(|| funs.err().not_found("iam", "init", "not found tenant audit admin role", ""))?;

    let role_app_admin_id = roles
        .iter()
        .find(|r| r.code == iam_constants::RBUM_ITEM_NAME_APP_ADMIN_ROLE)
        .map(|r| r.id.clone())
        .ok_or_else(|| funs.err().not_found("iam", "init", "not found app admin role", ""))?;

    IamBasicInfoManager::set(BasicInfo {
        kind_tenant_id,
        kind_app_id,
        kind_account_id,
        kind_role_id,
        kind_res_id,
        domain_iam_id,
        role_sys_admin_id,
        role_tenant_audit_id,
        role_tenant_admin_id,
        role_app_admin_id,
    })?;
    Ok(())
}

pub async fn init_rbum_data(funs: &TardisFunsInst) -> TardisResult<(String, String)> {
    let default_account_id = TardisFuns::field.nanoid();

    let ctx = TardisContext {
        own_paths: "".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: default_account_id.clone(),
        ..Default::default()
    };

    let kind_tenant_id = add_kind(iam_constants::RBUM_KIND_CODE_IAM_TENANT, iam_constants::RBUM_EXT_TABLE_IAM_TENANT, funs, &ctx).await?;
    let kind_app_id = add_kind(iam_constants::RBUM_KIND_CODE_IAM_APP, iam_constants::RBUM_EXT_TABLE_IAM_APP, funs, &ctx).await?;
    let kind_role_id = add_kind(iam_constants::RBUM_KIND_CODE_IAM_ROLE, iam_constants::RBUM_EXT_TABLE_IAM_ROLE, funs, &ctx).await?;
    let kind_account_id = add_kind(iam_constants::RBUM_KIND_CODE_IAM_ACCOUNT, iam_constants::RBUM_EXT_TABLE_IAM_ACCOUNT, funs, &ctx).await?;
    let kind_res_id = add_kind(iam_constants::RBUM_KIND_CODE_IAM_RES, iam_constants::RBUM_EXT_TABLE_IAM_RES, funs, &ctx).await?;

    let domain_iam_id = add_domain(funs, &ctx).await?;

    IamBasicInfoManager::set(BasicInfo {
        kind_tenant_id: kind_tenant_id.to_string(),
        kind_app_id: kind_app_id.to_string(),
        kind_account_id: kind_account_id.to_string(),
        kind_role_id: kind_role_id.to_string(),
        kind_res_id: kind_res_id.to_string(),
        domain_iam_id: domain_iam_id.to_string(),
        role_sys_admin_id: "".to_string(),
        role_tenant_audit_id: "".to_string(),
        role_tenant_admin_id: "".to_string(),
        role_app_admin_id: "".to_string(),
    })?;

    // Init resources
    IamSetServ::init_set(IamSetKind::Org, iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, &ctx).await?;
    let (set_res_id, cate_ids) = IamSetServ::init_set(IamSetKind::Res, iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, &ctx).await?;
    if cate_ids.is_none() {
        return Err(funs.err().not_found("iam_initializer", "init_rbum_data", "not found resource", "404-iam-res-not-exist"));
    }
    let (cate_menu_id, cate_api_id) = cate_ids.unwrap_or_default();

    let (set_menu_cs_id, set_api_cs_id) = add_res(&set_res_id, &cate_menu_id, &cate_api_id, "cs", "System Console", funs, &ctx).await?;
    let (set_menu_ct_id, set_api_ct_id) = add_res(&set_res_id, &cate_menu_id, &cate_api_id, "ct", "Tenant Console", funs, &ctx).await?;
    let (set_menu_ca_id, set_api_ca_id) = add_res(&set_res_id, &cate_menu_id, &cate_api_id, "ca", "App Console", funs, &ctx).await?;

    init_menu_by_file(&set_res_id, &cate_menu_id, &funs.conf::<IamConfig>().init_menu_json_path, funs, &ctx).await?;

    // Init kernel certs
    IamCertServ::init_default_ident_conf(
        &IamCertConfUserPwdAddOrModifyReq {
            // TODO config
            ak_rule_len_min: 0,
            ak_rule_len_max: 40,
            sk_rule_len_min: 0,
            sk_rule_len_max: 40,
            sk_rule_need_num: false,
            sk_rule_need_uppercase: false,
            sk_rule_need_lowercase: false,
            sk_rule_need_spec_char: false,
            sk_lock_cycle_sec: 60,
            sk_lock_err_times: 6,
            sk_lock_duration_sec: 300,
            repeatable: true,
            expire_sec: 2592000,
        },
        Some(IamCertConfPhoneVCodeAddOrModifyReq { ak_note: None, ak_rule: None }),
        Some(IamCertConfMailVCodeAddOrModifyReq { ak_note: None, ak_rule: None }),
        None,
        funs,
        &ctx,
    )
    .await?;

    let pwd = IamCertServ::get_new_pwd();
    IamAccountServ::add_account_agg(
        &IamAccountAggAddReq {
            id: Some(TrimString(default_account_id.clone())),
            name: TrimString(iam_constants::RBUM_ITEM_NAME_SYS_ADMIN_ACCOUNT.to_string()),
            cert_user_name: TrimString(iam_constants::RBUM_ITEM_NAME_SYS_ADMIN_ACCOUNT.to_string()),
            cert_password: Some(TrimString(pwd.clone())),
            cert_phone: None,
            cert_mail: None,
            icon: None,
            disabled: None,
            scope_level: None,
            role_ids: None,
            org_node_ids: None,
            exts: Default::default(),
            status: Some(RbumCertStatusKind::Pending),
            temporary: None,
            lock_status: None,
        },
        false,
        funs,
        &ctx,
    )
    .await?;

    // Init some roles
    let role_sys_admin_id = IamRoleServ::add_role_agg(
        &mut IamRoleAggAddReq {
            role: IamRoleAddReq {
                code: Some(TrimString(iam_constants::RBUM_ITEM_NAME_SYS_ADMIN_ROLE.to_string())),
                name: TrimString(iam_constants::RBUM_ITEM_NAME_SYS_ADMIN_ROLE.to_string()),
                icon: None,
                sort: None,
                scope_level: Some(iam_constants::RBUM_SCOPE_LEVEL_PRIVATE),
                disabled: None,
                kind: Some(IamRoleKind::System),
            },
            res_ids: Some(vec![set_menu_cs_id, set_api_cs_id]),
        },
        funs,
        &ctx,
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
            org_cate_ids: None,
            exts: None,
            status: None,
            cert_phone: None,
            cert_mail: None,
        },
        funs,
        &ctx,
    )
    .await?;

    let role_tenant_admin_id = IamRoleServ::add_role_agg(
        &mut IamRoleAggAddReq {
            role: IamRoleAddReq {
                code: Some(TrimString(iam_constants::RBUM_ITEM_NAME_TENANT_ADMIN_ROLE.to_string())),
                name: TrimString(iam_constants::RBUM_ITEM_NAME_TENANT_ADMIN_ROLE.to_string()),
                icon: None,
                sort: None,
                scope_level: Some(iam_constants::RBUM_SCOPE_LEVEL_TENANT),
                disabled: None,
                kind: Some(IamRoleKind::Tenant),
            },
            res_ids: Some(vec![set_menu_ct_id.clone(), set_api_ct_id.clone()]),
        },
        funs,
        &ctx,
    )
    .await?;

    let role_tenant_audit_id = IamRoleServ::add_role_agg(
        &mut IamRoleAggAddReq {
            role: IamRoleAddReq {
                code: Some(TrimString(iam_constants::RBUM_ITEM_NAME_TENANT_AUDIT_ROLE.to_string())),
                name: TrimString(iam_constants::RBUM_ITEM_NAME_TENANT_AUDIT_ROLE.to_string()),
                icon: None,
                sort: None,
                scope_level: Some(iam_constants::RBUM_SCOPE_LEVEL_TENANT),
                disabled: None,
                kind: Some(IamRoleKind::Tenant),
            },
            res_ids: Some(vec![set_menu_ct_id, set_api_ct_id]),
        },
        funs,
        &ctx,
    )
    .await?;

    let role_app_admin_id = IamRoleServ::add_role_agg(
        &mut IamRoleAggAddReq {
            role: IamRoleAddReq {
                code: Some(TrimString(iam_constants::RBUM_ITEM_NAME_APP_ADMIN_ROLE.to_string())),
                name: TrimString(iam_constants::RBUM_ITEM_NAME_APP_ADMIN_ROLE.to_string()),
                icon: None,
                sort: None,
                scope_level: Some(iam_constants::RBUM_SCOPE_LEVEL_APP),
                disabled: None,
                kind: Some(IamRoleKind::App),
            },
            res_ids: Some(vec![set_menu_ca_id.clone(), set_api_ca_id.clone()]),
        },
        funs,
        &ctx,
    )
    .await?;

    let _ = add_role(
        iam_constants::RBUM_ITEM_NAME_APP_ADMIN_OM_ROLE,
        iam_constants::RBUM_ITEM_NAME_APP_ADMIN_OM_ROLE,
        &iam_constants::RBUM_SCOPE_LEVEL_APP,
        &IamRoleKind::App,
        Some(vec![set_menu_ca_id.clone(), set_api_ca_id.clone()]),
        funs,
        &ctx,
    )
    .await?;
    let _ = add_role(
        iam_constants::RBUM_ITEM_NAME_APP_ADMIN_DEVELOP_ROLE,
        iam_constants::RBUM_ITEM_NAME_APP_ADMIN_DEVELOP_ROLE,
        &iam_constants::RBUM_SCOPE_LEVEL_APP,
        &IamRoleKind::App,
        Some(vec![set_menu_ca_id.clone(), set_api_ca_id.clone()]),
        funs,
        &ctx,
    )
    .await?;
    let _ = add_role(
        iam_constants::RBUM_ITEM_NAME_APP_ADMIN_PRODUCT_ROLE,
        iam_constants::RBUM_ITEM_NAME_APP_ADMIN_PRODUCT_ROLE,
        &iam_constants::RBUM_SCOPE_LEVEL_APP,
        &IamRoleKind::App,
        Some(vec![set_menu_ca_id.clone(), set_api_ca_id.clone()]),
        funs,
        &ctx,
    )
    .await?;
    let _ = add_role(
        iam_constants::RBUM_ITEM_NAME_APP_ADMIN_ITERATE_ROLE,
        iam_constants::RBUM_ITEM_NAME_APP_ADMIN_ITERATE_ROLE,
        &iam_constants::RBUM_SCOPE_LEVEL_APP,
        &IamRoleKind::App,
        Some(vec![set_menu_ca_id.clone(), set_api_ca_id.clone()]),
        funs,
        &ctx,
    )
    .await?;
    let _ = add_role(
        iam_constants::RBUM_ITEM_NAME_APP_ADMIN_TEST_ROLE,
        iam_constants::RBUM_ITEM_NAME_APP_ADMIN_TEST_ROLE,
        &iam_constants::RBUM_SCOPE_LEVEL_APP,
        &IamRoleKind::App,
        Some(vec![set_menu_ca_id.clone(), set_api_ca_id.clone()]),
        funs,
        &ctx,
    )
    .await?;
    let _ = add_role(
        iam_constants::RBUM_ITEM_NAME_APP_NORMAL_ROLE,
        iam_constants::RBUM_ITEM_NAME_APP_NORMAL_ROLE,
        &iam_constants::RBUM_SCOPE_LEVEL_APP,
        &IamRoleKind::App,
        Some(vec![set_menu_ca_id.clone(), set_api_ca_id.clone()]),
        funs,
        &ctx,
    )
    .await?;
    let _ = add_role(
        iam_constants::RBUM_ITEM_NAME_APP_NORMAL_OM_ROLE,
        iam_constants::RBUM_ITEM_NAME_APP_NORMAL_OM_ROLE,
        &iam_constants::RBUM_SCOPE_LEVEL_APP,
        &IamRoleKind::App,
        Some(vec![set_menu_ca_id.clone(), set_api_ca_id.clone()]),
        funs,
        &ctx,
    )
    .await?;
    let _ = add_role(
        iam_constants::RBUM_ITEM_NAME_APP_NORMAL_DEVELOP_ROLE,
        iam_constants::RBUM_ITEM_NAME_APP_NORMAL_DEVELOP_ROLE,
        &iam_constants::RBUM_SCOPE_LEVEL_APP,
        &IamRoleKind::App,
        Some(vec![set_menu_ca_id.clone(), set_api_ca_id.clone()]),
        funs,
        &ctx,
    )
    .await?;
    let _ = add_role(
        iam_constants::RBUM_ITEM_NAME_APP_NORMAL_PRODUCT_ROLE,
        iam_constants::RBUM_ITEM_NAME_APP_NORMAL_PRODUCT_ROLE,
        &iam_constants::RBUM_SCOPE_LEVEL_APP,
        &IamRoleKind::App,
        Some(vec![set_menu_ca_id.clone(), set_api_ca_id.clone()]),
        funs,
        &ctx,
    )
    .await?;
    let _ = add_role(
        iam_constants::RBUM_ITEM_NAME_APP_NORMAL_ITERATE_ROLE,
        iam_constants::RBUM_ITEM_NAME_APP_NORMAL_ITERATE_ROLE,
        &iam_constants::RBUM_SCOPE_LEVEL_APP,
        &IamRoleKind::App,
        Some(vec![set_menu_ca_id.clone(), set_api_ca_id.clone()]),
        funs,
        &ctx,
    )
    .await?;
    let _ = add_role(
        iam_constants::RBUM_ITEM_NAME_APP_NORMAL_TEST_ROLE,
        iam_constants::RBUM_ITEM_NAME_APP_NORMAL_TEST_ROLE,
        &iam_constants::RBUM_SCOPE_LEVEL_APP,
        &IamRoleKind::App,
        Some(vec![set_menu_ca_id.clone(), set_api_ca_id.clone()]),
        funs,
        &ctx,
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
        role_tenant_audit_id,
        role_app_admin_id,
    })?;

    info!(
        "Initialization is complete.
-----------
System administrator name: {} ,Initial password: {}
-----------",
        iam_constants::RBUM_ITEM_NAME_SYS_ADMIN_ACCOUNT,
        pwd
    );
    Ok((iam_constants::RBUM_ITEM_NAME_SYS_ADMIN_ACCOUNT.to_string(), pwd))
}

async fn add_kind<'a>(scheme: &str, ext_table: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
    RbumKindServ::add_rbum(
        &mut RbumKindAddReq {
            code: TrimString(scheme.to_string()),
            name: TrimString(scheme.to_string()),
            note: None,
            icon: None,
            sort: None,
            module: None,
            ext_table_name: Some(ext_table.to_string().to_lowercase()),
            scope_level: Some(iam_constants::RBUM_SCOPE_LEVEL_GLOBAL),
        },
        funs,
        ctx,
    )
    .await
}

async fn add_domain<'a>(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
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
        ctx,
    )
    .await
}

async fn add_role<'a>(
    code: &str,
    name: &str,
    scope_level: &RbumScopeLevelKind,
    kind: &IamRoleKind,
    res_ids: Option<Vec<String>>,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<String> {
    let role_id = IamRoleServ::add_role_agg(
        &mut IamRoleAggAddReq {
            role: IamRoleAddReq {
                code: Some(TrimString(code.to_string())),
                name: TrimString(name.to_string()),
                kind: Some(kind.clone()),
                scope_level: Some(scope_level.clone()),
                disabled: None,
                icon: None,
                sort: None,
            },
            res_ids,
        },
        funs,
        ctx,
    )
    .await?;
    Ok(role_id)
}

async fn init_menu_by_file(set_id: &str, parent_cate_id: &str, file_path: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let json_menu = TardisFuns::json.file_to_obj::<JsonMenu, &str>(file_path)?;
    IamMenuServ::parse_menu(set_id, parent_cate_id, json_menu, funs, ctx).await?;
    Ok(())
}

async fn add_res<'a>(
    set_res_id: &str,
    cate_menu_id: &str,
    cate_api_id: &str,
    code: &str,
    name: &str,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<(String, String)> {
    let res_menu_id = IamResServ::add_res_agg(
        &mut IamResAggAddReq {
            res: IamResAddReq {
                code: TrimString(format!("{}/{}", iam_constants::COMPONENT_CODE.to_lowercase(), code)),
                name: TrimString(name.to_string()),
                kind: IamResKind::Menu,
                icon: None,
                sort: None,
                method: None,
                hide: None,
                action: None,
                scope_level: Some(RBUM_SCOPE_LEVEL_GLOBAL),
                disabled: None,
                crypto_req: None,
                crypto_resp: None,
                double_auth: None,
                double_auth_msg: None,
                need_login: None,
            },
            set: IamSetItemAggAddReq {
                set_cate_id: cate_menu_id.to_string(),
            },
        },
        set_res_id,
        funs,
        ctx,
    )
    .await?;
    let res_api_id = IamResServ::add_res_agg(
        &mut IamResAggAddReq {
            res: IamResAddReq {
                code: TrimString(format!("{}/{}/**", iam_constants::COMPONENT_CODE.to_lowercase(), code)),
                name: TrimString(name.to_string()),
                kind: IamResKind::Api,
                icon: None,
                sort: None,
                method: None,
                hide: None,
                action: None,
                scope_level: Some(RBUM_SCOPE_LEVEL_GLOBAL),
                disabled: None,
                crypto_req: Some(false),
                crypto_resp: Some(false),
                double_auth: Some(false),
                double_auth_msg: None,
                need_login: None,
            },
            set: IamSetItemAggAddReq {
                set_cate_id: cate_api_id.to_string(),
            },
        },
        set_res_id,
        funs,
        ctx,
    )
    .await?;
    Ok((res_menu_id, res_api_id))
}

pub async fn truncate_data<'a>(funs: &TardisFunsInst) -> TardisResult<()> {
    rbum_initializer::truncate_data(funs).await?;
    funs.db().execute(Table::truncate().table(iam_account::Entity)).await?;
    funs.db().execute(Table::truncate().table(iam_app::Entity)).await?;
    funs.db().execute(Table::truncate().table(iam_res::Entity)).await?;
    funs.db().execute(Table::truncate().table(iam_role::Entity)).await?;
    funs.db().execute(Table::truncate().table(iam_tenant::Entity)).await?;
    funs.db().execute(Table::truncate().table(iam_config::Entity)).await?;
    funs.cache().flushdb().await?;
    Ok(())
}
