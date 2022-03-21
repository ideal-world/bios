use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::TardisFuns;

use bios_basic::rbum::dto::filer_dto::{RbumBasicFilterReq, RbumItemFilterReq};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_domain_serv::RbumDomainServ;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_iam::console_system::dto::iam_cs_tenant_dto::{IamCsTenantAddReq, IamCsTenantModifyReq};
use bios_iam::console_system::serv::iam_cs_tenant_serv::IamCsTenantServ;

pub async fn test() -> TardisResult<()> {
    let context = bios_basic::rbum::initializer::get_sys_admin_context().await?;
    let mut tx = TardisFuns::reldb().conn();
    tx.begin().await?;

    info!("Test IamCsTenantServ::add_item");
    let id1 = IamCsTenantServ::add_item(
        &mut IamCsTenantAddReq {
            name: TrimString("测试租户1".to_string()),
            icon: None,
            sort: None,
            contact_phone: None,
            disabled: None,
        },
        &tx,
        &context,
    )
    .await?;
    let id2 = IamCsTenantServ::add_item(
        &mut IamCsTenantAddReq {
            name: TrimString("测试租户2".to_string()),
            icon: None,
            sort: None,
            contact_phone: Some("12345678901".to_string()),
            disabled: None,
        },
        &tx,
        &context,
    )
    .await?;

    info!("Test IamCsTenantServ::get_item");
    let tenant = IamCsTenantServ::get_item(&id1, &RbumItemFilterReq::default(), &tx, &context).await?;
    assert_eq!(tenant.id, id1);
    assert_eq!(tenant.name, "测试租户1");
    assert_eq!(tenant.contact_phone, "");
    let tenant = IamCsTenantServ::get_item(&id2, &RbumItemFilterReq::default(), &tx, &context).await?;
    assert_eq!(tenant.id, id2);
    assert_eq!(tenant.name, "测试租户2");
    assert_eq!(tenant.contact_phone, "12345678901");

    info!("Test IamCsTenantServ::modify_item");
    IamCsTenantServ::modify_item(
        &id1,
        &mut IamCsTenantModifyReq {
            name: None,
            icon: None,
            sort: None,
            contact_phone: None,
            disabled: None,
        },
        &tx,
        &context,
    )
    .await?;
    IamCsTenantServ::modify_item(
        &id2,
        &mut IamCsTenantModifyReq {
            name: None,
            icon: None,
            sort: None,
            contact_phone: Some("xxxx".to_string()),
            disabled: None,
        },
        &tx,
        &context,
    )
    .await?;

    info!("Test IamCsTenantServ::paginate_items");
    let tenants = IamCsTenantServ::paginate_items(
        &RbumItemFilterReq {
            name: Some("测试租户%".to_string()),
            ..Default::default()
        },
        1,
        10,
        None,
        &tx,
        &context,
    )
    .await?;
    assert_eq!(tenants.page_number, 1);
    assert_eq!(tenants.page_size, 10);
    assert_eq!(tenants.total_size, 2);
    assert!(tenants.records.iter().any(|r| r.contact_phone == "xxxx"));

    info!("Test IamCsTenantServ::delete_rbum");
    IamCsTenantServ::delete_item(&id1, &tx, &context).await?;
    assert!(IamCsTenantServ::get_item(&id1, &RbumItemFilterReq::default(), &tx, &context).await.is_err());

    tx.rollback().await?;

    Ok(())
}
