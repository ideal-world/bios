use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;

use bios_iam::console_app::dto::iam_ca_account_dto::{IamCaAccountAddReq, IamCaAccountModifyReq};
use bios_iam::console_app::dto::iam_ca_role_dto::IamCaRoleAddReq;
use bios_iam::console_app::serv::iam_ca_account_serv::IamCaAccountServ;
use bios_iam::console_app::serv::iam_ca_role_serv::IamCaRoleServ;
use bios_iam::iam_constants;
use bios_iam::iam_constants::RBUM_SCOPE_LEVEL_APP;

pub async fn test(context1: &TardisContext, context2: &TardisContext) -> TardisResult<()> {
    let mut funs = iam_constants::get_tardis_inst();
    funs.begin().await?;

    info!("【test_ca_account】 : Prepare : Add Role");
    let role_id1 = IamCaRoleServ::add_role(
        &mut IamCaRoleAddReq {
            name: TrimString("角色1".to_string()),
            icon: None,
            sort: None,
            disabled: None,
        },
        &funs,
        context1,
    )
    .await?;

    info!("【test_ca_account】 : Add Account");
    let account_id1 = IamCaAccountServ::add_account(
        &mut IamCaAccountAddReq {
            name: TrimString("星航1".to_string()),
            icon: None,
            scope_level: RBUM_SCOPE_LEVEL_APP,
            disabled: None,
        },
        &funs,
        context1,
    )
    .await?;

    IamCaAccountServ::add_account(
        &mut IamCaAccountAddReq {
            name: TrimString("星航2".to_string()),
            icon: None,
            scope_level: RBUM_SCOPE_LEVEL_APP,
            disabled: None,
        },
        &funs,
        context2,
    )
    .await?;

    info!("【test_ca_account】 : Modify Account By Id, with err");
    assert!(IamCaAccountServ::modify_account(
        &account_id1,
        &mut IamCaAccountModifyReq {
            name: Some(TrimString("星航3".to_string())),
            icon: Some("/icon/icon.png".to_string()),
            scope_level: None,
            disabled: None
        },
        &funs,
        context2
    )
    .await
    .is_err());
    info!("【test_ca_account】 : Modify Account By Id");
    IamCaAccountServ::modify_account(
        &account_id1,
        &mut IamCaAccountModifyReq {
            name: Some(TrimString("星航".to_string())),
            icon: Some("/icon/icon.png".to_string()),
            scope_level: None,
            disabled: None,
        },
        &funs,
        context1,
    )
    .await?;

    info!("【test_ca_account】 : Get Account By Id, with err");
    assert!(IamCaAccountServ::get_account(&account_id1, &funs, context2).await.is_err());
    info!("【test_ca_account】 : Get Account By Id");
    let account = IamCaAccountServ::get_account(&account_id1, &funs, context1).await?;
    assert_eq!(account.id, account_id1);
    assert_eq!(account.name, "星航");
    assert_eq!(account.icon, "/icon/icon.png");
    assert!(!account.disabled);

    info!("【test_ca_account】 : Find Accounts");
    let accounts = IamCaAccountServ::paginate_accounts(None, None, 1, 10, None, None, &funs, context1).await?;
    assert_eq!(accounts.page_number, 1);
    assert_eq!(accounts.page_size, 10);
    assert_eq!(accounts.total_size, 1);
    assert!(accounts.records.iter().any(|i| i.name == "星航"));

    info!("【test_ca_account】 : Find Rel Roles By Account Id");
    let account_roles = IamCaAccountServ::paginate_rel_roles(&context1.owner, 1, 10, None, None, &funs, context1).await?;
    assert_eq!(account_roles.page_number, 1);
    assert_eq!(account_roles.page_size, 10);
    assert_eq!(account_roles.total_size, 1);
    assert_eq!(account_roles.records.len(), 1);
    assert_eq!(account_roles.records.get(0).unwrap().rel.from_rbum_item_name, "应用1管理员");
    assert_eq!(account_roles.records.get(0).unwrap().rel.to_rbum_item_name, "app_admin");
    let account_roles = IamCaAccountServ::paginate_rel_roles(&account_id1, 1, 10, None, None, &funs, context1).await?;
    assert_eq!(account_roles.page_number, 1);
    assert_eq!(account_roles.page_size, 10);
    assert_eq!(account_roles.total_size, 0);
    assert!(account_roles.records.is_empty());
    info!("【test_ca_account】 : Add Rel Account By Id");
    IamCaRoleServ::add_rel_account(&role_id1, &account_id1, None, None, &funs, context1).await?;
    let account_roles = IamCaAccountServ::paginate_rel_roles(&account_id1, 1, 10, None, None, &funs, context1).await?;
    assert_eq!(account_roles.page_number, 1);
    assert_eq!(account_roles.page_size, 10);
    assert_eq!(account_roles.total_size, 1);

    info!("【test_ca_account】 : Delete Account By Id, with err");
    assert!(IamCaAccountServ::delete_account("11111", &funs, context1).await.is_err());
    info!("【test_ca_account】 : Delete Account By Id, with err");
    assert!(IamCaAccountServ::delete_account(&account_id1, &funs, context2).await.is_err());
    info!("【test_ca_account】 : Delete Account By Id");
    assert_eq!(
        IamCaAccountServ::paginate_accounts(Some(account_id1.clone()), None, 1, 10, None, None, &funs, context1).await?.total_size,
        1
    );
    IamCaAccountServ::delete_account(&account_id1, &funs, context1).await?;
    assert_eq!(
        IamCaAccountServ::paginate_accounts(Some(account_id1.clone()), None, 1, 10, None, None, &funs, context1).await?.total_size,
        0
    );

    funs.rollback().await?;

    Ok(())
}
