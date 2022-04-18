use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;

use bios_iam::console_tenant::dto::iam_ct_account_dto::{IamCtAccountAddReq, IamCtAccountModifyReq};
use bios_iam::console_tenant::serv::iam_ct_account_serv::IamCtAccountServ;
use bios_iam::iam_constants;

pub async fn test(context1: &TardisContext, context2: &TardisContext) -> TardisResult<()> {
    let mut funs = iam_constants::get_tardis_inst();
    funs.begin().await?;

    info!("【test_ct_account】 : Add Account");
    let account_id1 = IamCtAccountServ::add_account(
        &mut IamCtAccountAddReq {
            name: TrimString("星航1".to_string()),
            icon: None,
            disabled: None,
        },
        &funs,
        context1,
    )
    .await?;

    IamCtAccountServ::add_account(
        &mut IamCtAccountAddReq {
            name: TrimString("星航2".to_string()),
            icon: None,
            disabled: None,
        },
        &funs,
        context2,
    )
    .await?;

    info!("【test_ct_account】 : Modify Account By Id, with err");
    assert!(IamCtAccountServ::modify_account(
        &account_id1,
        &mut IamCtAccountModifyReq {
            name: Some(TrimString("星航3".to_string())),
            icon: Some("/icon/icon.png".to_string()),
            disabled: None
        },
        &funs,
        context2
    )
    .await
    .is_err());
    info!("【test_ct_account】 : Modify Account By Id");
    IamCtAccountServ::modify_account(
        &account_id1,
        &mut IamCtAccountModifyReq {
            name: Some(TrimString("星航".to_string())),
            icon: Some("/icon/icon.png".to_string()),
            disabled: None,
        },
        &funs,
        context1,
    )
    .await?;

    info!("【test_ct_account】 : Get Account By Id, with err");
    assert!(IamCtAccountServ::get_account(&account_id1, &funs, context2).await.is_err());
    info!("【test_ct_account】 : Get Account By Id");
    let account = IamCtAccountServ::get_account(&account_id1, &funs, context1).await?;
    assert_eq!(account.id, account_id1);
    assert_eq!(account.name, "星航");
    assert_eq!(account.icon, "/icon/icon.png");
    assert!(!account.disabled);

    info!("【test_ct_account】 : Find Accounts");
    let accounts = IamCtAccountServ::paginate_accounts(None, None, 1, 10, None, None, &funs, context1).await?;
    assert_eq!(accounts.page_number, 1);
    assert_eq!(accounts.page_size, 10);
    assert_eq!(accounts.total_size, 2);
    assert!(accounts.records.iter().any(|i| i.name == "星航"));

    info!("【test_ct_account】 : Find Rel Roles By Account Id");
    let account_roles = IamCtAccountServ::paginate_rel_roles(&context1.owner, 1, 10, None, None, &funs, context1).await?;
    assert_eq!(account_roles.page_number, 1);
    assert_eq!(account_roles.page_size, 10);
    assert_eq!(account_roles.total_size, 1);
    assert_eq!(account_roles.records.len(), 1);
    assert_eq!(account_roles.records.get(0).unwrap().rel.from_rbum_item_name, "tenant_admin");
    assert_eq!(account_roles.records.get(0).unwrap().rel.to_rbum_item_name, "测试管理员");
    let account_roles = IamCtAccountServ::paginate_rel_roles(&account_id1, 1, 10, None, None, &funs, context1).await?;
    assert_eq!(account_roles.page_number, 1);
    assert_eq!(account_roles.page_size, 10);
    assert_eq!(account_roles.total_size, 1);
    assert_eq!(account_roles.records.len(), 1);
    assert!(account_roles.records.is_empty());

    info!("【test_ct_account】 : Delete Account By Id, with err");
    assert!(IamCtAccountServ::delete_account("11111", &funs, context1).await.is_err());
    info!("【test_ct_account】 : Delete Account By Id, with err");
    assert!(IamCtAccountServ::delete_account(&account_id1, &funs, context2).await.is_err());
    info!("【test_ct_account】 : Delete Account By Id");
    assert_eq!(
        IamCtAccountServ::paginate_accounts(Some(account_id1.clone()), None, 1, 10, None, None, &funs, context1).await?.total_size,
        1
    );
    IamCtAccountServ::delete_account(&account_id1, &funs, context1).await?;
    assert_eq!(
        IamCtAccountServ::paginate_accounts(Some(account_id1.clone()), None, 1, 10, None, None, &funs, context1).await?.total_size,
        0
    );

    funs.rollback().await?;

    Ok(())
}
