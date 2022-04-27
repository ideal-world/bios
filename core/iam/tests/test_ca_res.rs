use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;

use bios_iam::console_app::dto::iam_ca_res_dto::{IamCaResAddReq, IamCaResModifyReq};
use bios_iam::console_app::serv::iam_ca_res_serv::IamCaResServ;
use bios_iam::console_app::serv::iam_ca_role_serv::IamCaRoleServ;
use bios_iam::iam_constants;
use bios_iam::iam_enumeration::IamResKind;

pub async fn test(context1: &TardisContext, context2: &TardisContext) -> TardisResult<()> {
    let mut funs = iam_constants::get_tardis_inst();
    funs.begin().await?;

    info!("【test_ca_res】 : Add Res");
    let res_id1 = IamCaResServ::add_res(
        &mut IamCaResAddReq {
            name: TrimString("测试资源1".to_string()),
            code: TrimString("test_code1".to_string()),
            method: TrimString("GET".to_string()),
            hide: None,
            sort: None,
            icon: None,
            disabled: None,
            kind: IamResKind::API,
            action: None,
        },
        &funs,
        context1,
    )
    .await?;

    let res_id2 = IamCaResServ::add_res(
        &mut IamCaResAddReq {
            name: TrimString("测试资源2".to_string()),
            code: TrimString("test_code2".to_string()),
            method: TrimString("GET".to_string()),
            hide: None,
            sort: None,
            icon: None,
            disabled: None,
            kind: IamResKind::API,
            action: None,
        },
        &funs,
        context2,
    )
    .await?;

    info!("【test_ca_res】 : Modify Res By Id, with err");
    assert!(IamCaResServ::modify_res(
        &res_id1,
        &mut IamCaResModifyReq {
            name: Some(TrimString("测试资源3".to_string())),
            code: None,
            icon: None,
            sort: None,
            method: None,
            hide: None,
            action: None,
            disabled: None
        },
        &funs,
        context2
    )
    .await
    .is_err());
    info!("【test_ca_res】 : Modify Res By Id");
    IamCaResServ::modify_res(
        &res_id1,
        &mut IamCaResModifyReq {
            name: Some(TrimString("测试资源".to_string())),
            code: None,
            icon: None,
            sort: None,
            method: Some(TrimString("POST".to_string())),
            hide: None,
            action: None,
            disabled: None,
        },
        &funs,
        context1,
    )
    .await?;

    info!("【test_ca_res】 : Get Res By Id, with err");
    assert!(IamCaResServ::get_res(&res_id1, &funs, context2).await.is_err());
    info!("【test_ca_res】 : Get Res By Id");
    let res = IamCaResServ::get_res(&res_id1, &funs, context1).await?;
    assert_eq!(res.id, res_id1);
    assert_eq!(res.name, "测试资源");
    assert_eq!(res.method, "POST");
    assert!(!res.disabled);

    info!("【test_ca_res】 : Find Res");
    let res = IamCaResServ::paginate_res(IamResKind::API, None, None, 1, 10, None, None, &funs, context1).await?;
    assert_eq!(res.page_number, 1);
    assert_eq!(res.page_size, 10);
    assert_eq!(res.total_size, 1);
    assert!(res.records.iter().any(|i| i.name == "测试资源"));

    info!("【test_ca_res】 : Delete Res By Id, with err");
    assert!(IamCaResServ::delete_res("11111", &funs, context1).await.is_err());
    info!("【test_ca_res】 : Delete Res By Id, with err");
    assert!(IamCaResServ::delete_res(&res_id1, &funs, context2).await.is_err());
    info!("【test_ca_res】 : Delete Res By Id");
    assert_eq!(
        IamCaResServ::paginate_res(IamResKind::API, Some(res_id1.clone()), None, 1, 10, None, None, &funs, context1).await?.total_size,
        1
    );
    IamCaResServ::delete_res(&res_id1, &funs, context1).await?;
    assert_eq!(
        IamCaResServ::paginate_res(IamResKind::API, Some(res_id1.clone()), None, 1, 10, None, None, &funs, context1).await?.total_size,
        0
    );

    // ----------------------- Rel Role -----------------------

    info!("【test_ca_res】 : Find Rel Roles By Res Id, is empty");
    let rel_roles = IamCaResServ::paginate_rel_roles(&res_id2, 1, 10, None, None, &funs, context2).await?;
    assert_eq!(rel_roles.total_size, 0);

    info!("【test_ca_res】 : Add Rel Res By Id");
    IamCaRoleServ::add_rel_res(context2.roles.get(0).unwrap(), &res_id2, None, None, &funs, context2).await?;
    info!("【test_ca_res】 : Find Rel Accounts By Res Id");
    let rel_accounts = IamCaResServ::paginate_rel_roles(&res_id2, 1, 10, None, None, &funs, context2).await?;
    assert_eq!(rel_accounts.total_size, 1);
    assert_eq!(rel_accounts.records.get(0).unwrap().rel.from_rbum_item_name, "测试资源2");
    assert_eq!(rel_accounts.records.get(0).unwrap().rel.to_rbum_item_name, "app_admin");

    funs.rollback().await?;

    Ok(())
}
