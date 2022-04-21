use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;

use bios_iam::console_app::dto::iam_ca_http_res_dto::IamCaHttpResAddReq;
use bios_iam::console_app::dto::iam_ca_role_dto::{IamCaRoleAddReq, IamCaRoleModifyReq};
use bios_iam::console_app::serv::iam_ca_http_res_serv::IamCaHttpResServ;
use bios_iam::console_app::serv::iam_ca_role_serv::IamCaRoleServ;
use bios_iam::iam_constants;

pub async fn test(context1: &TardisContext, context2: &TardisContext) -> TardisResult<()> {
    let mut funs = iam_constants::get_tardis_inst();
    funs.begin().await?;

    info!("【test_ca_role】 : Add Role");
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

    let role_id2 = IamCaRoleServ::add_role(
        &mut IamCaRoleAddReq {
            name: TrimString("角色2".to_string()),
            icon: None,
            sort: None,
            disabled: None,
        },
        &funs,
        context2,
    )
    .await?;

    info!("【test_ca_role】 : Modify Role By Id, with err");
    assert!(IamCaRoleServ::modify_role(
        &role_id1,
        &mut IamCaRoleModifyReq {
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
    info!("【test_ca_role】 : Modify Role By Id");
    IamCaRoleServ::modify_role(
        &role_id1,
        &mut IamCaRoleModifyReq {
            name: Some(TrimString("角色".to_string())),
            icon: None,
            sort: None,
            disabled: None,
        },
        &funs,
        context1,
    )
    .await?;

    info!("【test_ca_role】 : Get Role By Id, with err");
    assert!(IamCaRoleServ::get_role(&role_id1, &funs, context2).await.is_err());
    info!("【test_ca_role】 : Get App By Id");
    let role = IamCaRoleServ::get_role(&role_id1, &funs, context1).await?;
    assert_eq!(role.id, role_id1);
    assert_eq!(role.name, "角色");
    assert!(!role.disabled);

    info!("【test_ca_role】 : Find Roles");
    let roles = IamCaRoleServ::paginate_roles(None, None, 1, 10, None, None, &funs, context1).await?;
    assert_eq!(roles.page_number, 1);
    assert_eq!(roles.page_size, 10);
    assert_eq!(roles.total_size, 1);
    assert!(roles.records.iter().any(|i| i.name == "角色"));

    info!("【test_ca_role】 : Delete Role By Id, with err");
    assert!(IamCaRoleServ::delete_role("11111", &funs, context1).await.is_err());
    info!("【test_ca_role】 : Delete Role By Id, with err");
    assert!(IamCaRoleServ::delete_role(&role_id1, &funs, context2).await.is_err());
    info!("【test_ca_role】 : Delete Role By Id");
    assert_eq!(
        IamCaRoleServ::paginate_roles(Some(role_id1.clone()), None, 1, 10, None, None, &funs, context1).await?.total_size,
        1
    );
    IamCaRoleServ::delete_role(&role_id1, &funs, context1).await?;
    assert_eq!(
        IamCaRoleServ::paginate_roles(Some(role_id1.clone()), None, 1, 10, None, None, &funs, context1).await?.total_size,
        0
    );

    // ----------------------- Rel Account -----------------------

    info!("【test_ca_role】 : Find Rel Accounts By Role Id, is empty");
    let rel_accounts = IamCaRoleServ::paginate_rel_accounts(&role_id2, 1, 10, None, None, &funs, context2).await?;
    assert_eq!(rel_accounts.total_size, 0);

    info!("【test_ca_role】 : Add Rel Account By Id, with err");
    assert!(IamCaRoleServ::add_rel_account(&role_id1, &context1.owner, None, None, &funs, context2).await.is_err());
    info!("【test_ca_role】 : Add Rel Account By Id");
    IamCaRoleServ::add_rel_account(&role_id2, &context2.owner, None, None, &funs, context2).await?;

    info!("【test_ca_role】 : Find Rel Accounts By Role Id");
    let rel_accounts = IamCaRoleServ::paginate_rel_accounts(&role_id2, 1, 10, None, None, &funs, context2).await?;
    assert_eq!(rel_accounts.total_size, 1);
    assert_eq!(rel_accounts.records.get(0).unwrap().rel.from_rbum_item_name, "应用2管理员");
    assert_eq!(rel_accounts.records.get(0).unwrap().rel.to_rbum_item_name, "角色2");

    info!("【test_ca_role】 : Delete Rel By Id");
    IamCaRoleServ::delete_rel(&rel_accounts.records.get(0).unwrap().rel.id, &funs, context2).await?;
    let rel_accounts = IamCaRoleServ::paginate_rel_accounts(&role_id2, 1, 10, None, None, &funs, context2).await?;
    assert_eq!(rel_accounts.total_size, 0);

    // ----------------------- Rel Http Res -----------------------
    let http_res_id = IamCaHttpResServ::add_http_res(
        &mut IamCaHttpResAddReq {
            name: TrimString("测试资源".to_string()),
            code: TrimString("test_code".to_string()),
            method: TrimString("GET".to_string()),
            sort: None,
            icon: None,
            disabled: None,
        },
        &funs,
        context2,
    )
    .await?;

    info!("【test_ca_role】 : Find Rel Http Res By Role Id, is empty");
    let rel_http_res = IamCaRoleServ::paginate_rel_http_res(&role_id2, 1, 10, None, None, &funs, context2).await?;
    assert_eq!(rel_http_res.total_size, 0);

    info!("【test_ca_role】 : Add Rel Http Res By Id, with err");
    assert!(IamCaRoleServ::add_rel_http_res(&role_id1, "xxxx", None, None, &funs, context2).await.is_err());
    info!("【test_ca_role】 : Add Rel Http Res By Id");
    IamCaRoleServ::add_rel_http_res(&role_id2, &http_res_id, None, None, &funs, context2).await?;

    info!("【test_ca_role】 : Find Rel Http Res By Role Id");
    let rel_http_res = IamCaRoleServ::paginate_rel_http_res(&role_id2, 1, 10, None, None, &funs, context2).await?;
    assert_eq!(rel_http_res.total_size, 1);
    assert_eq!(rel_http_res.records.get(0).unwrap().rel.from_rbum_item_name, "测试资源");
    assert_eq!(rel_http_res.records.get(0).unwrap().rel.to_rbum_item_name, "角色2");

    info!("【test_ca_role】 : Delete Rel By Id");
    IamCaRoleServ::delete_rel(&rel_http_res.records.get(0).unwrap().rel.id, &funs, context2).await?;
    let rel_http_res = IamCaRoleServ::paginate_rel_http_res(&role_id2, 1, 10, None, None, &funs, context2).await?;
    assert_eq!(rel_http_res.total_size, 0);

    funs.rollback().await?;

    Ok(())
}
