use std::default::default;

use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemFilterReq};
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_iam::basic::dto::iam_filer_dto::IamTenantFilterReq;
use bios_iam::basic::dto::iam_tenant_dto::IamTenantModifyReq;
use bios_iam::basic::serv::iam_tenant_serv::IamTenantServ;
use bios_iam::console_system::dto::iam_cs_tenant_dto::{IamCsTenantAddReq, IamCsTenantModifyReq};
use bios_iam::console_system::serv::iam_cs_tenant_serv::IamCsTenantServ;

pub async fn test(context: &TardisContext) -> TardisResult<()> {
    let mut funs = TardisFunsInst::conn("");
    funs.begin().await?;

    info!("【test_cs_tenant】 : Test Add : IamCsTenantServ::add_tenant");
    let id1 = IamCsTenantServ::add_tenant(
        &mut IamCsTenantAddReq {
            tenant_name: TrimString("测试租户1".to_string()),
            tenant_icon: None,
            tenant_contact_phone: None,
            admin_username: TrimString("admin".to_string()),
            disabled: None,
            admin_name: TrimString("测试管理员".to_string()),
        },
        &funs,
        context,
    )
    .await?
    .0;
    IamCsTenantServ::add_tenant(
        &mut IamCsTenantAddReq {
            tenant_name: TrimString("测试租户2".to_string()),
            tenant_icon: None,
            tenant_contact_phone: Some("12345678901".to_string()),
            admin_username: TrimString("admin".to_string()),
            disabled: None,
            admin_name: TrimString("测试管理员".to_string()),
        },
        &funs,
        context,
    )
    .await?;

    let id2 = IamCsTenantServ::add_tenant(
        &mut IamCsTenantAddReq {
            tenant_name: TrimString("测试租户2".to_string()),
            tenant_icon: None,
            tenant_contact_phone: Some("12345678901".to_string()),
            admin_username: TrimString("admin1".to_string()),
            disabled: None,
            admin_name: TrimString("测试管理员".to_string()),
        },
        &funs,
        context,
    )
    .await?
    .0;

    info!("【test_cs_tenant】 : Test Get : IamTenantCrudServ::get_item");
    let tenant = IamTenantServ::get_item(&id1, &IamTenantFilterReq::default(), funs.db(), context).await?;
    assert_eq!(tenant.id, id1);
    assert_eq!(tenant.name, "测试租户1");
    assert_eq!(tenant.contact_phone, "");
    let tenant = IamTenantServ::get_item(&id2, &IamTenantFilterReq::default(), funs.db(), context).await?;
    assert_eq!(tenant.id, id2);
    assert_eq!(tenant.name, "测试租户2");
    assert_eq!(tenant.contact_phone, "12345678901");

    info!("【test_cs_tenant】 : Test Modify : IamCsTenantServ::modify_tenant");
    IamCsTenantServ::modify_tenant(&id1, &mut IamCsTenantModifyReq { disabled: Some(true) }, funs.db(), context).await?;

    info!("【test_cs_tenant】 : Test Modify : IamTenantCrudServ::modify_item");
    IamTenantServ::modify_item(
        &id2,
        &mut IamTenantModifyReq {
            name: None,
            icon: None,
            sort: None,
            contact_phone: Some("xxxx".to_string()),
            disabled: None,
            scope_level: None,
        },
        funs.db(),
        context,
    )
    .await?;

    info!("【test_cs_tenant】 : Test Find : IamTenantCrudServ::paginate_items");
    let tenants = IamTenantServ::paginate_items(
        &IamTenantFilterReq {
            basic: RbumBasicFilterReq {
                name: Some("测试租户%".to_string()),
                ..Default::default()
            },
            ..Default::default()
        },
        1,
        10,
        None,
        None,
        funs.db(),
        context,
    )
    .await?;
    assert_eq!(tenants.page_number, 1);
    assert_eq!(tenants.page_size, 10);
    assert!(tenants.records.iter().any(|r| r.contact_phone == "xxxx"));
    assert!(tenants.records.iter().any(|r| r.disabled));

    info!("【test_cs_tenant】 : Test Delete : IamTenantCrudServ::delete_item");
    assert!(IamTenantServ::delete_item(&id1, funs.db(), context).await.is_err());

    funs.rollback().await?;

    Ok(())
}
