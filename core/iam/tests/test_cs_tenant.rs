use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;

use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_iam::basic::dto::iam_cert_conf_dto::{IamMailVCodeCertConfAddOrModifyReq, IamPhoneVCodeCertConfAddOrModifyReq, IamUserPwdCertConfAddOrModifyReq};
use bios_iam::basic::dto::iam_filer_dto::IamTenantFilterReq;
use bios_iam::basic::dto::iam_tenant_dto::IamTenantModifyReq;
use bios_iam::basic::serv::iam_cert_serv::IamCertServ;
use bios_iam::basic::serv::iam_tenant_serv::IamTenantServ;
use bios_iam::console_system::dto::iam_cs_tenant_dto::IamCsTenantAddReq;
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
            tenant_note: None,
            admin_username: TrimString("admin".to_string()),
            disabled: None,
            admin_name: TrimString("测试管理员".to_string()),
            admin_password: None,
            cert_conf_by_user_pwd: IamUserPwdCertConfAddOrModifyReq {
                ak_note: None,
                ak_rule: None,
                sk_note: None,
                sk_rule: None,
                repeatable: Some(true),
                expire_sec: None
            },
            cert_conf_by_phone_vcode: Some(IamPhoneVCodeCertConfAddOrModifyReq{
                ak_note: None,
                ak_rule: None
            }),

            cert_conf_by_mail_vcode: Some(IamMailVCodeCertConfAddOrModifyReq{
                ak_note: None,
                ak_rule: None
            }),
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
            tenant_note: None,
            disabled: None,
            admin_name: TrimString("测试管理员".to_string()),
            admin_password: None,
            cert_conf_by_user_pwd: IamUserPwdCertConfAddOrModifyReq {
                ak_note: None,
                ak_rule: None,
                sk_note: None,
                sk_rule: None,
                repeatable: Some(true),
                expire_sec: None
            },
            cert_conf_by_phone_vcode: Some(IamPhoneVCodeCertConfAddOrModifyReq{
                ak_note: None,
                ak_rule: None
            }),

            cert_conf_by_mail_vcode: Some(IamMailVCodeCertConfAddOrModifyReq{
                ak_note: None,
                ak_rule: None
            }),
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
            tenant_note: None,
            admin_username: TrimString("admin1".to_string()),
            disabled: None,
            admin_name: TrimString("测试管理员".to_string()),
            admin_password: None,
            cert_conf_by_user_pwd: IamUserPwdCertConfAddOrModifyReq {
                ak_note: None,
                ak_rule: None,
                sk_note: None,
                sk_rule: None,
                repeatable: Some(true),
                expire_sec: None
            },
            cert_conf_by_phone_vcode: Some(IamPhoneVCodeCertConfAddOrModifyReq{
                ak_note: None,
                ak_rule: None
            }),

            cert_conf_by_mail_vcode: Some(IamMailVCodeCertConfAddOrModifyReq{
                ak_note: None,
                ak_rule: None
            }),
        },
        &funs,
        context,
    )
    .await?
    .0;

    info!("【test_cs_tenant】 : Get Tenant By Id");
    let tenant = IamTenantServ::get_item(&tenant_id, &IamTenantFilterReq::default(), &funs, context).await?;
    assert_eq!(tenant.id, tenant_id);
    assert_eq!(tenant.name, "测试租户1");
    assert_eq!(tenant.contact_phone, "");
    let tenant = IamTenantServ::get_item(&tenant_id2, &IamTenantFilterReq::default(), &funs, context).await?;
    assert_eq!(tenant.id, tenant_id2);
    assert_eq!(tenant.name, "测试租户2");
    assert_eq!(tenant.contact_phone, "12345678901");

    info!("【test_cs_tenant】 : Modify Tenant By Id");
    IamTenantServ::modify_item(
        &tenant_id,
        &mut IamTenantModifyReq {
            name: None,
            icon: None,
            sort: None,
            contact_phone: None,
            disabled: Some(true),
            scope_level: None,
            note: None
        },
        &funs,
        context,
    )
    .await?;

    IamTenantServ::modify_item(
        &tenant_id2,
        &mut IamTenantModifyReq {
            name: None,
            icon: None,
            sort: None,
            contact_phone: Some("xxxx".to_string()),
            disabled: None,
            scope_level: None,
            note: None
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_cs_tenant】 : Find Tenants");
    let tenants = IamTenantServ::paginate_items(
        &IamTenantFilterReq {
            basic: RbumBasicFilterReq {
                ids: None,
                name: None,
                ..Default::default()
            },
            ..Default::default()
        },
        1,
        10,
        None,
        None,
        &funs,
        context,
    )
    .await?;
    assert_eq!(tenants.page_number, 1);
    assert_eq!(tenants.page_size, 10);
    assert!(tenants.records.iter().any(|r| r.contact_phone == "xxxx"));
    assert!(tenants.records.iter().any(|r| r.disabled));

    let tenants = IamTenantServ::find_items(
        &IamTenantFilterReq {
            basic: RbumBasicFilterReq {
                ignore_scope: true,
                own_paths: Some("".to_string()),
                with_sub_own_paths: true,
                enabled: Some(true),
                ..Default::default()
            },
            ..Default::default()
        },
        None,
        None,
        &funs,
        &IamCertServ::get_anonymous_context(),
    )
    .await?;
    assert_eq!(tenants.len(), 2);

    funs.rollback().await?;

    Ok(())
}
