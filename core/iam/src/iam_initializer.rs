use bios_basic::rbum::dto::rbum_set_cate_dto::RbumSetCateAddReq;
use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use bios_basic::rbum::serv::rbum_set_serv::RbumSetCateServ;
use serde::Deserialize;
use serde::Serialize;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm::sea_query::Table;
use tardis::futures::executor;
use tardis::log::{info, warn};
use tardis::web::web_server::TardisWebServer;
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

use crate::basic::domain::{iam_account, iam_app, iam_res, iam_role, iam_tenant};
use crate::basic::dto::iam_account_dto::{IamAccountAggAddReq, IamAccountAggModifyReq};
use crate::basic::dto::iam_cert_conf_dto::{
    IamCertConfLdapAddOrModifyReq, IamCertConfMailVCodeAddOrModifyReq, IamCertConfPhoneVCodeAddOrModifyReq, IamCertConfUserPwdAddOrModifyReq,
};
use crate::basic::dto::iam_res_dto::{IamResAddReq, IamResAggAddReq, JsonMenu};
use crate::basic::dto::iam_role_dto::{IamRoleAddReq, IamRoleAggAddReq};
use crate::basic::dto::iam_set_dto::IamSetItemAggAddReq;
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_res_serv::{IamMenuServ, IamResServ};
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::console_app::api::{iam_ca_account_api, iam_ca_app_api, iam_ca_res_api, iam_ca_role_api};
use crate::console_common::api::{iam_cc_account_api, iam_cc_org_api, iam_cc_res_api, iam_cc_role_api, iam_cc_system_api, iam_cc_tenant_api};
use crate::console_passport::api::{iam_cp_account_api, iam_cp_cert_api, iam_cp_tenant_api};
use crate::console_system::api::{iam_cs_account_api, iam_cs_account_attr_api, iam_cs_cert_api, iam_cs_res_api, iam_cs_role_api, iam_cs_tenant_api};
use crate::console_tenant::api::{
    iam_ct_account_api, iam_ct_account_attr_api, iam_ct_app_api, iam_ct_app_set_api, iam_ct_cert_api, iam_ct_cert_manage_api, iam_ct_org_api, iam_ct_res_api, iam_ct_role_api,
    iam_ct_tenant_api,
};
use crate::iam_config::{BasicInfo, IamBasicInfoManager, IamConfig};
use crate::iam_constants;
use crate::iam_constants::RBUM_SCOPE_LEVEL_GLOBAL;
use crate::iam_enumeration::{IamResKind, IamRoleKind, IamSetCateKind, IamSetKind};

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
                (
                    iam_cc_account_api::IamCcAccountApi,
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
                    iam_cs_role_api::IamCsRoleApi,
                    iam_cs_res_api::IamCsResApi,
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
                    iam_ca_res_api::IamCaResApi,
                ),
            ),
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
        funs.db().init(iam_tenant::ActiveModel::init(db_kind, None)).await?;
        funs.db().init(iam_app::ActiveModel::init(db_kind, None)).await?;
        funs.db().init(iam_role::ActiveModel::init(db_kind, None)).await?;
        funs.db().init(iam_account::ActiveModel::init(db_kind, None)).await?;
        funs.db().init(iam_res::ActiveModel::init(db_kind, None)).await?;
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
        3,
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
        role_tenant_admin_id: "".to_string(),
        role_app_admin_id: "".to_string(),
    })?;

    // Init resources
    IamSetServ::init_set(IamSetKind::Org, iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, &ctx).await?;
    let (set_res_id, cate_ids) = IamSetServ::init_set(IamSetKind::Res, iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, &ctx).await?;
    let (cate_menu_id, cate_api_id) = cate_ids.unwrap();

    let (set_menu_cs_id, set_api_cs_id) = add_res(&set_res_id, &cate_menu_id, &cate_api_id, "cs", "System Console", funs, &ctx).await?;
    let (set_menu_ct_id, set_api_ct_id) = add_res(&set_res_id, &cate_menu_id, &cate_api_id, "ct", "Tenant Console", funs, &ctx).await?;
    let (set_menu_ca_id, set_api_ca_id) = add_res(&set_res_id, &cate_menu_id, &cate_api_id, "ca", "App Console", funs, &ctx).await?;

    init_menu_by_file(&set_res_id, &cate_menu_id, ,funs, &ctx).await?
    // init_menu(&set_res_id, &cate_menu_id, funs, &ctx).await?;

    // Init kernel certs
    let mut iam_cert_conf_ldap_add_or_modify_req: Vec<IamCertConfLdapAddOrModifyReq> = vec![];
    for config in &funs.conf::<IamConfig>().ldap.client {
        iam_cert_conf_ldap_add_or_modify_req.push((*config).clone().into());
    }
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
            sk_lock_err_times: 3,
            sk_lock_duration_sec: 300,
            repeatable: true,
            expire_sec: 2592000,
        },
        Some(IamCertConfPhoneVCodeAddOrModifyReq { ak_note: None, ak_rule: None }),
        Some(IamCertConfMailVCodeAddOrModifyReq { ak_note: None, ak_rule: None }),
        Some(iam_cert_conf_ldap_add_or_modify_req),
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
            cert_password: TrimString(pwd.clone()),
            cert_phone: None,
            cert_mail: None,
            icon: None,
            disabled: None,
            scope_level: None,
            role_ids: None,
            org_node_ids: None,
            exts: Default::default(),
            status: None,
        },
        funs,
        &ctx,
    )
    .await?;

    // Init some roles
    let role_sys_admin_id = IamRoleServ::add_role_agg(
        &mut IamRoleAggAddReq {
            role: IamRoleAddReq {
                code: TrimString(iam_constants::RBUM_ITEM_NAME_SYS_ADMIN_ROLE.to_string()),
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
        },
        funs,
        &ctx,
    )
    .await?;

    let role_tenant_admin_id = IamRoleServ::add_role_agg(
        &mut IamRoleAggAddReq {
            role: IamRoleAddReq {
                code: TrimString(iam_constants::RBUM_ITEM_NAME_TENANT_ADMIN_ROLE.to_string()),
                name: TrimString(iam_constants::RBUM_ITEM_NAME_TENANT_ADMIN_ROLE.to_string()),
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
                code: TrimString(iam_constants::RBUM_ITEM_NAME_APP_ADMIN_ROLE.to_string()),
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
                code: TrimString(code.to_string()),
                name: TrimString(name.to_string()),
                kind: Some(IamRoleKind::App),
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

#[deprecated]
async fn init_menu<'a>(set_id: &str, parent_cate_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    init_menu_workbench(set_id, parent_cate_id, funs, ctx).await?;
    init_menu_account_info(set_id, parent_cate_id, funs, ctx).await?;
    init_menu_knoledge(set_id, parent_cate_id, funs, ctx).await?;
    init_menu_app(set_id, parent_cate_id, funs, ctx).await?;
    init_menu_configuration(set_id, parent_cate_id, funs, ctx).await?;

    Ok(())
}

async fn init_menu_workbench<'a>(set_id: &str, parent_cate_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    // workbench
    let workbench_cate_id = add_cate_menu(set_id, parent_cate_id, "工作台", "__workbench__", &IamSetCateKind::Root, funs, ctx).await?;
    let _ = add_menu_res(set_id, workbench_cate_id.as_str(), "工作台", "workbench", funs, ctx).await?;
    Ok(())
}

async fn init_menu_account_info<'a>(set_id: &str, parent_cate_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    // account_info
    let personal_center_cate_id = add_cate_menu(set_id, parent_cate_id, "个人中心", "__personal_center__", &IamSetCateKind::Root, funs, ctx).await?;
    let account_info_cate_id = add_cate_menu(set_id, personal_center_cate_id.as_str(), "账号信息", "__account_info__", &IamSetCateKind::Root, funs, ctx).await?;
    let _ = add_menu_res(set_id, account_info_cate_id.as_str(), "账号信息", "account_info", funs, ctx).await?;
    Ok(())
}

async fn init_menu_knoledge<'a>(set_id: &str, parent_cate_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    // knowledge
    let collaboration_cate_id = add_cate_menu(set_id, parent_cate_id, "协作空间", "__collaboration__", &IamSetCateKind::Root, funs, ctx).await?;
    let knowledge_cate_id = add_cate_menu(set_id, collaboration_cate_id.as_str(), "知识库", "__knowledge__", &IamSetCateKind::Root, funs, ctx).await?;
    let _ = add_menu_res(set_id, knowledge_cate_id.as_str(), "知识库", "knowledge", funs, ctx).await?;
    let _ = add_ele_res(set_id, knowledge_cate_id.as_str(), "创建", "knowledge*list*create", funs, ctx).await?;
    Ok(())
}

async fn init_menu_app<'a>(set_id: &str, parent_cate_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    // app
    let app_cate_id = add_cate_menu(set_id, parent_cate_id, "项目", "__app__", &IamSetCateKind::Tenant, funs, ctx).await?;
    // app -> manage
    let app_manage_cate_id = add_cate_menu(set_id, app_cate_id.as_str(), "项目管理", "__app_manage__", &IamSetCateKind::Tenant, funs, ctx).await?;
    let _ = add_menu_res(set_id, app_manage_cate_id.as_str(), "项目管理", "app_manage", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_manage_cate_id.as_str(), "创建", "app*manage*create", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_manage_cate_id.as_str(), "更改状态", "app*manage*update*state", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_manage_cate_id.as_str(), "归档", "app*manage*archive", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_manage_cate_id.as_str(), "恢复", "app*manage*recover", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_manage_cate_id.as_str(), "删除", "app*manage*delete", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_manage_cate_id.as_str(), "配置", "app*manage*config", funs, ctx).await?;

    // app -> manage -> config
    let app_manage_config_cate_id = add_cate_menu(set_id, app_manage_cate_id.as_str(), "配置", "__app_manage_config__", &IamSetCateKind::App, funs, ctx).await?;
    // app -> manage -> config -> info
    let app_manage_config_info_cate_id = add_cate_menu(
        set_id,
        app_manage_config_cate_id.as_str(),
        "项目信息",
        "__app_manage_config_info__",
        &IamSetCateKind::App,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(set_id, app_manage_config_info_cate_id.as_str(), "项目信息", "app_manage_config_info", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_manage_config_info_cate_id.as_str(), "编辑", "app*manage*config*info*update", funs, ctx).await?;

    // app -> manage -> config -> notice
    let app_manage_config_notice_cate_id = add_cate_menu(
        set_id,
        app_manage_config_cate_id.as_str(),
        "通知配置",
        "__app_manage_config_notice__",
        &IamSetCateKind::App,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(set_id, app_manage_config_notice_cate_id.as_str(), "通知配置", "app_manage_config_notice", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_manage_config_notice_cate_id.as_str(), "编辑", "app*manage*config*notice*update", funs, ctx).await?;

    // app -> manage -> config -> func
    let app_manage_config_func_cate_id = add_cate_menu(
        set_id,
        app_manage_config_cate_id.as_str(),
        "功能配置",
        "__app_manage_config_func__",
        &IamSetCateKind::App,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(set_id, app_manage_config_func_cate_id.as_str(), "功能配置", "app_manage_config_func", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_manage_config_func_cate_id.as_str(), "编辑", "app*manage*config*func*update", funs, ctx).await?;

    // app -> manage -> config -> template
    let app_manage_config_template_cate_id = add_cate_menu(
        set_id,
        app_manage_config_cate_id.as_str(),
        "模板管理",
        "__app_manage_config_template__",
        &IamSetCateKind::App,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(set_id, app_manage_config_template_cate_id.as_str(), "功能配置", "app_manage_config_template", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_manage_config_template_cate_id.as_str(), "编辑", "app*manage*config*template*update", funs, ctx).await?;

    // app -> manage -> config -> account
    let app_manage_config_account_cate_id = add_cate_menu(
        set_id,
        app_manage_config_cate_id.as_str(),
        "人员管理",
        "__app_manage_config_account__",
        &IamSetCateKind::App,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(set_id, app_manage_config_account_cate_id.as_str(), "人员管理", "app_manage_config_account", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_manage_config_account_cate_id.as_str(), "创建", "app*manage*config*account*create", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_manage_config_account_cate_id.as_str(), "编辑", "app*manage*config*account*update", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_manage_config_account_cate_id.as_str(), "删除", "app*manage*config*account*delete", funs, ctx).await?;
    let _ = add_ele_res(
        set_id,
        app_manage_config_account_cate_id.as_str(),
        "查看权限",
        "app*manage*config*account*view_permissions",
        funs,
        ctx,
    )
    .await?;
    let _ = add_ele_res(set_id, app_manage_config_account_cate_id.as_str(), "添加人员", "app*manage*config*account*add", funs, ctx).await?;
    let _ = add_ele_res(
        set_id,
        app_manage_config_account_cate_id.as_str(),
        "移除人员",
        "app*manage*config*account*remove",
        funs,
        ctx,
    )
    .await?;

    // app -> manage -> config -> res
    let app_manage_config_res_cate_id = add_cate_menu(
        set_id,
        app_manage_config_cate_id.as_str(),
        "资源管理",
        "__app_manage_config_res__",
        &IamSetCateKind::App,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(set_id, app_manage_config_res_cate_id.as_str(), "资源管理", "app_manage_config_res", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_manage_config_res_cate_id.as_str(), "申请资源", "app*manage*config*res*apply", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_manage_config_res_cate_id.as_str(), "申请记录", "app*manage*config*res*record", funs, ctx).await?;

    // app -> manage -> config -> res -> apply page
    let app_manage_config_res_apply_page_cate_id = add_cate_menu(
        set_id,
        app_manage_config_cate_id.as_str(),
        "申请资源页面",
        "__app_manage_config_res_apply_page__",
        &IamSetCateKind::App,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(
        set_id,
        app_manage_config_res_apply_page_cate_id.as_str(),
        "申请资源页面",
        "app_manage_config_res_apply_page",
        funs,
        ctx,
    )
    .await?;

    // app -> manage -> config -> res -> inst page
    let app_manage_config_res_inst_page_cate_id = add_cate_menu(
        set_id,
        app_manage_config_cate_id.as_str(),
        "实例详情页面",
        "__app_manage_config_res_inst_page__",
        &IamSetCateKind::App,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(
        set_id,
        app_manage_config_res_inst_page_cate_id.as_str(),
        "实例详情页面",
        "app_manage_config_res_inst_page",
        funs,
        ctx,
    )
    .await?;

    // app -> manage -> config -> res -> apply record page
    let app_manage_config_res_apply_record_page_cate_id = add_cate_menu(
        set_id,
        app_manage_config_cate_id.as_str(),
        "申请记录页面",
        "__app_manage_config_res_apply_record_page__",
        &IamSetCateKind::App,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(
        set_id,
        app_manage_config_res_apply_record_page_cate_id.as_str(),
        "申请记录页面",
        "app_manage_config_res_apply_record_page",
        funs,
        ctx,
    )
    .await?;
    // app -> manage -> config -> res -> apply record page -> detail
    let app_manage_config_res_apply_record_page_detail_cate_id = add_cate_menu(
        set_id,
        app_manage_config_cate_id.as_str(),
        "申请详情页面",
        "__app_manage_config_res_apply_record_page_detail__",
        &IamSetCateKind::App,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(
        set_id,
        app_manage_config_res_apply_record_page_detail_cate_id.as_str(),
        "申请详情页面",
        "app_manage_config_res_apply_record_page_detail",
        funs,
        ctx,
    )
    .await?;

    // app -> manage -> overview
    let app_manage_overview_cate_id = add_cate_menu(
        set_id,
        app_manage_config_cate_id.as_str(),
        "概览",
        "__app_manage_config_overview__",
        &IamSetCateKind::App,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(set_id, app_manage_overview_cate_id.as_str(), "概览", "app_manage_config_overview", funs, ctx).await?;

    // app -> milestone
    let app_milestone_cate_id = add_cate_menu(set_id, app_cate_id.as_str(), "里程碑", "__app_milestone__", &IamSetCateKind::App, funs, ctx).await?;
    let _ = add_menu_res(set_id, app_milestone_cate_id.as_str(), "里程碑", "app_config_milestone", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_milestone_cate_id.as_str(), "创建", "app*milestone*create", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_milestone_cate_id.as_str(), "编辑", "app*milestone*update", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_milestone_cate_id.as_str(), "更改状态", "app*milestone*update*status", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_milestone_cate_id.as_str(), "关闭", "app*milestone*closed", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_milestone_cate_id.as_str(), "删除", "app*milestone*delete", funs, ctx).await?;

    // app -> need
    let app_need_cate_id = add_cate_menu(set_id, app_cate_id.as_str(), "需求", "__app_manage_config_need__", &IamSetCateKind::App, funs, ctx).await?;
    let _ = add_menu_res(set_id, app_need_cate_id.as_str(), "需求", "app_need", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_need_cate_id.as_str(), "创建", "app*need*create", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_need_cate_id.as_str(), "变更状态", "app*need*update*status", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_need_cate_id.as_str(), "变更", "app*need*change*status", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_need_cate_id.as_str(), "编辑", "app**need*update", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_need_cate_id.as_str(), "指派", "app*need*assign", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_need_cate_id.as_str(), "建任务", "app*need*create*task", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_need_cate_id.as_str(), "关闭", "app*need*closed", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_need_cate_id.as_str(), "删除", "app*need*delete", funs, ctx).await?;

    // app -> task
    let app_task_cate_id = add_cate_menu(set_id, app_cate_id.as_str(), "任务", "__app_task__", &IamSetCateKind::App, funs, ctx).await?;
    let _ = add_menu_res(set_id, app_task_cate_id.as_str(), "任务", "app_task", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_task_cate_id.as_str(), "创建", "app*task*create", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_task_cate_id.as_str(), "更改状态", "app*task*update*status", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_task_cate_id.as_str(), "编辑", "app*task*update", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_task_cate_id.as_str(), "删除", "app*task*delete", funs, ctx).await?;

    // app -> iterate
    let app_iterate_cate_id = add_cate_menu(set_id, app_cate_id.as_str(), "迭代", "__app_iterate__", &IamSetCateKind::App, funs, ctx).await?;
    let _ = add_menu_res(set_id, app_iterate_cate_id.as_str(), "迭代", "app_iterate", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_iterate_cate_id.as_str(), "创建", "app*iterate*create", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_iterate_cate_id.as_str(), "更改状态", "app*iterate*update*status", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_iterate_cate_id.as_str(), "编辑", "app*iterate*update", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_iterate_cate_id.as_str(), "人员", "app*iterate*account", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_iterate_cate_id.as_str(), "关联需求", "app*iterate*link*need", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_iterate_cate_id.as_str(), "建任务", "app*iterate*task*need", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_iterate_cate_id.as_str(), "关闭", "app*iterate*close", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_iterate_cate_id.as_str(), "删除", "app*iterate*delete", funs, ctx).await?;

    // app -> develop
    let app_develop_cate_id = add_cate_menu(set_id, app_cate_id.as_str(), "开发", "__app_develop__", &IamSetCateKind::App, funs, ctx).await?;

    // app -> develop -> app
    let app_develop_app_cate_id = add_cate_menu(set_id, app_develop_cate_id.as_str(), "工程管理", "__app_develop_app__", &IamSetCateKind::App, funs, ctx).await?;
    let _ = add_menu_res(set_id, app_develop_app_cate_id.as_str(), "工程管理", "app_develop_app", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_develop_app_cate_id.as_str(), "创建", "app*develop*app*create", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_develop_app_cate_id.as_str(), "配置", "app*develop*app*config", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_develop_app_cate_id.as_str(), "编辑", "app*develop*app*update", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_develop_app_cate_id.as_str(), "复制", "app*develop*app*copy", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_develop_app_cate_id.as_str(), "删除", "app*develop*app*delete", funs, ctx).await?;

    // app -> develop -> env
    let app_develop_env_cate_id = add_cate_menu(set_id, app_develop_cate_id.as_str(), "环境管理", "__app_develop_env__", &IamSetCateKind::App, funs, ctx).await?;
    let _ = add_menu_res(set_id, app_develop_env_cate_id.as_str(), "环境管理", "app_develop_env", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_develop_env_cate_id.as_str(), "创建", "app*develop*env*create", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_develop_env_cate_id.as_str(), "配置工程", "app*develop*env*config", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_develop_app_cate_id.as_str(), "编辑", "app*develop*env*update", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_develop_app_cate_id.as_str(), "删除", "app*develop*env*delete", funs, ctx).await?;

    // app -> develop -> api
    let app_develop_api_cate_id = add_cate_menu(set_id, app_develop_cate_id.as_str(), "接口管理", "__app_develop_api__", &IamSetCateKind::App, funs, ctx).await?;
    let _ = add_menu_res(set_id, app_develop_api_cate_id.as_str(), "接口管理", "app_develop_api", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_develop_api_cate_id.as_str(), "创建", "app*develop*api*create", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_develop_api_cate_id.as_str(), "导入", "app*develop*api*Import", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_develop_api_cate_id.as_str(), "编辑", "app*develop*api*update", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_develop_api_cate_id.as_str(), "运行", "app*develop*api*run", funs, ctx).await?;
    let _ = add_ele_res(set_id, app_develop_api_cate_id.as_str(), "删除", "app*develop*api*delete", funs, ctx).await?;
    Ok(())
}

async fn init_menu_configuration<'a>(set_id: &str, parent_cate_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    // configuration
    let configuration_cate_id = add_cate_menu(set_id, parent_cate_id, "配置", "__configuration__", &IamSetCateKind::Root, funs, ctx).await?;
    init_menu_configuration_user_and_org(set_id, configuration_cate_id.as_str(), funs, ctx).await?;
    init_menu_configuration_menu_and_role(set_id, configuration_cate_id.as_str(), funs, ctx).await?;
    init_menu_configuration_res_and_cert(set_id, configuration_cate_id.as_str(), funs, ctx).await?;
    init_menu_configuration_template_and_tag(set_id, configuration_cate_id.as_str(), funs, ctx).await?;
    init_menu_configuration_notice(set_id, configuration_cate_id.as_str(), funs, ctx).await?;
    Ok(())
}

async fn init_menu_configuration_user_and_org<'a>(set_id: &str, parent_cate_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    // configuration -> user and org
    let user_and_org_cate_id = add_cate_menu(set_id, parent_cate_id, "用户与组织", "__user_and_org__", &IamSetCateKind::Root, funs, ctx).await?;

    // configuration -> user and org -> tenant
    let tenant_cate_id = add_cate_menu(set_id, user_and_org_cate_id.as_str(), "租户管理", "__tenant__", &IamSetCateKind::System, funs, ctx).await?;
    let _ = add_menu_res(set_id, tenant_cate_id.as_str(), "租户管理", "tenant", funs, ctx).await?;
    let _ = add_ele_res(set_id, tenant_cate_id.as_str(), "创建", "tenant*create", funs, ctx).await?;
    let _ = add_ele_res(set_id, tenant_cate_id.as_str(), "编辑", "tenant*update", funs, ctx).await?;
    let _ = add_ele_res(set_id, tenant_cate_id.as_str(), "启用", "tenant*enable", funs, ctx).await?;
    let _ = add_ele_res(set_id, tenant_cate_id.as_str(), "禁用", "tenant*disable", funs, ctx).await?;
    let _ = add_ele_res(set_id, tenant_cate_id.as_str(), "人员", "tenant*personal", funs, ctx).await?;
    let _ = add_ele_res(set_id, tenant_cate_id.as_str(), "删除", "tenant*delete", funs, ctx).await?;

    // configuration -> user and org -> tenant -> tenant create page
    let tenant_create_page_cate_id = add_cate_menu(set_id, tenant_cate_id.as_str(), "创建页面", "__tenant_create_page__", &IamSetCateKind::System, funs, ctx).await?;
    let _ = add_menu_res(set_id, tenant_create_page_cate_id.as_str(), "创建页面", "tenant_create_page", funs, ctx).await?;

    // configuration -> user and org -> tenant -> tenant detail page
    let tenant_detail_page_id = add_cate_menu(set_id, tenant_cate_id.as_str(), "详情页面", "__tenant_detail_page__", &IamSetCateKind::System, funs, ctx).await?;
    let _ = add_menu_res(set_id, tenant_detail_page_id.as_str(), "详情页面", "tenant_detail_page", funs, ctx).await?;

    // configuration -> user and org -> tenant -> tenant update page
    let tenant_update_page_id = add_cate_menu(set_id, tenant_cate_id.as_str(), "编辑页面", "__tenant_update_page__", &IamSetCateKind::System, funs, ctx).await?;
    let _ = add_menu_res(set_id, tenant_update_page_id.as_str(), "编辑页面", "tenant_update_page", funs, ctx).await?;

    // configuration -> user and org -> tenant -> tenant personal page
    let tenant_personal_page_id = add_cate_menu(set_id, tenant_cate_id.as_str(), "人员页面", "__tenant_personal_page__", &IamSetCateKind::System, funs, ctx).await?;
    let _ = add_menu_res(set_id, tenant_personal_page_id.as_str(), "人员页面", "tenant_personal_page", funs, ctx).await?;
    let _ = add_ele_res(set_id, tenant_personal_page_id.as_str(), "重置密码", "tenant*personal*resetPwd", funs, ctx).await?;

    // configuration -> user and org -> tenant info
    let tenant_info_cate_id = add_cate_menu(set_id, user_and_org_cate_id.as_str(), "租户信息", "__tenant_info__", &IamSetCateKind::Tenant, funs, ctx).await?;
    let _ = add_menu_res(set_id, tenant_info_cate_id.as_str(), "租户信息", "tenant_info", funs, ctx).await?;
    let _ = add_ele_res(set_id, tenant_info_cate_id.as_str(), "编辑", "tenant_info*update", funs, ctx).await?;

    // configuration -> user and org -> org
    let org_cate_id = add_cate_menu(set_id, user_and_org_cate_id.as_str(), "组织架构", "__org__", &IamSetCateKind::Tenant, funs, ctx).await?;
    let _ = add_menu_res(set_id, org_cate_id.as_str(), "组织架构", "org", funs, ctx).await?;
    let _ = add_ele_res(set_id, org_cate_id.as_str(), "创建", "org*create", funs, ctx).await?;
    let _ = add_ele_res(set_id, org_cate_id.as_str(), "编辑", "org*update", funs, ctx).await?;
    let _ = add_ele_res(set_id, org_cate_id.as_str(), "删除", "org*delete", funs, ctx).await?;
    let _ = add_ele_res(set_id, org_cate_id.as_str(), "添加账号", "org*add*account", funs, ctx).await?;
    let _ = add_ele_res(set_id, org_cate_id.as_str(), "移除账号", "org*delete*account", funs, ctx).await?;

    // configuration -> user and org -> personnel management
    let personnel_management_tenant_cate_id = add_cate_menu(
        set_id,
        user_and_org_cate_id.as_str(),
        "人员管理",
        "__personnel_management__",
        &IamSetCateKind::Tenant,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(set_id, personnel_management_tenant_cate_id.as_str(), "人员管理", "personnel_management", funs, ctx).await?;
    let _ = add_ele_res(set_id, personnel_management_tenant_cate_id.as_str(), "创建", "personnel_management*create", funs, ctx).await?;
    let _ = add_ele_res(set_id, personnel_management_tenant_cate_id.as_str(), "编辑", "personnel_management*update", funs, ctx).await?;
    let _ = add_ele_res(set_id, personnel_management_tenant_cate_id.as_str(), "删除", "personnel_management*delete", funs, ctx).await?;
    let _ = add_ele_res(set_id, personnel_management_tenant_cate_id.as_str(), "重置密码", "personnel_management*resetPwd", funs, ctx).await?;

    // configuration -> user and org -> personnel management -> create page
    let personnel_management_create_page_cate_id = add_cate_menu(
        set_id,
        personnel_management_tenant_cate_id.as_str(),
        "创建页面",
        "__personnel_management_create_page__",
        &IamSetCateKind::Tenant,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(
        set_id,
        personnel_management_create_page_cate_id.as_str(),
        "创建页面",
        "personnel_management_create_page",
        funs,
        ctx,
    )
    .await?;

    // configuration -> user and org -> personnel management -> detail page
    let personnel_management_detail_page_id = add_cate_menu(
        set_id,
        personnel_management_tenant_cate_id.as_str(),
        "详情页面",
        "__personnel_management_detail_page__",
        &IamSetCateKind::Tenant,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(
        set_id,
        personnel_management_detail_page_id.as_str(),
        "详情页面",
        "personnel_management_detail_page",
        funs,
        ctx,
    )
    .await?;

    // configuration -> user and org -> personnel management -> update page
    let personnel_management_update_page_id = add_cate_menu(
        set_id,
        personnel_management_tenant_cate_id.as_str(),
        "编辑页面",
        "__personnel_management_update_page__",
        &IamSetCateKind::Tenant,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(
        set_id,
        personnel_management_update_page_id.as_str(),
        "编辑页面",
        "personnel_management_update_page",
        funs,
        ctx,
    )
    .await?;

    // configuration -> user and org -> apps
    let apps_cate_id = add_cate_menu(set_id, user_and_org_cate_id.as_str(), "项目组", "__apps__", &IamSetCateKind::Tenant, funs, ctx).await?;
    let _ = add_menu_res(set_id, apps_cate_id.as_str(), "项目组", "apps", funs, ctx).await?;
    let _ = add_ele_res(set_id, apps_cate_id.as_str(), "创建", "apps*create", funs, ctx).await?;
    let _ = add_ele_res(set_id, apps_cate_id.as_str(), "重命名", "apps*rename", funs, ctx).await?;
    let _ = add_ele_res(set_id, apps_cate_id.as_str(), "删除", "apps*delete", funs, ctx).await?;
    let _ = add_ele_res(set_id, apps_cate_id.as_str(), "添加项目", "apps*add*app", funs, ctx).await?;
    let _ = add_ele_res(set_id, apps_cate_id.as_str(), "移除项目", "apps*delete*app", funs, ctx).await?;
    let _ = add_ele_res(set_id, apps_cate_id.as_str(), "添加人员", "apps*add*account", funs, ctx).await?;
    let _ = add_ele_res(set_id, apps_cate_id.as_str(), "移除人员", "apps*delete*account", funs, ctx).await?;
    Ok(())
}

async fn init_menu_configuration_menu_and_role<'a>(set_id: &str, parent_cate_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    // configuration -> menu and role
    let menu_and_role_cate_id = add_cate_menu(set_id, parent_cate_id, "菜单与权限", "__menu_and_role__", &IamSetCateKind::System, funs, ctx).await?;

    // configuration -> menu and role -> menu
    let menu_cate_id = add_cate_menu(set_id, menu_and_role_cate_id.as_str(), "菜单管理", "__menu__", &IamSetCateKind::System, funs, ctx).await?;
    let _ = add_menu_res(set_id, menu_cate_id.as_str(), "菜单管理", "menu", funs, ctx).await?;
    let _ = add_ele_res(set_id, menu_cate_id.as_str(), "创建", "menu*create", funs, ctx).await?;
    let _ = add_ele_res(set_id, menu_cate_id.as_str(), "编辑", "menu*update", funs, ctx).await?;
    let _ = add_ele_res(set_id, menu_cate_id.as_str(), "删除", "menu*delete", funs, ctx).await?;
    let _ = add_ele_res(set_id, menu_cate_id.as_str(), "添加授权API", "menu*add*api", funs, ctx).await?;
    let _ = add_ele_res(set_id, menu_cate_id.as_str(), "移除授权API", "menu*delete*api", funs, ctx).await?;
    let _ = add_ele_res(set_id, menu_cate_id.as_str(), "添加按钮", "menu*add*ele", funs, ctx).await?;
    let _ = add_ele_res(set_id, menu_cate_id.as_str(), "编辑按钮", "menu*update*ele", funs, ctx).await?;
    let _ = add_ele_res(set_id, menu_cate_id.as_str(), "删除按钮", "menu*delete*ele", funs, ctx).await?;

    // configuration -> menu and role -> api
    let api_cate_id = add_cate_menu(set_id, menu_and_role_cate_id.as_str(), "Api管理", "__api__", &IamSetCateKind::System, funs, ctx).await?;
    let _ = add_menu_res(set_id, api_cate_id.as_str(), "Api管理", "api", funs, ctx).await?;
    let _ = add_ele_res(set_id, api_cate_id.as_str(), "创建", "api*create", funs, ctx).await?;
    let _ = add_ele_res(set_id, api_cate_id.as_str(), "编辑", "api*update", funs, ctx).await?;
    let _ = add_ele_res(set_id, api_cate_id.as_str(), "删除", "api*delete", funs, ctx).await?;

    // configuration -> menu and role -> api -> create page
    let api_create_page_cate_id = add_cate_menu(set_id, api_cate_id.as_str(), "创建页面", "__api_create_page__", &IamSetCateKind::System, funs, ctx).await?;
    let _ = add_menu_res(set_id, api_create_page_cate_id.as_str(), "创建页面", "api_create_page", funs, ctx).await?;

    // configuration -> menu and role -> api -> update page
    let api_update_page_cate_id = add_cate_menu(set_id, api_cate_id.as_str(), "编辑页面", "__api_update_page__", &IamSetCateKind::System, funs, ctx).await?;
    let _ = add_menu_res(set_id, api_update_page_cate_id.as_str(), "编辑页面", "api_update_page", funs, ctx).await?;

    // configuration -> menu and role -> api -> detail page
    let api_detail_page_id = add_cate_menu(set_id, api_cate_id.as_str(), "详情页面", "__api_detail_page__", &IamSetCateKind::System, funs, ctx).await?;
    let _ = add_menu_res(set_id, api_detail_page_id.as_str(), "详情页面", "api_detail_page", funs, ctx).await?;

    // configuration -> menu and role -> role
    let api_cate_id = add_cate_menu(set_id, menu_and_role_cate_id.as_str(), "角色管理", "__role__", &IamSetCateKind::Root, funs, ctx).await?;
    let _ = add_menu_res(set_id, api_cate_id.as_str(), "角色管理", "role", funs, ctx).await?;
    let _ = add_ele_res(set_id, api_cate_id.as_str(), "创建", "role*create", funs, ctx).await?;
    let _ = add_ele_res(set_id, api_cate_id.as_str(), "编辑", "role*update", funs, ctx).await?;
    let _ = add_ele_res(set_id, api_cate_id.as_str(), "删除", "role*delete", funs, ctx).await?;
    let _ = add_ele_res(set_id, api_cate_id.as_str(), "添加人员", "role*add*account", funs, ctx).await?;
    let _ = add_ele_res(set_id, api_cate_id.as_str(), "移除人员", "role*delete*account", funs, ctx).await?;
    let _ = add_ele_res(set_id, api_cate_id.as_str(), "查看权限", "role*list*permission", funs, ctx).await?;
    Ok(())
}

async fn init_menu_configuration_res_and_cert<'a>(set_id: &str, parent_cate_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    // configuration -> res and cert
    let res_and_cert_cate_id = add_cate_menu(set_id, parent_cate_id, "资源与凭证", "__res_and_cert__", &IamSetCateKind::Tenant, funs, ctx).await?;

    // configuration -> res and cert -> res
    let res_cate_id = add_cate_menu(set_id, res_and_cert_cate_id.as_str(), "资源管理", "__res__", &IamSetCateKind::Tenant, funs, ctx).await?;
    let _ = add_menu_res(set_id, res_cate_id.as_str(), "资源管理", "res", funs, ctx).await?;
    let _ = add_ele_res(set_id, res_cate_id.as_str(), "创建", "res*create", funs, ctx).await?;
    let _ = add_ele_res(set_id, res_cate_id.as_str(), "编辑", "res*update", funs, ctx).await?;
    let _ = add_ele_res(set_id, res_cate_id.as_str(), "连接凭证", "res*link*cert", funs, ctx).await?;
    let _ = add_ele_res(set_id, res_cate_id.as_str(), "启用", "res*enable", funs, ctx).await?;
    let _ = add_ele_res(set_id, res_cate_id.as_str(), "禁用", "res*disabled", funs, ctx).await?;
    let _ = add_ele_res(set_id, res_cate_id.as_str(), "删除", "res*delete", funs, ctx).await?;

    // configuration -> res and cert -> res -> create page
    let res_create_page_cate_id = add_cate_menu(set_id, res_cate_id.as_str(), "创建资源页面", "__res_create_page__", &IamSetCateKind::Tenant, funs, ctx).await?;
    let _ = add_menu_res(set_id, res_create_page_cate_id.as_str(), "创建资源页面", "res_create_page", funs, ctx).await?;

    // configuration -> res and cert -> res -> update page
    let res_update_page_cate_id = add_cate_menu(set_id, res_cate_id.as_str(), "编辑资源页面", "__res_update_page__", &IamSetCateKind::Tenant, funs, ctx).await?;
    let _ = add_menu_res(set_id, res_update_page_cate_id.as_str(), "编辑资源页面", "res_update_page", funs, ctx).await?;

    // configuration -> res and cert -> res -> detail page
    let res_detail_page_id = add_cate_menu(set_id, res_cate_id.as_str(), "详情资源页面", "__res_detail_page__", &IamSetCateKind::Tenant, funs, ctx).await?;
    let _ = add_menu_res(set_id, res_detail_page_id.as_str(), "详情资源页面", "res_detail_page", funs, ctx).await?;

    // configuration -> res and cert -> res -> cert page
    let res_detail_page_id = add_cate_menu(set_id, res_cate_id.as_str(), "凭证页面", "__res_cert_page__", &IamSetCateKind::Tenant, funs, ctx).await?;
    let _ = add_menu_res(set_id, res_detail_page_id.as_str(), "凭证页面", "res_cret_page", funs, ctx).await?;

    // configuration -> res and cert -> res apply
    let res_apply_cate_id = add_cate_menu(set_id, res_and_cert_cate_id.as_str(), "申请管理", "__res_apply__", &IamSetCateKind::Tenant, funs, ctx).await?;
    let _ = add_menu_res(set_id, res_apply_cate_id.as_str(), "申请管理", "res_apply", funs, ctx).await?;
    let _ = add_ele_res(set_id, res_apply_cate_id.as_str(), "审批", "res_apply*approval", funs, ctx).await?;
    let _ = add_ele_res(set_id, res_cate_id.as_str(), "连接凭证", "res_apply*link*cert", funs, ctx).await?;

    // configuration -> res and cert -> res -> approval page
    let res_apply_approval_page_cate_id = add_cate_menu(set_id, res_cate_id.as_str(), "审批页面", "__res_apply_approval_page__", &IamSetCateKind::Tenant, funs, ctx).await?;
    let _ = add_menu_res(set_id, res_apply_approval_page_cate_id.as_str(), "审批页面", "res_apply_approval_page", funs, ctx).await?;

    // configuration -> res and cert -> res apply -> detail page
    let res_apply_detail_page_id = add_cate_menu(set_id, res_cate_id.as_str(), "详情页面", "__res_apply_detail_page__", &IamSetCateKind::Tenant, funs, ctx).await?;
    let _ = add_menu_res(set_id, res_apply_detail_page_id.as_str(), "详情页面", "res_apply_detail_page", funs, ctx).await?;

    // configuration -> res and cert -> res apply -> cert page
    let res_apply_detail_page_id = add_cate_menu(set_id, res_cate_id.as_str(), "凭证页面", "__res_apply_cert_page__", &IamSetCateKind::Tenant, funs, ctx).await?;
    let _ = add_menu_res(set_id, res_apply_detail_page_id.as_str(), "凭证页面", "res_apply_cret_page", funs, ctx).await?;

    // configuration -> res and cert -> cert
    let cert_cate_id = add_cate_menu(set_id, res_and_cert_cate_id.as_str(), "凭证管理", "__cert__", &IamSetCateKind::Tenant, funs, ctx).await?;
    let _ = add_menu_res(set_id, cert_cate_id.as_str(), "凭证管理", "cert", funs, ctx).await?;
    let _ = add_ele_res(set_id, cert_cate_id.as_str(), "创建", "cert*create", funs, ctx).await?;
    let _ = add_ele_res(set_id, cert_cate_id.as_str(), "编辑", "cert*update", funs, ctx).await?;
    let _ = add_ele_res(set_id, cert_cate_id.as_str(), "删除", "cert*delete", funs, ctx).await?;

    // configuration -> res and cert -> cert -> create page
    let cert_create_page_cate_id = add_cate_menu(set_id, cert_cate_id.as_str(), "创建页面", "__cert_create_page__", &IamSetCateKind::Tenant, funs, ctx).await?;
    let _ = add_menu_res(set_id, cert_create_page_cate_id.as_str(), "创建页面", "cert_create_page", funs, ctx).await?;

    // configuration -> res and cert -> cert -> detail page
    let cert_detail_page_id = add_cate_menu(set_id, cert_cate_id.as_str(), "详情页面", "__cert_detail_page__", &IamSetCateKind::Tenant, funs, ctx).await?;
    let _ = add_menu_res(set_id, cert_detail_page_id.as_str(), "详情页面", "cert_detail_page", funs, ctx).await?;

    // configuration -> res and cert -> cert -> update page
    let cert_detail_page_id = add_cate_menu(set_id, cert_cate_id.as_str(), "编辑页面", "__cert_update_page__", &IamSetCateKind::Tenant, funs, ctx).await?;
    let _ = add_menu_res(set_id, cert_detail_page_id.as_str(), "编辑页面", "cert_update_page", funs, ctx).await?;
    Ok(())
}

async fn init_menu_configuration_template_and_tag<'a>(set_id: &str, parent_cate_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    // configuration -> template and tag
    let template_and_tag_cate_id = add_cate_menu(set_id, parent_cate_id, "模板与标签", "__template_and_tag__", &IamSetCateKind::Tenant, funs, ctx).await?;
    // configuration -> template and tag -> template
    let template_cate_id = add_cate_menu(set_id, template_and_tag_cate_id.as_str(), "模板", "__template__", &IamSetCateKind::Tenant, funs, ctx).await?;
    // configuration -> template and tag -> template -> app
    let template_app_cate_id = add_cate_menu(set_id, template_cate_id.as_str(), "项目", "__template_app__", &IamSetCateKind::Tenant, funs, ctx).await?;

    // configuration -> template and tag -> template -> app -> app template
    let template_app_app_cate_id = add_cate_menu(
        set_id,
        template_app_cate_id.as_str(),
        "项目模板",
        "__template_app_app_template__",
        &IamSetCateKind::Tenant,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(set_id, template_app_app_cate_id.as_str(), "项目模板", "template_app_app_template", funs, ctx).await?;
    let _ = add_ele_res(set_id, template_app_app_cate_id.as_str(), "创建", "template*app*app*create", funs, ctx).await?;
    let _ = add_ele_res(set_id, template_app_app_cate_id.as_str(), "编辑", "template*app*app*update", funs, ctx).await?;
    let _ = add_ele_res(set_id, template_app_app_cate_id.as_str(), "删除", "template*app*app*delete", funs, ctx).await?;

    // configuration -> template and tag -> template -> app -> page template
    let template_app_page_cate_id = add_cate_menu(
        set_id,
        template_app_cate_id.as_str(),
        "页面模板",
        "__template_app_page_template__",
        &IamSetCateKind::Tenant,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(set_id, template_app_page_cate_id.as_str(), "页面模板", "template_app_page_template", funs, ctx).await?;
    let _ = add_ele_res(set_id, template_app_page_cate_id.as_str(), "创建", "template*app*page*create", funs, ctx).await?;
    let _ = add_ele_res(set_id, template_app_page_cate_id.as_str(), "编辑", "template*app*page*update", funs, ctx).await?;
    let _ = add_ele_res(set_id, template_app_page_cate_id.as_str(), "删除", "template*app*page*delete", funs, ctx).await?;

    // configuration -> template and tag -> template -> app -> project template
    let template_app_project_cate_id = add_cate_menu(
        set_id,
        template_app_cate_id.as_str(),
        "工程模板",
        "__template_app_project_template__",
        &IamSetCateKind::Tenant,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(set_id, template_app_project_cate_id.as_str(), "工程模板", "template_app_project_template", funs, ctx).await?;
    let _ = add_ele_res(set_id, template_app_project_cate_id.as_str(), "创建", "template*app*project*create", funs, ctx).await?;
    let _ = add_ele_res(set_id, template_app_project_cate_id.as_str(), "编辑", "template*app*project*update", funs, ctx).await?;
    let _ = add_ele_res(set_id, template_app_project_cate_id.as_str(), "删除", "template*app*project*delete", funs, ctx).await?;

    // configuration -> template and tag -> template -> res
    let template_res_cate_id = add_cate_menu(set_id, template_cate_id.as_str(), "资源", "__template_res__", &IamSetCateKind::Tenant, funs, ctx).await?;

    // configuration -> template and tag -> template -> res -> res template
    let template_res_res_cate_id = add_cate_menu(
        set_id,
        template_res_cate_id.as_str(),
        "资源模板",
        "__template_res_res_template__",
        &IamSetCateKind::Tenant,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(set_id, template_res_res_cate_id.as_str(), "资源模板", "template_res_res_template", funs, ctx).await?;
    let _ = add_ele_res(set_id, template_res_res_cate_id.as_str(), "创建", "template*res*res*create", funs, ctx).await?;
    let _ = add_ele_res(set_id, template_res_res_cate_id.as_str(), "编辑", "template*res*res*update", funs, ctx).await?;
    let _ = add_ele_res(set_id, template_res_res_cate_id.as_str(), "页面配置", "template*res*res*page", funs, ctx).await?;
    let _ = add_ele_res(set_id, template_res_res_cate_id.as_str(), "删除", "template*res*res*delete", funs, ctx).await?;

    // configuration -> template and tag -> template -> notice
    let template_notice_cate_id = add_cate_menu(set_id, template_cate_id.as_str(), "通知", "__template_notice__", &IamSetCateKind::Tenant, funs, ctx).await?;

    // configuration -> template and tag -> template -> notice -> notice template
    let template_notice_notice_cate_id = add_cate_menu(
        set_id,
        template_notice_cate_id.as_str(),
        "通知模板",
        "__template_notice_notice_template__",
        &IamSetCateKind::Tenant,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(set_id, template_notice_notice_cate_id.as_str(), "通知模板", "template_notice_notice_template", funs, ctx).await?;
    let _ = add_ele_res(set_id, template_notice_notice_cate_id.as_str(), "创建", "template*notice*notice*create", funs, ctx).await?;
    let _ = add_ele_res(set_id, template_notice_notice_cate_id.as_str(), "编辑", "template*notice*notice*update", funs, ctx).await?;
    let _ = add_ele_res(set_id, template_notice_notice_cate_id.as_str(), "删除", "template*notice*notice*delete", funs, ctx).await?;

    // configuration -> template and tag -> tag
    let tag_cate_id = add_cate_menu(set_id, template_and_tag_cate_id.as_str(), "标签", "__tag__", &IamSetCateKind::Tenant, funs, ctx).await?;
    let _ = add_menu_res(set_id, tag_cate_id.as_str(), "标签", "tag", funs, ctx).await?;
    let _ = add_ele_res(set_id, tag_cate_id.as_str(), "创建", "ttag*create", funs, ctx).await?;
    let _ = add_ele_res(set_id, tag_cate_id.as_str(), "编辑", "tag*update", funs, ctx).await?;
    let _ = add_ele_res(set_id, tag_cate_id.as_str(), "删除", "tag*delete", funs, ctx).await?;

    Ok(())
}

async fn init_menu_configuration_notice<'a>(set_id: &str, parent_cate_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    // configuration -> notice
    let notice_cate_id = add_cate_menu(set_id, parent_cate_id, "通知", "__notice__", &IamSetCateKind::Tenant, funs, ctx).await?;

    // configuration -> notice -> sign
    let sing_cate_id = add_cate_menu(set_id, notice_cate_id.as_str(), "签名管理", "__notice_sign__", &IamSetCateKind::Tenant, funs, ctx).await?;
    let _ = add_menu_res(set_id, sing_cate_id.as_str(), "签名管理", "sign", funs, ctx).await?;
    let _ = add_ele_res(set_id, sing_cate_id.as_str(), "创建", "sign*create", funs, ctx).await?;
    let _ = add_ele_res(set_id, sing_cate_id.as_str(), "编辑", "sign*update", funs, ctx).await?;
    let _ = add_ele_res(set_id, sing_cate_id.as_str(), "删除", "sign*delete", funs, ctx).await?;

    // configuration -> notice -> config
    let notice_config_cate_id = add_cate_menu(set_id, notice_cate_id.as_str(), "通知配置", "__notice_config__", &IamSetCateKind::Tenant, funs, ctx).await?;
    let _ = add_menu_res(set_id, notice_config_cate_id.as_str(), "通知配置", "notice*config", funs, ctx).await?;
    let _ = add_ele_res(set_id, notice_config_cate_id.as_str(), "编辑", "notice*config*update", funs, ctx).await?;

    // configuration -> notice -> record
    let notice_record_cate_id = add_cate_menu(set_id, notice_cate_id.as_str(), "通知记录", "__notice_record__", &IamSetCateKind::Tenant, funs, ctx).await?;
    let _ = add_menu_res(set_id, notice_record_cate_id.as_str(), "通知记录", "notice*record", funs, ctx).await?;

    // configuration -> notice -> record -> detail
    let notice_record_detail_cate_id = add_cate_menu(
        set_id,
        notice_record_cate_id.as_str(),
        "详情页面",
        "__notice_record_detail__",
        &IamSetCateKind::Tenant,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(set_id, notice_record_detail_cate_id.as_str(), "详情页面", "notice*record*detail", funs, ctx).await?;

    Ok(())
}

async fn add_cate_menu<'a>(
    set_id: &str,
    parent_cate_menu_id: &str,
    name: &str,
    bus_code: &str,
    ext: &IamSetCateKind,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<String> {
    RbumSetCateServ::add_rbum(
        &mut RbumSetCateAddReq {
            name: TrimString(name.to_string()),
            bus_code: TrimString(bus_code.to_string()),
            icon: None,
            sort: None,
            ext: Some(ext.to_string()),
            rbum_parent_cate_id: Some(parent_cate_menu_id.to_string()),
            rel_rbum_set_id: set_id.to_string(),
            scope_level: Some(iam_constants::RBUM_SCOPE_LEVEL_GLOBAL),
        },
        funs,
        ctx,
    )
    .await
}

async fn add_menu_res<'a>(set_id: &str, cate_menu_id: &str, name: &str, code: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
    IamResServ::add_res_agg(
        &mut IamResAggAddReq {
            res: IamResAddReq {
                code: TrimString(code.to_string()),
                name: TrimString(name.to_string()),
                kind: IamResKind::Menu,
                icon: None,
                sort: None,
                method: None,
                hide: None,
                action: None,
                scope_level: Some(iam_constants::RBUM_SCOPE_LEVEL_GLOBAL),
                disabled: None,
            },
            set: IamSetItemAggAddReq {
                set_cate_id: cate_menu_id.to_string(),
            },
        },
        set_id,
        funs,
        ctx,
    )
    .await
}

async fn add_ele_res<'a>(set_id: &str, cate_menu_id: &str, name: &str, code: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
    IamResServ::add_res_agg(
        &mut IamResAggAddReq {
            res: IamResAddReq {
                code: TrimString(code.to_string()),
                name: TrimString(name.to_string()),
                kind: IamResKind::Ele,
                icon: None,
                sort: None,
                method: None,
                hide: None,
                action: None,
                scope_level: Some(iam_constants::RBUM_SCOPE_LEVEL_GLOBAL),
                disabled: None,
            },
            set: IamSetItemAggAddReq {
                set_cate_id: cate_menu_id.to_string(),
            },
        },
        set_id,
        funs,
        ctx,
    )
    .await
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
    funs.cache().flushdb().await?;
    Ok(())
}
