use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;

use bios_iam::console_tenant::dto::iam_ct_http_res_dto::IamCtHttpResAddReq;
use bios_iam::console_tenant::dto::iam_ct_role_dto::{IamCtRoleAddReq, IamCtRoleModifyReq};
use bios_iam::console_tenant::serv::iam_ct_http_res_serv::IamCtHttpResServ;
use bios_iam::console_tenant::serv::iam_ct_role_serv::IamCtRoleServ;
use bios_iam::iam_constants;

pub async fn test(context1: &TardisContext, context2: &TardisContext) -> TardisResult<()> {
    let mut funs = iam_constants::get_tardis_inst();
    funs.begin().await?;

    info!("【test_ct_role】 : Add Role");
    let role_id1 = IamCtRoleServ::add_role(
        &mut IamCtRoleAddReq {
            name: TrimString("角色1".to_string()),
            icon: None,
            sort: None,
            disabled: None,
        },
        &funs,
        context1,
    )
    .await?;

    let role_id2 = IamCtRoleServ::add_role(
        &mut IamCtRoleAddReq {
            name: TrimString("角色2".to_string()),
            icon: None,
            sort: None,
            disabled: None,
        },
        &funs,
        context2,
    )
    .await?;

    info!("【test_ct_role】 : Modify Role By Id, with err");
    assert!(IamCtRoleServ::modify_role(
        &role_id1,
        &mut IamCtRoleModifyReq {
            name: Some(TrimString("角色3".to_string())),
            icon: None,
            sort: None,
            disabled: None
        },
        &funs,
        context2
    )
    .await
    .is_err());
    info!("【test_ct_role】 : Modify Role By Id");
    IamCtRoleServ::modify_role(
        &role_id1,
        &mut IamCtRoleModifyReq {
            name: Some(TrimString("角色".to_string())),
            icon: None,
            sort: None,
            disabled: None,
        },
        &funs,
        context1,
    )
    .await?;

    info!("【test_ct_role】 : Get Role By Id, with err");
    assert!(IamCtRoleServ::get_role(&role_id1, &funs, context2).await.is_err());
    info!("【test_ct_role】 : Get App By Id");
    let role = IamCtRoleServ::get_role(&role_id1, &funs, context1).await?;
    assert_eq!(role.id, role_id1);
    assert_eq!(role.name, "角色");
    assert!(!role.disabled);

    info!("【test_ct_role】 : Find Roles");
    let roles = IamCtRoleServ::paginate_roles(None, None, 1, 10, None, None, &funs, context1).await?;
    assert_eq!(roles.page_number, 1);
    assert_eq!(roles.page_size, 10);
    assert_eq!(roles.total_size, 1);
    assert!(roles.records.iter().any(|i| i.name == "角色"));

    info!("【test_ct_role】 : Delete Role By Id, with err");
    assert!(IamCtRoleServ::delete_role("11111", &funs, &context1).await.is_err());
    info!("【test_ct_role】 : Delete Role By Id, with err");
    assert!(IamCtRoleServ::delete_role(&role_id1, &funs, &context2).await.is_err());
    info!("【test_ct_role】 : Delete Role By Id");
    assert_eq!(
        IamCtRoleServ::paginate_roles(Some(role_id1.clone()), None, 1, 10, None, None, &funs, context1).await?.total_size,
        1
    );
    IamCtRoleServ::delete_role(&role_id1, &funs, &context1).await?;
    assert_eq!(
        IamCtRoleServ::paginate_roles(Some(role_id1.clone()), None, 1, 10, None, None, &funs, context1).await?.total_size,
        0
    );

    // ----------------------- Rel Account -----------------------

    info!("【test_ct_role】 : Find Rel Accounts By Role Id, is empty");
    let rel_accounts = IamCtRoleServ::paginate_rel_accounts(&role_id2, 1, 10, None, None, &funs, context2).await?;
    assert_eq!(rel_accounts.total_size, 0);

    info!("【test_ct_role】 : Add Rel Account By Id, with err");
    assert!(IamCtRoleServ::add_rel_account(&role_id1, &context1.owner, None, None, &funs, context2).await.is_err());
    info!("【test_ct_role】 : Add Rel Account By Id");
    IamCtRoleServ::add_rel_account(&role_id2, &context2.owner, None, None, &funs, context2).await?;

    info!("【test_ct_role】 : Find Rel Accounts By Role Id");
    let rel_accounts = IamCtRoleServ::paginate_rel_accounts(&role_id2, 1, 10, None, None, &funs, context2).await?;
    assert_eq!(rel_accounts.total_size, 1);
    assert_eq!(rel_accounts.records.get(0).unwrap().rel.from_rbum_item_name, "角色2");
    assert_eq!(rel_accounts.records.get(0).unwrap().rel.to_rbum_item_name, "测试管理员2");

    info!("【test_ct_role】 : Delete Rel By Id");
    IamCtRoleServ::delete_rel(&rel_accounts.records.get(0).unwrap().rel.id, &funs, context2).await?;
    let rel_accounts = IamCtRoleServ::paginate_rel_accounts(&role_id2, 1, 10, None, None, &funs, context2).await?;
    assert_eq!(rel_accounts.total_size, 0);

    // ----------------------- Rel Http Res -----------------------
    let http_res_id = IamCtHttpResServ::add_http_res(
        &mut IamCtHttpResAddReq {
            name: TrimString("测试资源".to_string()),
            code: TrimString("test_code".to_string()),
            method: TrimString("GET".to_string()),
            sort: None,
            icon: None,
            disabled: None,
        },
        &funs,
        context1,
    )
    .await?;

    info!("【test_ct_role】 : Find Rel Http Res By Role Id, is empty");
    let rel_http_res = IamCtRoleServ::paginate_rel_http_res(&role_id2, 1, 10, None, None, &funs, context2).await?;
    assert_eq!(rel_http_res.total_size, 0);

    info!("【test_ct_role】 : Add Rel Http Res By Id, with err");
    assert!(IamCtRoleServ::add_rel_http_res(&role_id1, "xxxx", None, None, &funs, context2).await.is_err());
    info!("【test_ct_role】 : Add Rel Http Res By Id");
    IamCtRoleServ::add_rel_http_res(&role_id2, &http_res_id, None, None, &funs, context2).await?;

    info!("【test_ct_role】 : Find Rel Http Res By Role Id");
    let rel_http_res = IamCtRoleServ::paginate_rel_http_res(&role_id2, 1, 10, None, None, &funs, context2).await?;
    assert_eq!(rel_http_res.total_size, 1);
    assert_eq!(rel_http_res.records.get(0).unwrap().rel.from_rbum_item_name, "角色2");
    assert_eq!(rel_http_res.records.get(0).unwrap().rel.to_rbum_item_name, "测试资源");

    info!("【test_ct_role】 : Delete Rel By Id");
    IamCtRoleServ::delete_rel(&rel_http_res.records.get(0).unwrap().rel.id, &funs, context2).await?;
    let rel_http_res = IamCtRoleServ::paginate_rel_http_res(&role_id2, 1, 10, None, None, &funs, context2).await?;
    assert_eq!(rel_http_res.total_size, 0);

    funs.rollback().await?;

    Ok(())
}
