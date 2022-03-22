use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::TardisFuns;

use bios_basic::rbum::dto::filer_dto::RbumItemFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_iam::basic::dto::iam_tenant_dto::IamTenantModifyReq;
use bios_iam::basic::serv::iam_tenant_serv::IamTenantCrudServ;
use bios_iam::console_system::dto::iam_cs_tenant_dto::{IamCsTenantAddReq, IamCsTenantModifyReq};
use bios_iam::console_system::serv::iam_cs_tenant_serv::IamCsTenantServ;

pub async fn test(context: &TardisContext) -> TardisResult<()> {
    let mut tx = TardisFuns::reldb().conn();
    tx.begin().await?;

    info!("【test_cs_tenant】 : Test Add : IamCsTenantServ::add_tenant");
    let id1 = IamCsTenantServ::add_tenant(
        &mut IamCsTenantAddReq {
            name: TrimString("测试租户1".to_string()),
            icon: None,
            contact_phone: None,
            disabled: None,
        },
        &tx,
        context,
    )
    .await?;
    let id2 = IamCsTenantServ::add_tenant(
        &mut IamCsTenantAddReq {
            name: TrimString("测试租户2".to_string()),
            icon: None,
            contact_phone: Some("12345678901".to_string()),
            disabled: None,
        },
        &tx,
        context,
    )
    .await?;

    info!("【test_cs_tenant】 : Test Get : IamTenantCrudServ::get_item");
    let tenant = IamTenantCrudServ::get_item(&id1, &RbumItemFilterReq::default(), &tx, context).await?;
    assert_eq!(tenant.id, id1);
    assert_eq!(tenant.name, "测试租户1");
    assert_eq!(tenant.contact_phone, "");
    let tenant = IamTenantCrudServ::get_item(&id2, &RbumItemFilterReq::default(), &tx, context).await?;
    assert_eq!(tenant.id, id2);
    assert_eq!(tenant.name, "测试租户2");
    assert_eq!(tenant.contact_phone, "12345678901");

    info!("【test_cs_tenant】 : Test Modify : IamCsTenantServ::modify_tenant");
    IamCsTenantServ::modify_tenant(&id1, &mut IamCsTenantModifyReq { disabled: Some(true) }, &tx, context).await?;

    info!("【test_cs_tenant】 : Test Modify : IamTenantCrudServ::modify_item");
    IamTenantCrudServ::modify_item(
        &id2,
        &mut IamTenantModifyReq {
            name: None,
            icon: None,
            sort: None,
            contact_phone: Some("xxxx".to_string()),
            scope_kind: None,
            disabled: None,
        },
        &tx,
        context,
    )
    .await?;

    info!("【test_cs_tenant】 : Test Find : IamTenantCrudServ::paginate_items");
    let tenants = IamTenantCrudServ::paginate_items(
        &RbumItemFilterReq {
            name: Some("测试租户%".to_string()),
            ..Default::default()
        },
        1,
        10,
        None,
        None,
        &tx,
        context,
    )
    .await?;
    assert_eq!(tenants.page_number, 1);
    assert_eq!(tenants.page_size, 10);
    assert_eq!(tenants.total_size, 2);
    assert!(tenants.records.iter().any(|r| r.contact_phone == "xxxx"));
    assert!(tenants.records.iter().any(|r| r.disabled));

    info!("【test_cs_tenant】 : Test Delete : IamTenantCrudServ::delete_item");
    IamTenantCrudServ::delete_item(&id1, &tx, context).await?;
    assert!(IamTenantCrudServ::get_item(&id1, &RbumItemFilterReq::default(), &tx, context).await.is_err());

    tx.rollback().await?;

    Ok(())
}
