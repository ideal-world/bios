use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;

use bios_iam::console_system::dto::iam_cs_account_dto::IamCsAccountModifyReq;
use bios_iam::console_system::dto::iam_cs_tenant_dto::IamCsTenantAddReq;
use bios_iam::console_system::serv::iam_cs_account_serv::IamCsAccountServ;
use bios_iam::console_system::serv::iam_cs_tenant_serv::IamCsTenantServ;
use bios_iam::iam_constants;

pub async fn test(context: &TardisContext) -> TardisResult<()> {
    let mut funs = iam_constants::get_tardis_inst();
    funs.begin().await?;

    info!("【test_cs_account】 : Prepare Kind : IamCsTenantServ::add_tenant");
    let (tenant_id, _) = IamCsTenantServ::add_tenant(
        &mut IamCsTenantAddReq {
            tenant_name: TrimString("测试租户1".to_string()),
            tenant_icon: None,
            tenant_contact_phone: None,
            admin_username: TrimString("bios".to_string()),
            disabled: None,
            admin_name: TrimString("测试管理员".to_string()),
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_cs_account】 : Modify Account By Id");
    assert!(IamCsAccountServ::modify_account("1111111", &mut IamCsAccountModifyReq { disabled: Some(true) }, &funs, context).await.is_err());
    IamCsAccountServ::modify_account(&context.owner, &mut IamCsAccountModifyReq { disabled: Some(true) }, &funs, &context).await?;

    info!("【test_cs_account】 : Get Account By Id");
    let account = IamCsAccountServ::get_account(&context.owner, &funs, context).await?;
    assert_eq!(account.id, context.owner);
    assert_eq!(account.name, "bios");
    assert!(account.disabled);

    info!("【test_cs_account】 : Find Accounts");
    let accounts = IamCsAccountServ::paginate_accounts(tenant_id.clone(), None, None, 1, 10, None, None, &funs, context).await?;
    assert_eq!(accounts.page_number, 1);
    assert_eq!(accounts.page_size, 10);
    assert_eq!(accounts.total_size, 1);
    assert!(accounts.records.iter().any(|i| i.name == "测试管理员"));

    funs.rollback().await?;

    Ok(())
}
