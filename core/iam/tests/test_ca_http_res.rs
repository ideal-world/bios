use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;

use bios_iam::console_app::dto::iam_ca_http_res_dto::{IamCaHttpResAddReq, IamCaHttpResModifyReq};
use bios_iam::console_app::serv::iam_ca_http_res_serv::IamCaHttpResServ;
use bios_iam::console_app::serv::iam_ca_role_serv::IamCaRoleServ;
use bios_iam::iam_constants;

pub async fn test(context1: &TardisContext, context2: &TardisContext) -> TardisResult<()> {
    let mut funs = iam_constants::get_tardis_inst();
    funs.begin().await?;

    info!("【test_ca_http_res】 : Add Http Res");
    let http_res_id1 = IamCaHttpResServ::add_http_res(
        &mut IamCaHttpResAddReq {
            name: TrimString("测试资源1".to_string()),
            code: TrimString("test_code1".to_string()),
            method: TrimString("GET".to_string()),
            sort: None,
            icon: None,
            disabled: None,
        },
        &funs,
        context1,
    )
    .await?;

    let http_res_id2 = IamCaHttpResServ::add_http_res(
        &mut IamCaHttpResAddReq {
            name: TrimString("测试资源2".to_string()),
            code: TrimString("test_code2".to_string()),
            method: TrimString("GET".to_string()),
            sort: None,
            icon: None,
            disabled: None,
        },
        &funs,
        context2,
    )
    .await?;

    info!("【test_ca_http_res】 : Modify Http Res By Id, with err");
    assert!(IamCaHttpResServ::modify_http_res(
        &http_res_id1,
        &mut IamCaHttpResModifyReq {
            name: Some(TrimString("测试资源3".to_string())),
            code: None,
            icon: None,
            sort: None,
            method: None,
            disabled: None
        },
        &funs,
        context2
    )
    .await
    .is_err());
    info!("【test_ca_http_res】 : Modify Http Res By Id");
    IamCaHttpResServ::modify_http_res(
        &http_res_id1,
        &mut IamCaHttpResModifyReq {
            name: Some(TrimString("测试资源".to_string())),
            code: None,
            icon: None,
            sort: None,
            method: Some(TrimString("POST".to_string())),
            disabled: None,
        },
        &funs,
        context1,
    )
    .await?;

    info!("【test_ca_http_res】 : Get Http Res By Id, with err");
    assert!(IamCaHttpResServ::get_http_res(&http_res_id1, &funs, context2).await.is_err());
    info!("【test_ca_http_res】 : Get Http Res By Id");
    let http_res = IamCaHttpResServ::get_http_res(&http_res_id1, &funs, context1).await?;
    assert_eq!(http_res.id, http_res_id1);
    assert_eq!(http_res.name, "测试资源");
    assert_eq!(http_res.method, "POST");
    assert!(!http_res.disabled);

    info!("【test_ca_http_res】 : Find Http Res");
    let http_res = IamCaHttpResServ::paginate_http_res(None, None, 1, 10, None, None, &funs, context1).await?;
    assert_eq!(http_res.page_number, 1);
    assert_eq!(http_res.page_size, 10);
    assert_eq!(http_res.total_size, 1);
    assert!(http_res.records.iter().any(|i| i.name == "测试资源"));

    info!("【test_ca_http_res】 : Delete Http Res By Id, with err");
    assert!(IamCaHttpResServ::delete_http_res("11111", &funs, context1).await.is_err());
    info!("【test_ca_http_res】 : Delete Http Res By Id, with err");
    assert!(IamCaHttpResServ::delete_http_res(&http_res_id1, &funs, context2).await.is_err());
    info!("【test_ca_http_res】 : Delete Http Res By Id");
    assert_eq!(
        IamCaHttpResServ::paginate_http_res(Some(http_res_id1.clone()), None, 1, 10, None, None, &funs, context1).await?.total_size,
        1
    );
    IamCaHttpResServ::delete_http_res(&http_res_id1, &funs, context1).await?;
    assert_eq!(
        IamCaHttpResServ::paginate_http_res(Some(http_res_id1.clone()), None, 1, 10, None, None, &funs, context1).await?.total_size,
        0
    );

    // ----------------------- Rel Role -----------------------

    info!("【test_ca_http_res】 : Find Rel Roles By Http Res Id, is empty");
    let rel_roles = IamCaHttpResServ::paginate_rel_roles(&http_res_id2, 1, 10, None, None, &funs, context2).await?;
    assert_eq!(rel_roles.total_size, 0);

    info!("【test_ca_http_res】 : Add Rel Http Res By Id");
    IamCaRoleServ::add_rel_http_res(context2.roles.get(0).unwrap(), &http_res_id2, None, None, &funs, context2).await?;
    info!("【test_ca_http_res】 : Find Rel Accounts By Http Res Id");
    let rel_accounts = IamCaHttpResServ::paginate_rel_roles(&http_res_id2, 1, 10, None, None, &funs, context2).await?;
    assert_eq!(rel_accounts.total_size, 1);
    assert_eq!(rel_accounts.records.get(0).unwrap().rel.from_rbum_item_name, "测试资源2");
    assert_eq!(rel_accounts.records.get(0).unwrap().rel.to_rbum_item_name, "app_admin");

    funs.rollback().await?;

    Ok(())
}
