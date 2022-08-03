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
use crate::console_common::api::{iam_cc_account_api, iam_cc_org_api, iam_cc_role_api};
use crate::console_passport::api::{iam_cp_account_api, iam_cp_cert_api, iam_cp_tenant_api};
use crate::console_system::api::{iam_cs_account_api, iam_cs_account_attr_api, iam_cs_cert_api, iam_cs_res_api, iam_cs_role_api, iam_cs_tenant_api};
use crate::console_tenant::api::{
    iam_ct_account_api, iam_ct_account_attr_api, iam_ct_app_api, iam_ct_app_set_api, iam_ct_cert_api, iam_ct_org_api, iam_ct_res_api, iam_ct_role_api, iam_ct_tenant_api,
};
use crate::iam_config::{BasicInfo, IamBasicInfoManager, IamConfig};
use crate::iam_constants;
use crate::iam_constants::RBUM_SCOPE_LEVEL_GLOBAL;
use crate::iam_enumeration::{IamResKind, IamSetKind};

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
                (iam_cc_account_api::IamCcAccountApi, iam_cc_role_api::IamCcRoleApi, iam_cc_org_api::IamCcOrgApi),
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

pub async fn init_db(mut funs: TardisFunsInst<'_>) -> TardisResult<Option<(String, String)>> {
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

async fn init_basic_info<'a>(funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<()> {
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

pub async fn init_rbum_data(funs: &TardisFunsInst<'_>) -> TardisResult<(String, String)> {
    let default_account_id = TardisFuns::field.nanoid();

    let ctx = TardisContext {
        own_paths: "".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: default_account_id.clone(),
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
            exts: Default::default(),
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

async fn add_kind<'a>(scheme: &str, ext_table: &str, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<String> {
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

async fn add_domain<'a>(funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<String> {
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

async fn add_res<'a>(
    set_res_id: &str,
    cate_menu_id: &str,
    cate_api_id: &str,
    code: &str,
    name: &str,
    funs: &TardisFunsInst<'a>,
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

pub async fn truncate_data<'a>(funs: &TardisFunsInst<'a>) -> TardisResult<()> {
    rbum_initializer::truncate_data(funs).await?;
    funs.db().execute(Table::truncate().table(iam_account::Entity)).await?;
    funs.db().execute(Table::truncate().table(iam_app::Entity)).await?;
    funs.db().execute(Table::truncate().table(iam_res::Entity)).await?;
    funs.db().execute(Table::truncate().table(iam_role::Entity)).await?;
    funs.db().execute(Table::truncate().table(iam_tenant::Entity)).await?;
    funs.cache().flushdb().await?;
    Ok(())
}
