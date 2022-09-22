use bios_basic::rbum::dto::rbum_set_cate_dto::RbumSetCateAddReq;
use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use bios_basic::rbum::serv::rbum_set_serv::RbumSetCateServ;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm::sea_query::Table;
use tardis::log::info;
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
use crate::basic::dto::iam_cert_conf_dto::{IamMailVCodeCertConfAddOrModifyReq, IamPhoneVCodeCertConfAddOrModifyReq, IamUserPwdCertConfAddOrModifyReq};
use crate::basic::dto::iam_res_dto::{IamResAddReq, IamResAggAddReq};
use crate::basic::dto::iam_role_dto::{IamRoleAddReq, IamRoleAggAddReq};
use crate::basic::dto::iam_set_dto::IamSetItemAggAddReq;
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_res_serv::IamResServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::console_app::api::{iam_ca_account_api, iam_ca_app_api, iam_ca_res_api, iam_ca_role_api};
use crate::console_common::api::{iam_cc_account_api, iam_cc_org_api, iam_cc_res_api, iam_cc_role_api, iam_cc_system_api};
use crate::console_passport::api::{iam_cp_account_api, iam_cp_cert_api, iam_cp_tenant_api};
use crate::console_system::api::{iam_cs_account_api, iam_cs_account_attr_api, iam_cs_cert_api, iam_cs_res_api, iam_cs_role_api, iam_cs_tenant_api};
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
            (
                (
                    iam_cc_account_api::IamCcAccountApi,
                    iam_cc_role_api::IamCcRoleApi,
                    iam_cc_org_api::IamCcOrgApi,
                    iam_cc_res_api::IamCcResApi,
                    iam_cc_system_api::IamCcSystemApi,
                ),
                (iam_cp_account_api::IamCpAccountApi, iam_cp_cert_api::IamCpCertApi, iam_cp_tenant_api::IamCpTenantApi),
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
        .find(|r| r.name == iam_constants::RBUM_ITEM_NAME_SYS_ADMIN_ROLE)
        .map(|r| r.id.clone())
        .ok_or_else(|| funs.err().not_found("iam", "init", "not found sys admin role", ""))?;

    let role_tenant_admin_id = roles
        .iter()
        .find(|r| r.name == iam_constants::RBUM_ITEM_NAME_TENANT_ADMIN_ROLE)
        .map(|r| r.id.clone())
        .ok_or_else(|| funs.err().not_found("iam", "init", "not found tenant admin role", ""))?;

    let role_app_admin_id = roles
        .iter()
        .find(|r| r.name == iam_constants::RBUM_ITEM_NAME_APP_ADMIN_ROLE)
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

    init_menu(&set_res_id, &cate_menu_id, funs, &ctx).await?;

    // Init kernel certs
    IamCertServ::init_default_ident_conf(
        IamUserPwdCertConfAddOrModifyReq {
            // TODO config
            ak_note: None,
            ak_rule: None,
            sk_note: None,
            sk_rule: None,
            ext: None,
            repeatable: Some(true),
            expire_sec: None,
            sk_lock_cycle_sec: None,
            sk_lock_err_times: None,
            sk_lock_duration_sec: None,
        },
        Some(IamPhoneVCodeCertConfAddOrModifyReq { ak_note: None, ak_rule: None }),
        Some(IamMailVCodeCertConfAddOrModifyReq { ak_note: None, ak_rule: None }),
        funs,
        &ctx,
    )
    .await?;

    // Init ext certs
    IamCertServ::init_default_ext_conf(funs, &ctx).await?;

    // Init manage certs
    IamCertServ::init_default_manage_conf(funs, &ctx).await?;

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
            res_ids: Some(vec![set_menu_ca_id, set_api_ca_id]),
        },
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

#[deprecated]
async fn init_menu<'a>(set_id: &str, parent_cate_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    init_menu_workbench(set_id, parent_cate_id, funs, ctx).await?;
    init_menu_account_info(set_id, parent_cate_id, funs, ctx).await?;
    init_menu_knoledge(set_id, parent_cate_id, funs, ctx).await?;
    init_menu_configuration(set_id, parent_cate_id, funs, ctx).await?;

    Ok(())
}

async fn init_menu_workbench<'a>(set_id: &str, parent_cate_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    // workbench
    let workbench_cate_id = add_cate_menu(set_id, parent_cate_id, "工作台", "__workbench__", &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, ctx).await?;
    let _ = add_menu_res(
        set_id,
        workbench_cate_id.as_str(),
        "工作台",
        "workbench",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    Ok(())
}

async fn init_menu_account_info<'a>(set_id: &str, parent_cate_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    // account_info
    let personal_center_cate_id = add_cate_menu(
        set_id,
        parent_cate_id,
        "个人中心",
        "__personal_center__",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let account_info_cate_id = add_cate_menu(
        set_id,
        personal_center_cate_id.as_str(),
        "账号信息",
        "__account_info__",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(
        set_id,
        account_info_cate_id.as_str(),
        "账号信息",
        "account_info",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    Ok(())
}

async fn init_menu_knoledge<'a>(set_id: &str, parent_cate_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    // knowledge
    let collaboration_cate_id = add_cate_menu(set_id, parent_cate_id, "协作空间", "__collaboration__", &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, ctx).await?;
    let knowledge_cate_id = add_cate_menu(
        set_id,
        collaboration_cate_id.as_str(),
        "知识库",
        "__knowledge__",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(
        set_id,
        knowledge_cate_id.as_str(),
        "知识库",
        "knowledge",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_ele_res(
        set_id,
        knowledge_cate_id.as_str(),
        "创建",
        "knowledge*list*create",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    Ok(())
}

async fn init_menu_configuration<'a>(set_id: &str, parent_cate_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    // configuration
    let configuration_cate_id = add_cate_menu(set_id, parent_cate_id, "配置", "__configuration__", &iam_constants::RBUM_SCOPE_LEVEL_TENANT, funs, ctx).await?;
    init_menu_configuration_user_and_org(set_id, configuration_cate_id.as_str(), funs, ctx).await?;
    init_menu_configuration_menu_and_role(set_id, configuration_cate_id.as_str(), funs, ctx).await?;
    init_menu_configuration_res_and_cert(set_id, configuration_cate_id.as_str(), funs, ctx).await?;
    init_menu_configuration_template_and_tag(set_id, configuration_cate_id.as_str(), funs, ctx).await?;
    // init_menu_configuration_notice(set_id, configuration_cate_id.as_str(), funs, ctx).await?;

    Ok(())
}

async fn init_menu_configuration_user_and_org<'a>(set_id: &str, parent_cate_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    // configuration -> user and org
    let user_and_org_cate_id = add_cate_menu(set_id, parent_cate_id, "用户与组织", "__user_and_org__", &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, ctx).await?;

    // configuration -> user and org -> tenant
    let tenant_cate_id = add_cate_menu(
        set_id,
        user_and_org_cate_id.as_str(),
        "租户管理",
        "__tenant__",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(set_id, tenant_cate_id.as_str(), "租户管理", "tenant", &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, ctx).await?;
    let _ = add_ele_res(set_id, tenant_cate_id.as_str(), "创建", "tenant*create", &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, ctx).await?;
    let _ = add_ele_res(set_id, tenant_cate_id.as_str(), "编辑", "tenant*update", &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, ctx).await?;
    let _ = add_ele_res(set_id, tenant_cate_id.as_str(), "启用", "tenant*enable", &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, ctx).await?;
    let _ = add_ele_res(
        set_id,
        tenant_cate_id.as_str(),
        "禁用",
        "tenant*disable",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_ele_res(
        set_id,
        tenant_cate_id.as_str(),
        "人员",
        "tenant*personal",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_ele_res(set_id, tenant_cate_id.as_str(), "删除", "tenant*delete", &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, ctx).await?;

    // configuration -> user and org -> tenant -> tenant create page
    let tenant_create_page_cate_id = add_cate_menu(
        set_id,
        tenant_cate_id.as_str(),
        "创建页面",
        "__tenant_create_page__",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(
        set_id,
        tenant_create_page_cate_id.as_str(),
        "创建页面",
        "tenant_create_page",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;

    // configuration -> user and org -> tenant -> tenant detail page
    let tenant_detail_page_id = add_cate_menu(
        set_id,
        tenant_cate_id.as_str(),
        "详情页面",
        "__tenant_detail_page__",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(
        set_id,
        tenant_detail_page_id.as_str(),
        "详情页面",
        "tenant_detail_page",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;

    // configuration -> user and org -> tenant -> tenant update page
    let tenant_update_page_id = add_cate_menu(
        set_id,
        tenant_cate_id.as_str(),
        "编辑页面",
        "__tenant_update_page__",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(
        set_id,
        tenant_update_page_id.as_str(),
        "编辑页面",
        "tenant_update_page",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;

    // configuration -> user and org -> tenant -> tenant personal page
    let tenant_personal_page_id = add_cate_menu(
        set_id,
        tenant_cate_id.as_str(),
        "人员页面",
        "__tenant_personal_page__",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(
        set_id,
        tenant_personal_page_id.as_str(),
        "人员页面",
        "tenant_personal_page",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_ele_res(
        set_id,
        tenant_personal_page_id.as_str(),
        "重置密码",
        "tenant*personal*resetPwd",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;

    // configuration -> user and org -> tenant info
    let tenant_info_cate_id = add_cate_menu(
        set_id,
        user_and_org_cate_id.as_str(),
        "租户信息",
        "__tenant_info__",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(
        set_id,
        tenant_info_cate_id.as_str(),
        "租户信息",
        "tenant_info",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_ele_res(
        set_id,
        tenant_info_cate_id.as_str(),
        "编辑",
        "tenant_info*update",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;

    // configuration -> user and org -> org
    let org_cate_id = add_cate_menu(
        set_id,
        user_and_org_cate_id.as_str(),
        "组织架构",
        "__org__",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(set_id, org_cate_id.as_str(), "组织架构", "org", &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, ctx).await?;
    let _ = add_ele_res(set_id, org_cate_id.as_str(), "创建", "org*create", &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, ctx).await?;
    let _ = add_ele_res(set_id, org_cate_id.as_str(), "编辑", "org*update", &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, ctx).await?;
    let _ = add_ele_res(set_id, org_cate_id.as_str(), "删除", "org*delete", &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, ctx).await?;
    let _ = add_ele_res(
        set_id,
        org_cate_id.as_str(),
        "添加账号",
        "org*add*account",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_ele_res(
        set_id,
        org_cate_id.as_str(),
        "移除账号",
        "org*delete*account",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;

    // configuration -> user and org -> personnel management
    let personnel_management_tenant_cate_id = add_cate_menu(
        set_id,
        user_and_org_cate_id.as_str(),
        "人员管理",
        "__personnel_management__",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(
        set_id,
        personnel_management_tenant_cate_id.as_str(),
        "人员管理",
        "personnel_management",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_ele_res(
        set_id,
        personnel_management_tenant_cate_id.as_str(),
        "创建",
        "personnel_management*create",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_ele_res(
        set_id,
        personnel_management_tenant_cate_id.as_str(),
        "编辑",
        "personnel_management*update",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_ele_res(
        set_id,
        personnel_management_tenant_cate_id.as_str(),
        "删除",
        "personnel_management*delete",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_ele_res(
        set_id,
        personnel_management_tenant_cate_id.as_str(),
        "重置密码",
        "personnel_management*resetPwd",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;

    // configuration -> user and org -> personnel management -> create page
    let personnel_management_create_page_cate_id = add_cate_menu(
        set_id,
        personnel_management_tenant_cate_id.as_str(),
        "创建页面",
        "__personnel_management_create_page__",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(
        set_id,
        personnel_management_create_page_cate_id.as_str(),
        "创建页面",
        "personnel_management_create_page",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
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
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(
        set_id,
        personnel_management_detail_page_id.as_str(),
        "详情页面",
        "personnel_management_detail_page",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
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
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(
        set_id,
        personnel_management_update_page_id.as_str(),
        "编辑页面",
        "personnel_management_update_page",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;

    // configuration -> user and org -> apps
    let apps_cate_id = add_cate_menu(
        set_id,
        user_and_org_cate_id.as_str(),
        "项目组",
        "__apps__",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(set_id, apps_cate_id.as_str(), "项目组", "apps", &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, ctx).await?;
    let _ = add_ele_res(set_id, apps_cate_id.as_str(), "创建", "apps*create", &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, ctx).await?;
    let _ = add_ele_res(set_id, apps_cate_id.as_str(), "重命名", "apps*rename", &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, ctx).await?;
    let _ = add_ele_res(set_id, apps_cate_id.as_str(), "删除", "apps*delete", &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, ctx).await?;
    let _ = add_ele_res(
        set_id,
        apps_cate_id.as_str(),
        "添加项目",
        "apps*add*app",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_ele_res(
        set_id,
        apps_cate_id.as_str(),
        "移除项目",
        "apps*delete*app",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_ele_res(
        set_id,
        apps_cate_id.as_str(),
        "添加人员",
        "apps*add*account",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_ele_res(
        set_id,
        apps_cate_id.as_str(),
        "移除人员",
        "apps*delete*account",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    Ok(())
}

async fn init_menu_configuration_menu_and_role<'a>(set_id: &str, parent_cate_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    // configuration -> menu and role
    let menu_and_role_cate_id = add_cate_menu(
        set_id,
        parent_cate_id,
        "菜单与权限",
        "__menu_and_role__",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;

    // configuration -> menu and role -> menu
    let menu_cate_id = add_cate_menu(
        set_id,
        menu_and_role_cate_id.as_str(),
        "菜单管理",
        "__menu__",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(set_id, menu_cate_id.as_str(), "菜单管理", "menu", &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, ctx).await?;
    let _ = add_ele_res(set_id, menu_cate_id.as_str(), "创建", "menu*create", &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, ctx).await?;
    let _ = add_ele_res(set_id, menu_cate_id.as_str(), "编辑", "menu*update", &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, ctx).await?;
    let _ = add_ele_res(set_id, menu_cate_id.as_str(), "删除", "menu*delete", &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, ctx).await?;
    let _ = add_ele_res(
        set_id,
        menu_cate_id.as_str(),
        "添加授权API",
        "menu*add*api",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_ele_res(
        set_id,
        menu_cate_id.as_str(),
        "移除授权API",
        "menu*delete*api",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_ele_res(
        set_id,
        menu_cate_id.as_str(),
        "添加按钮",
        "menu*add*ele",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_ele_res(
        set_id,
        menu_cate_id.as_str(),
        "编辑按钮",
        "menu*update*ele",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_ele_res(
        set_id,
        menu_cate_id.as_str(),
        "删除按钮",
        "menu*delete*ele",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;

    // configuration -> menu and role -> api
    let api_cate_id = add_cate_menu(
        set_id,
        menu_and_role_cate_id.as_str(),
        "Api管理",
        "__api__",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(set_id, api_cate_id.as_str(), "Api管理", "api", &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, ctx).await?;
    let _ = add_ele_res(set_id, api_cate_id.as_str(), "创建", "api*create", &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, ctx).await?;
    let _ = add_ele_res(set_id, api_cate_id.as_str(), "编辑", "api*update", &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, ctx).await?;
    let _ = add_ele_res(set_id, api_cate_id.as_str(), "删除", "api*delete", &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, ctx).await?;

    // configuration -> menu and role -> api -> create page
    let api_create_page_cate_id = add_cate_menu(
        set_id,
        api_cate_id.as_str(),
        "创建页面",
        "__api_create_page__",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(
        set_id,
        api_create_page_cate_id.as_str(),
        "创建页面",
        "api_create_page",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;

    // configuration -> menu and role -> api -> update page
    let api_update_page_cate_id = add_cate_menu(
        set_id,
        api_cate_id.as_str(),
        "编辑页面",
        "__api_update_page__",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(
        set_id,
        api_update_page_cate_id.as_str(),
        "编辑页面",
        "api_update_page",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;

    // configuration -> menu and role -> api -> detail page
    let api_detail_page_id = add_cate_menu(
        set_id,
        api_cate_id.as_str(),
        "详情页面",
        "__api_detail_page__",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(
        set_id,
        api_detail_page_id.as_str(),
        "详情页面",
        "api_detail_page",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;

    // configuration -> menu and role -> role
    let api_cate_id = add_cate_menu(
        set_id,
        menu_and_role_cate_id.as_str(),
        "角色管理",
        "__role__",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(set_id, api_cate_id.as_str(), "角色管理", "role", &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, ctx).await?;
    let _ = add_ele_res(set_id, api_cate_id.as_str(), "创建", "role*create", &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, ctx).await?;
    let _ = add_ele_res(set_id, api_cate_id.as_str(), "编辑", "role*update", &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, ctx).await?;
    let _ = add_ele_res(set_id, api_cate_id.as_str(), "删除", "role*delete", &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, ctx).await?;
    let _ = add_ele_res(
        set_id,
        api_cate_id.as_str(),
        "添加人员",
        "role*add*account",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_ele_res(
        set_id,
        api_cate_id.as_str(),
        "移除人员",
        "role*delete*account",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_ele_res(
        set_id,
        api_cate_id.as_str(),
        "查看权限",
        "role*list*permission",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    Ok(())
}

async fn init_menu_configuration_res_and_cert<'a>(set_id: &str, parent_cate_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    // configuration -> res and cert
    let res_and_cert_cate_id = add_cate_menu(set_id, parent_cate_id, "资源与凭证", "__res_and_cert__", &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, ctx).await?;

    // configuration -> res and cert -> res
    let res_cate_id = add_cate_menu(
        set_id,
        res_and_cert_cate_id.as_str(),
        "资源管理",
        "__res__",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(set_id, res_cate_id.as_str(), "资源管理", "res", &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, ctx).await?;
    let _ = add_ele_res(set_id, res_cate_id.as_str(), "创建", "res*create", &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, ctx).await?;
    let _ = add_ele_res(set_id, res_cate_id.as_str(), "编辑", "res*update", &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, ctx).await?;
    let _ = add_ele_res(
        set_id,
        res_cate_id.as_str(),
        "连接凭证",
        "res*link*cert",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_ele_res(set_id, res_cate_id.as_str(), "启用", "res*enable", &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, ctx).await?;
    let _ = add_ele_res(set_id, res_cate_id.as_str(), "禁用", "res*disabled", &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, ctx).await?;
    let _ = add_ele_res(set_id, res_cate_id.as_str(), "删除", "res*delete", &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, ctx).await?;

    // configuration -> res and cert -> res -> create page
    let res_create_page_cate_id = add_cate_menu(
        set_id,
        res_cate_id.as_str(),
        "创建资源页面",
        "__res_create_page__",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(
        set_id,
        res_create_page_cate_id.as_str(),
        "创建资源页面",
        "res_create_page",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;

    // configuration -> res and cert -> res -> update page
    let res_update_page_cate_id = add_cate_menu(
        set_id,
        res_cate_id.as_str(),
        "编辑资源页面",
        "__res_update_page__",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(
        set_id,
        res_update_page_cate_id.as_str(),
        "编辑资源页面",
        "res_update_page",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;

    // configuration -> res and cert -> res -> detail page
    let res_detail_page_id = add_cate_menu(
        set_id,
        res_cate_id.as_str(),
        "详情资源页面",
        "__res_detail_page__",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(
        set_id,
        res_detail_page_id.as_str(),
        "详情资源页面",
        "res_detail_page",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;

    // configuration -> res and cert -> res -> cert page
    let res_detail_page_id = add_cate_menu(
        set_id,
        res_cate_id.as_str(),
        "凭证页面",
        "__res_cert_page__",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(
        set_id,
        res_detail_page_id.as_str(),
        "凭证页面",
        "res_cret_page",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;

    // configuration -> res and cert -> res apply
    let res_apply_cate_id = add_cate_menu(
        set_id,
        res_and_cert_cate_id.as_str(),
        "申请管理",
        "__res_apply__",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(
        set_id,
        res_apply_cate_id.as_str(),
        "申请管理",
        "res_apply",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_ele_res(
        set_id,
        res_apply_cate_id.as_str(),
        "审批",
        "res_apply*approval",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_ele_res(
        set_id,
        res_cate_id.as_str(),
        "连接凭证",
        "res_apply*link*cert",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;

    // configuration -> res and cert -> res -> approval page
    let res_apply_approval_page_cate_id = add_cate_menu(
        set_id,
        res_cate_id.as_str(),
        "审批页面",
        "__res_apply_approval_page__",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(
        set_id,
        res_apply_approval_page_cate_id.as_str(),
        "审批页面",
        "res_apply_approval_page",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;

    // configuration -> res and cert -> res apply -> detail page
    let res_apply_detail_page_id = add_cate_menu(
        set_id,
        res_cate_id.as_str(),
        "详情页面",
        "__res_apply_detail_page__",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(
        set_id,
        res_apply_detail_page_id.as_str(),
        "详情页面",
        "res_apply_detail_page",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;

    // configuration -> res and cert -> res apply -> cert page
    let res_apply_detail_page_id = add_cate_menu(
        set_id,
        res_cate_id.as_str(),
        "凭证页面",
        "__res_apply_cert_page__",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(
        set_id,
        res_apply_detail_page_id.as_str(),
        "凭证页面",
        "res_apply_cret_page",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;

    // configuration -> res and cert -> cert
    let cert_cate_id = add_cate_menu(
        set_id,
        res_and_cert_cate_id.as_str(),
        "凭证管理",
        "__cert__",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(set_id, cert_cate_id.as_str(), "凭证管理", "cert", &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, ctx).await?;
    let _ = add_ele_res(set_id, cert_cate_id.as_str(), "创建", "cert*create", &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, ctx).await?;
    let _ = add_ele_res(set_id, cert_cate_id.as_str(), "编辑", "cert*update", &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, ctx).await?;
    let _ = add_ele_res(set_id, cert_cate_id.as_str(), "删除", "cert*delete", &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL, funs, ctx).await?;

    // configuration -> res and cert -> cert -> create page
    let cert_create_page_cate_id = add_cate_menu(
        set_id,
        cert_cate_id.as_str(),
        "创建页面",
        "__cert_create_page__",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(
        set_id,
        cert_create_page_cate_id.as_str(),
        "创建页面",
        "cert_create_page",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;

    // configuration -> res and cert -> cert -> detail page
    let cert_detail_page_id = add_cate_menu(
        set_id,
        cert_cate_id.as_str(),
        "详情页面",
        "__cert_detail_page__",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(
        set_id,
        cert_detail_page_id.as_str(),
        "详情页面",
        "cert_detail_page",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;

    // configuration -> res and cert -> cert -> update page
    let cert_detail_page_id = add_cate_menu(
        set_id,
        cert_cate_id.as_str(),
        "编辑页面",
        "__cert_update_page__",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    let _ = add_menu_res(
        set_id,
        cert_detail_page_id.as_str(),
        "编辑页面",
        "cert_update_page",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    Ok(())
}

async fn init_menu_configuration_template_and_tag<'a>(set_id: &str, parent_cate_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    // configuration -> template and tag
    let template_and_tag_cate_id = add_cate_menu(
        set_id,
        parent_cate_id,
        "模板与标签",
        "__template_and_tag__",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    // configuration -> template and tag -> template
    let template_cate_id = add_cate_menu(
        set_id,
        template_and_tag_cate_id.as_str(),
        "模板",
        "__template__",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;
    // configuration -> template and tag -> template -> app
    let _ = add_cate_menu(
        set_id,
        template_cate_id.as_str(),
        "项目",
        "__template_app__",
        &iam_constants::RBUM_SCOPE_LEVEL_GLOBAL,
        funs,
        ctx,
    )
    .await?;

    Ok(())
}

// async fn init_menu_configuration_notice<'a>(set_id: &str, parent_cate_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
//     Ok(())
// }

async fn add_cate_menu<'a>(
    set_id: &str,
    parent_cate_menu_id: &str,
    name: &str,
    bus_code: &str,
    scope_level: &RbumScopeLevelKind,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<String> {
    RbumSetCateServ::add_rbum(
        &mut RbumSetCateAddReq {
            name: TrimString(name.to_string()),
            bus_code: TrimString(bus_code.to_string()),
            icon: None,
            sort: None,
            ext: None,
            rbum_parent_cate_id: Some(parent_cate_menu_id.to_string()),
            rel_rbum_set_id: set_id.to_string(),
            scope_level: Some(scope_level.clone()),
        },
        funs,
        ctx,
    )
    .await
}

async fn add_menu_res<'a>(
    set_id: &str,
    cate_menu_id: &str,
    name: &str,
    code: &str,
    scope_level: &RbumScopeLevelKind,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<String> {
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
                scope_level: Some(scope_level.clone()),
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

async fn add_ele_res<'a>(
    set_id: &str,
    cate_menu_id: &str,
    name: &str,
    code: &str,
    scope_level: &RbumScopeLevelKind,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<String> {
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
                scope_level: Some(scope_level.clone()),
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
