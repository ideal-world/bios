use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_iam::basic::dto::iam_tenant_dto::IamTenantAggModifyReq;
use bios_iam::basic::dto::iam_account_dto::IamAccountAddReq;
use bios_iam::basic::dto::iam_role_dto::IamRoleAddReq;
use bios_iam::basic::serv::iam_cert_oauth2_serv::IamCertOAuth2Serv;
use bios_iam::basic::serv::iam_account_serv::IamAccountServ;
use bios_iam::basic::serv::iam_app_serv::IamAppServ;
use bios_iam::basic::serv::iam_cert_oauth2_service_serv::IamCertOAuth2ServiceServ;
use bios_iam::basic::serv::iam_role_serv::IamRoleServ;
use bios_iam::basic::serv::iam_tenant_serv::IamTenantServ;
use bios_iam::iam_constants;
use bios_iam::iam_config::IamBasicConfigApi;
use bios_iam::iam_enumeration::{IamCertOAuth2Supplier, IamRoleKind};
use ldap3::log::info;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;

pub async fn test(tenant1_admin_context: &TardisContext) -> TardisResult<()> {
    const GITHUB_OAUTH2_AK: &str = "";
    const GITHUB_OAUTH2_SK: &str = "";
    // Manually splicing address to obtain code
    // https://github.com/login/oauth/authorize?client_id={GITHUB_OAUTH2_AK}&redirect_uri=http://127.0.0.1/
    let code = "";

    let mut funs = iam_constants::get_tardis_inst();
    funs.begin().await?;
    let id = &IamTenantServ::get_id_by_ctx(tenant1_admin_context, &funs)?;
    IamTenantServ::modify_tenant_agg(
        id,
        &IamTenantAggModifyReq {
            name: None,
            icon: None,
            sort: None,
            contact_phone: None,
            note: None,
            account_self_reg: None,
            disabled: None,
        },
        &funs,
        tenant1_admin_context,
    )
    .await?;

    let account = IamCertOAuth2Serv::get_or_add_account(IamCertOAuth2Supplier::Github, code, id, &funs).await?;
    info!("account info= {:?}", account);
    Ok(())
}

fn account_add_req(name: &str, disabled: bool) -> IamAccountAddReq {
    IamAccountAddReq {
        id: None,
        name: name.into(),
        scope_level: Some(iam_constants::RBUM_SCOPE_LEVEL_APP),
        disabled: Some(disabled),
        logout_type: None,
        labor_type: None,
        id_card_no: None,
        employee_code: None,
        others_id: None,
        temporary: None,
        lock_status: None,
        status: None,
        icon: None,
    }
}

pub async fn test_role_members(app1_context: &TardisContext, app2_context: &TardisContext) -> TardisResult<()> {
    let mut funs = iam_constants::get_tardis_inst();
    funs.begin().await?;

    let app1_id = IamAppServ::get_id_by_ctx(app1_context, &funs)?;
    let app_admin_role_id = IamRoleServ::get_embed_sub_role_id(&funs.iam_basic_role_app_admin_id(), &funs, app1_context).await?;

    let enabled_account_id = IamAccountServ::add_item(&mut account_add_req("oauth-role-enabled", false), &funs, app1_context).await?;
    IamAppServ::add_rel_account(&app1_id, &enabled_account_id, true, &funs, app1_context).await?;
    IamRoleServ::add_rel_account(&app_admin_role_id, &enabled_account_id, None, &funs, app1_context).await?;

    let disabled_account_id = IamAccountServ::add_item(&mut account_add_req("oauth-role-disabled", true), &funs, app1_context).await?;
    IamAppServ::add_rel_account(&app1_id, &disabled_account_id, true, &funs, app1_context).await?;
    IamRoleServ::add_rel_account(&app_admin_role_id, &disabled_account_id, None, &funs, app1_context).await?;

    let custom_role_id = IamRoleServ::add_item(
        &mut IamRoleAddReq {
            id: None,
            code: Some("oauth_custom".into()),
            name: "OAuth custom role".into(),
            icon: None,
            sort: None,
            disabled: None,
            scope_level: Some(iam_constants::RBUM_SCOPE_LEVEL_PRIVATE),
            kind: Some(IamRoleKind::App),
            extend_role_id: None,
            in_embed: Some(false),
            in_base: Some(false),
            deletable: Some(true),
        },
        &funs,
        app1_context,
    )
    .await?;
    IamRoleServ::add_rel_account(&custom_role_id, &enabled_account_id, None, &funs, app1_context).await?;

    let first_page = IamCertOAuth2ServiceServ::find_role_members(&app1_id, "app_admin", 1, 1, app1_context, &funs).await?;
    let second_page = IamCertOAuth2ServiceServ::find_role_members(&app1_id, "app_admin", 2, 1, app1_context, &funs).await?;
    assert_eq!(first_page.total_size, 2);
    assert_eq!(second_page.total_size, 2);
    let returned_ids = first_page.records.into_iter().chain(second_page.records.into_iter()).map(|record| record.id).collect::<Vec<_>>();
    assert!(returned_ids.contains(&app1_context.owner));
    assert!(returned_ids.contains(&enabled_account_id));
    assert!(!returned_ids.contains(&disabled_account_id));

    let custom_page = IamCertOAuth2ServiceServ::find_role_members(&app1_id, "oauth_custom", 1, 10, app1_context, &funs).await?;
    assert_eq!(custom_page.total_size, 1);
    assert_eq!(custom_page.records[0].id, enabled_account_id);

    assert!(IamCertOAuth2ServiceServ::find_role_members(&app1_id, " ", 1, 10, app1_context, &funs).await.is_err());
    assert!(IamCertOAuth2ServiceServ::find_role_members(&app1_id, "app_admin", 1, 10, app2_context, &funs).await.is_err());

    funs.rollback().await?;
    Ok(())
}
