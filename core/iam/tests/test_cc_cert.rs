use bios_basic::rbum::dto::rbum_filer_dto::RbumCertFilterReq;
use bios_basic::rbum::helper::rbum_scope_helper;
use bios_iam::basic::dto::iam_cert_dto::{IamCertExtAddReq, IamCertManageAddReq};
use bios_iam::basic::dto::iam_cert_dto::{IamCertUserPwdModifyReq, IamCertUserPwdRestReq};
use bios_iam::basic::serv::iam_cert_serv::IamCertServ;
use bios_iam::basic::serv::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use bios_iam::console_passport::dto::iam_cp_cert_dto::IamCpUserPwdLoginReq;
use bios_iam::console_passport::serv::iam_cp_cert_user_pwd_serv::IamCpCertUserPwdServ;
use bios_iam::iam_constants;
use bios_iam::iam_constants::{RBUM_ITEM_NAME_SYS_ADMIN_ACCOUNT, RBUM_SCOPE_LEVEL_TENANT};
use bios_iam::iam_enumeration::IamCertKernelKind;
use bios_iam::iam_enumeration::{IamCertExtKind, IamCertManageKind};
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;

pub async fn test(
    sys_context: &TardisContext,
    t1_context: &TardisContext,
    t2_context: &TardisContext,
    t2_a1_context: &TardisContext,
    t2_a2_context: &TardisContext,
) -> TardisResult<()> {
    test_single_level(sys_context, RBUM_ITEM_NAME_SYS_ADMIN_ACCOUNT, t1_context).await?;
    test_single_level(t1_context, "bios", t2_context).await?;
    // test_single_level(t2_a1_context, "app_admin1", t2_a2_context).await?;
    Ok(())
}

async fn test_single_level(context: &TardisContext, ak: &str, another_context: &TardisContext) -> TardisResult<()> {
    let mut funs = iam_constants::get_tardis_inst();
    funs.begin().await?;

    info!("【test_cc_cert】 : test_single_level : Rest Password");
    let rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_code(
        IamCertKernelKind::UserPwd.to_string().as_str(),
        rbum_scope_helper::get_path_item(RBUM_SCOPE_LEVEL_TENANT.to_int(), &context.own_paths),
        &funs,
    )
    .await?;
    assert!(IamCertUserPwdServ::reset_sk(
        &IamCertUserPwdRestReq {
            new_sk: TrimString("sssssssssss".to_string())
        },
        &another_context.owner,
        &rbum_cert_conf_id,
        &funs,
        context
    )
    .await
    .is_err());
    assert!(IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
            ak: TrimString(ak.to_string()),
            sk: TrimString("sssssssssss".to_string()),
            tenant_id: rbum_scope_helper::get_path_item(RBUM_SCOPE_LEVEL_TENANT.to_int(), &context.own_paths),
            flag: None
        },
        &funs,
    )
    .await
    .is_err());
    IamCertUserPwdServ::reset_sk(
        &IamCertUserPwdRestReq {
            new_sk: TrimString("sssssssssss".to_string()),
        },
        &context.owner,
        &rbum_cert_conf_id,
        &funs,
        context,
    )
    .await?;
    assert!(IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
            ak: TrimString(ak.to_string()),
            sk: TrimString("sssssssssss".to_string()),
            tenant_id: rbum_scope_helper::get_path_item(RBUM_SCOPE_LEVEL_TENANT.to_int(), &another_context.own_paths),
            flag: None
        },
        &funs,
    )
    .await
    .is_err());
    let account_info = IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
            ak: TrimString(ak.to_string()),
            sk: TrimString("sssssssssss".to_string()),
            tenant_id: rbum_scope_helper::get_path_item(RBUM_SCOPE_LEVEL_TENANT.to_int(), &context.own_paths),
            flag: None,
        },
        &funs,
    )
    .await?;

    info!("【test_cc_cert】 : test_single_level : Modify Cert");
    assert!(IamCpCertUserPwdServ::modify_cert_user_pwd(
        &another_context.owner,
        &IamCertUserPwdModifyReq {
            original_sk: TrimString("aaa".to_string()),
            new_sk: TrimString("123456789".to_string())
        },
        &funs,
        another_context
    )
    .await
    .is_err());
    assert!(IamCpCertUserPwdServ::modify_cert_user_pwd(
        &context.owner,
        &IamCertUserPwdModifyReq {
            original_sk: TrimString("aaa".to_string()),
            new_sk: TrimString("123456789".to_string())
        },
        &funs,
        context
    )
    .await
    .is_err());

    IamCpCertUserPwdServ::modify_cert_user_pwd(
        &context.owner,
        &IamCertUserPwdModifyReq {
            original_sk: TrimString("sssssssssss".to_string()),
            new_sk: TrimString("123456789".to_string()),
        },
        &funs,
        context,
    )
    .await?;

    IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
            ak: TrimString(ak.to_string()),
            sk: TrimString("123456789".to_string()),
            tenant_id: rbum_scope_helper::get_path_item(RBUM_SCOPE_LEVEL_TENANT.to_int(), &context.own_paths),
            flag: None,
        },
        &funs,
    )
    .await?;

    info!("【test_cc_cert】 : test_single_level : Add Ext Cert - Gitlab");
    assert!(IamCertServ::get_ext_cert(&account_info.account_id, &IamCertExtKind::Gitlab, &funs, context).await.is_err());
    IamCertServ::add_ext_cert(
        &mut IamCertExtAddReq {
            ak: "GitlabUserId".to_string(),
            sk: Some("ssssssssss".to_string()),
            ext: None,
        },
        &account_info.account_id,
        &IamCertExtKind::Gitlab,
        &funs,
        context,
    )
    .await?;
    assert_eq!(
        IamCertServ::get_ext_cert(&account_info.account_id, &IamCertExtKind::Gitlab, &funs, context).await?.ak,
        "GitlabUserId"
    );

    info!("【test_cc_cert】 : test_single_level : Manage Cert");
    let manage_user_pwd_conf_id = IamCertServ::get_cert_conf_id_by_code(
        IamCertManageKind::ManageUserPwd.to_string().as_str(),
        rbum_scope_helper::get_max_level_id_by_context(&another_context),
        &funs,
    )
    .await?;
    let manage_user_visa_conf_id = IamCertServ::get_cert_conf_id_by_code(
        IamCertManageKind::ManageUserVisa.to_string().as_str(),
        rbum_scope_helper::get_max_level_id_by_context(&another_context),
        &funs,
    )
    .await?;

    let manage_cert_pwd_id = IamCertServ::add_manage_cert(
        &IamCertManageAddReq {
            ak: "manage_pwd_ak".to_string(),
            sk: Some("123456".to_string()),
            rel_rbum_cert_conf_id: Some(manage_user_pwd_conf_id.clone()),
            ext: Some("测试用户名/密码".to_string()),
        },
        &funs,
        &another_context,
    )
    .await?;

    let manage_cert_visa_id = IamCertServ::add_manage_cert(
        &IamCertManageAddReq {
            ak: "manage_visa_ak".to_string(),
            sk: Some("123456".to_string()),
            rel_rbum_cert_conf_id: Some(manage_user_visa_conf_id.clone()),
            ext: Some("测试用户名/证书".to_string()),
        },
        &funs,
        &another_context,
    )
    .await?;
    IamCertServ::modify_manage_cert_ext(&manage_cert_visa_id, "测试用户名/密码2", &funs, &another_context).await?;
    let manage_cert_result = IamCertServ::paginate_certs(
        &RbumCertFilterReq {
            rel_rbum_cert_conf_ids: Some(vec![manage_user_pwd_conf_id.clone(), manage_user_visa_conf_id.clone()]),
            ..Default::default()
        },
        1,
        20,
        None,
        None,
        &funs,
        &another_context,
    )
    .await?;
    assert_eq!(manage_cert_result.records.len(), 2);

    IamCertServ::add_rel_cert(&manage_cert_pwd_id, "123456", None, None, &funs, &another_context).await?;
    assert_eq!(IamCertServ::find_to_simple_rel_cert("123456", None, None, &funs, &another_context).await?.len(), 1);
    IamCertServ::add_rel_cert(&manage_cert_visa_id, "123456", None, None, &funs, &another_context).await?;
    assert_eq!(IamCertServ::find_to_simple_rel_cert("123456", None, None, &funs, &another_context).await?.len(), 2);
    IamCertServ::delete_rel_cert(&manage_cert_pwd_id, "123456", &funs, &another_context).await?;
    assert_eq!(IamCertServ::find_to_simple_rel_cert("123456", None, None, &funs, &another_context).await?.len(), 1);
    IamCertServ::delete_rel_cert(&manage_cert_visa_id, "123456", &funs, &another_context).await?;
    assert_eq!(IamCertServ::find_to_simple_rel_cert("123456", None, None, &funs, &another_context).await?.len(), 0);
    funs.rollback().await?;
    Ok(())
}
