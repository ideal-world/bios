use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;

use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_iam::basic::dto::iam_tenant_dto::IamTenantModifyReq;
use bios_iam::basic::serv::iam_tenant_serv::IamTenantServ;
use bios_iam::console_system::dto::iam_cs_tenant_dto::{IamCsTenantAddReq, IamCsTenantModifyReq};
use bios_iam::console_system::serv::iam_cs_tenant_serv::IamCsTenantServ;
use bios_iam::iam_constants;

pub async fn test(context: &TardisContext) -> TardisResult<()> {
    let mut funs = iam_constants::get_tardis_inst();
    funs.begin().await?;

    info!("【test_cs_tenant】 : Add Tenant");
    let (tenant_id, _) = IamCsTenantServ::add_tenant(
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
    .await?;

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

    let tenant_id2 = IamCsTenantServ::add_tenant(
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

    info!("【test_cs_tenant】 : Get Tenant By Id");
    let tenant = IamCsTenantServ::get_tenant(&tenant_id, &funs, context).await?;
    assert_eq!(tenant.id, tenant_id);
    assert_eq!(tenant.name, "测试租户1");
    assert_eq!(tenant.contact_phone, "");
    let tenant = IamCsTenantServ::get_tenant(&tenant_id2, &funs, context).await?;
    assert_eq!(tenant.id, tenant_id2);
    assert_eq!(tenant.name, "测试租户2");
    assert_eq!(tenant.contact_phone, "12345678901");

    info!("【test_cs_tenant】 : Modify Tenant By Id");
    IamCsTenantServ::modify_tenant(&tenant_id, &mut IamCsTenantModifyReq { disabled: Some(true) }, &funs, context).await?;

    IamTenantServ::modify_item(
        &tenant_id2,
        &mut IamTenantModifyReq {
            name: None,
            icon: None,
            sort: None,
            contact_phone: Some("xxxx".to_string()),
            disabled: None,
            scope_level: None,
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_cs_tenant】 : Find Tenants");
    let tenants = IamCsTenantServ::paginate_tenants(None, None, 1, 10, None, None, &funs, context).await?;
    assert_eq!(tenants.page_number, 1);
    assert_eq!(tenants.page_size, 10);
    assert!(tenants.records.iter().any(|r| r.contact_phone == "xxxx"));
    assert!(tenants.records.iter().any(|r| r.disabled));

    funs.rollback().await?;

    Ok(())
}
