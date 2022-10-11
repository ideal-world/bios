use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;

use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_iam::basic::dto::iam_cert_conf_dto::IamCertConfUserPwdAddOrModifyReq;
use bios_iam::basic::dto::iam_filer_dto::IamTenantFilterReq;
use bios_iam::basic::dto::iam_tenant_dto::{IamTenantAggAddReq, IamTenantModifyReq};
use bios_iam::basic::serv::iam_cert_serv::IamCertServ;
use bios_iam::basic::serv::iam_tenant_serv::IamTenantServ;
use bios_iam::iam_constants;

pub async fn test(context: &TardisContext) -> TardisResult<()> {
    let mut funs = iam_constants::get_tardis_inst();
    funs.begin().await?;

    info!("【test_cs_tenant】 : Add Tenant");
    let (tenant_id, _) = IamTenantServ::add_tenant_agg(
        &IamTenantAggAddReq {
            name: TrimString("测试租户1".to_string()),
            icon: None,
            contact_phone: None,
            note: None,
            admin_username: TrimString("admin".to_string()),
            disabled: None,
            admin_name: TrimString("测试管理员".to_string()),
            admin_password: None,
            cert_conf_by_user_pwd: IamCertConfUserPwdAddOrModifyReq {
                ak_rule_len_min: 2,
                ak_rule_len_max: 20,
                sk_rule_len_min: 2,
                sk_rule_len_max: 20,
                sk_rule_need_num: false,
                sk_rule_need_uppercase: false,
                sk_rule_need_lowercase: false,
                sk_rule_need_spec_char: false,
                sk_lock_cycle_sec: 0,
                sk_lock_err_times: 0,
                sk_lock_duration_sec: 0,
                repeatable: true,
                expire_sec: 111,
            },
            cert_conf_by_phone_vcode: true,
            cert_conf_by_mail_vcode: true,
            account_self_reg: None,
            cert_conf_by_wechat_mp: None,
            cert_conf_by_ldap: None,
        },
        &funs,
    )
    .await?;

    IamTenantServ::add_tenant_agg(
        &IamTenantAggAddReq {
            name: TrimString("测试租户2".to_string()),
            icon: None,
            contact_phone: Some("12345678901".to_string()),
            admin_username: TrimString("admin".to_string()),
            note: None,
            disabled: None,
            admin_name: TrimString("测试管理员".to_string()),
            admin_password: None,
            cert_conf_by_user_pwd: IamCertConfUserPwdAddOrModifyReq {
                ak_rule_len_min: 2,
                ak_rule_len_max: 20,
                sk_rule_len_min: 2,
                sk_rule_len_max: 20,
                sk_rule_need_num: false,
                sk_rule_need_uppercase: false,
                sk_rule_need_lowercase: false,
                sk_rule_need_spec_char: false,
                sk_lock_cycle_sec: 0,
                sk_lock_err_times: 0,
                sk_lock_duration_sec: 0,
                repeatable: true,
                expire_sec: 111,
            },
            cert_conf_by_phone_vcode: true,
            cert_conf_by_mail_vcode: true,
            account_self_reg: None,
            cert_conf_by_wechat_mp: None,
            cert_conf_by_ldap: None,
        },
        &funs,
    )
    .await?;

    let tenant_id2 = IamTenantServ::add_tenant_agg(
        &IamTenantAggAddReq {
            name: TrimString("测试租户2".to_string()),
            icon: None,
            contact_phone: Some("12345678901".to_string()),
            note: None,
            admin_username: TrimString("admin1".to_string()),
            disabled: None,
            admin_name: TrimString("测试管理员".to_string()),
            admin_password: None,
            cert_conf_by_user_pwd: IamCertConfUserPwdAddOrModifyReq {
                ak_rule_len_min: 2,
                ak_rule_len_max: 20,
                sk_rule_len_min: 2,
                sk_rule_len_max: 20,
                sk_rule_need_num: false,
                sk_rule_need_uppercase: false,
                sk_rule_need_lowercase: false,
                sk_rule_need_spec_char: false,
                sk_lock_cycle_sec: 0,
                sk_lock_err_times: 0,
                sk_lock_duration_sec: 0,
                repeatable: true,
                expire_sec: 111,
            },
            cert_conf_by_phone_vcode: true,
            cert_conf_by_mail_vcode: true,
            account_self_reg: None,
            cert_conf_by_wechat_mp: None,
            cert_conf_by_ldap: None,
        },
        &funs,
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
            note: None,
            account_self_reg: None,
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
            note: None,
            account_self_reg: None,
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
