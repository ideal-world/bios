use std::time::Duration;

use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::{TardisFuns, TardisFunsInst};

use bios_basic::rbum::dto::rbum_cert_conf_dto::{RbumCertConfAddReq, RbumCertConfModifyReq};
use bios_basic::rbum::dto::rbum_cert_dto::{RbumCertAddReq, RbumCertModifyReq};
use bios_basic::rbum::dto::rbum_domain_dto::RbumDomainAddReq;
use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumCertConfFilterReq, RbumCertFilterReq};
use bios_basic::rbum::rbum_enumeration::{RbumCertRelKind, RbumCertStatusKind, RbumScopeLevelKind};
use bios_basic::rbum::serv::rbum_cert_serv::{RbumCertConfServ, RbumCertServ};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_domain_serv::RbumDomainServ;

pub async fn test(context: &TardisContext) -> TardisResult<()> {
    test_rbum_cert_conf(context).await?;
    test_rbum_cert_basic(context).await?;
    test_rbum_cert_sk_dynamic(context).await?;
    Ok(())
}

async fn test_rbum_cert_conf(context: &TardisContext) -> TardisResult<()> {
    let mut funs = TardisFuns::inst_with_db_conn("".to_string(), None);
    funs.begin().await?;

    info!("【test_rbum_cert_conf】 : Prepare Domain : RbumDomainServ::add_rbum");
    let domain_iam_id = RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: TrimString("iam2".to_string()),
            name: TrimString("IAM".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: Some(RbumScopeLevelKind::L2),
        },
        &funs,
        context,
    )
    .await?;
    test_rbum_cert_conf_is_ak_repeatable(domain_iam_id.clone(), &funs, context);
    // -----------------------------------

    info!("【test_rbum_cert_conf】 : Test Add : RbumCertConfServ::add_rbum");
    assert!(RbumCertConfServ::add_rbum(
        &mut RbumCertConfAddReq {
            code: TrimString("UserPwd".to_string()),
            name: TrimString("用户名+密码".to_string()),
            note: None,
            ak_note: None,
            ak_rule: None,
            sk_note: None,
            sk_rule: None,
            ext: None,
            sk_need: None,
            sk_dynamic: None,
            sk_encrypted: None,
            repeatable: None,
            is_basic: None,
            is_ak_repeatable: None,
            rest_by_kinds: None,
            expire_sec: None,
            coexist_num: None,
            conn_uri: None,
            rel_rbum_domain_id: "".to_string(),
            rel_rbum_item_id: None,
            sk_lock_cycle_sec: None,
            sk_lock_err_times: None,
            sk_lock_duration_sec: None
        },
        &funs,
        context,
    )
    .await
    .is_err());

    assert!(RbumCertConfServ::add_rbum(
        &mut RbumCertConfAddReq {
            code: TrimString("UserPwd".to_string()),
            name: TrimString("用户名+密码".to_string()),
            note: None,
            ak_note: None,
            ak_rule: Some("--".to_string()),
            sk_note: None,
            sk_rule: None,
            ext: None,
            sk_need: None,
            sk_dynamic: None,
            sk_encrypted: None,
            repeatable: None,
            is_basic: None,
            is_ak_repeatable: None,
            rest_by_kinds: None,
            expire_sec: None,
            coexist_num: None,
            conn_uri: None,
            rel_rbum_domain_id: "".to_string(),
            rel_rbum_item_id: None,
            sk_lock_cycle_sec: None,
            sk_lock_err_times: None,
            sk_lock_duration_sec: None
        },
        &funs,
        context,
    )
    .await
    .is_err());

    let id = RbumCertConfServ::add_rbum(
        &mut RbumCertConfAddReq {
            code: TrimString("UserPwd".to_string()),
            name: TrimString("用户名+密码".to_string()),
            note: None,
            ak_note: None,
            ak_rule: Some("^[a-zA-Z0-9]{6,20}$".to_string()),
            sk_note: None,
            sk_rule: Some("^.{8,40}$".to_string()),
            ext: None,
            sk_need: Some(true),
            sk_dynamic: None,
            sk_encrypted: Some(true),
            repeatable: None,
            is_basic: None,
            is_ak_repeatable: None,
            rest_by_kinds: None,
            expire_sec: None,
            coexist_num: None,
            conn_uri: None,
            rel_rbum_domain_id: domain_iam_id.to_string(),
            rel_rbum_item_id: None,
            sk_lock_cycle_sec: None,
            sk_lock_err_times: None,
            sk_lock_duration_sec: None,
        },
        &funs,
        context,
    )
    .await?;

    assert!(RbumCertConfServ::add_rbum(
        &mut RbumCertConfAddReq {
            code: TrimString("UserPwd".to_string()),
            name: TrimString("用户名+密码".to_string()),
            note: None,
            ak_note: None,
            ak_rule: Some("^[a-zA-Z0-9]{6,20}$".to_string()),
            sk_note: None,
            sk_rule: Some("^.{8,40}$".to_string()),
            ext: None,
            sk_need: Some(true),
            sk_dynamic: None,
            sk_encrypted: Some(true),
            repeatable: None,
            is_basic: None,
            is_ak_repeatable: None,
            rest_by_kinds: None,
            expire_sec: None,
            coexist_num: None,
            conn_uri: None,
            rel_rbum_domain_id: domain_iam_id.to_string(),
            rel_rbum_item_id: None,
            sk_lock_cycle_sec: None,
            sk_lock_err_times: None,
            sk_lock_duration_sec: None
        },
        &funs,
        context,
    )
    .await
    .is_err());

    info!("【test_rbum_cert_conf】 : Test Get : RbumCertConfServ::get_rbum");
    let rbum = RbumCertConfServ::get_rbum(&id, &RbumCertConfFilterReq::default(), &funs, context).await?;
    assert_eq!(rbum.id, id);
    assert_eq!(rbum.name, "用户名+密码");
    assert_eq!(rbum.expire_sec, u32::MAX);

    info!("【test_rbum_cert_conf】 : Test Modify : RbumCertConfServ::modify_rbum");
    RbumCertConfServ::modify_rbum(
        &id,
        &mut RbumCertConfModifyReq {
            name: None,
            note: None,
            ak_note: Some("AK".to_string()),
            ak_rule: None,
            sk_note: None,
            sk_rule: None,
            ext: None,
            sk_need: None,
            sk_encrypted: None,
            repeatable: None,
            is_basic: None,
            rest_by_kinds: None,
            expire_sec: None,
            coexist_num: None,
            conn_uri: None,
            sk_lock_cycle_sec: None,
            sk_lock_err_times: None,
            sk_lock_duration_sec: None,
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_rbum_cert_conf】 : Test Find : RbumCertConfServ::paginate_rbums");
    let rbums = RbumCertConfServ::paginate_rbums(
        &RbumCertConfFilterReq {
            basic: RbumBasicFilterReq {
                name: Some("用户名%".to_string()),
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
    assert_eq!(rbums.page_number, 1);
    assert_eq!(rbums.page_size, 10);
    assert_eq!(rbums.total_size, 1);
    assert_eq!(rbums.records.get(0).unwrap().name, "用户名+密码");

    info!("【test_rbum_cert_conf】 : Test Delete : RbumCertConfServ::delete_rbum");
    assert!(RbumCertConfServ::delete_rbum(&id, &funs, context).await.is_err());

    funs.rollback().await?;

    Ok(())
}

async fn test_rbum_cert_conf_is_ak_repeatable(domain_iam_id: String, funs: &TardisFunsInst, context: &TardisContext) -> TardisResult<()> {
    let ak_repeatable_true_test_id = RbumCertConfServ::add_rbum(
        &mut RbumCertConfAddReq {
            code: TrimString("Test1".to_string()),
            name: TrimString("akRepeatableTest1".to_string()),
            note: None,
            ak_note: None,
            ak_rule: None,
            sk_note: None,
            sk_rule: None,
            ext: None,
            sk_need: Some(true),
            sk_dynamic: None,
            sk_encrypted: Some(false),
            repeatable: None,
            is_basic: None,
            is_ak_repeatable: Some(true),
            rest_by_kinds: None,
            expire_sec: None,
            coexist_num: None,
            conn_uri: None,
            rel_rbum_domain_id: domain_iam_id.to_string(),
            rel_rbum_item_id: None,
            sk_lock_cycle_sec: None,
            sk_lock_err_times: None,
            sk_lock_duration_sec: None,
        },
        funs,
        context,
    )
    .await?;
    let ak_repeatable_false_test_id = RbumCertConfServ::add_rbum(
        &mut RbumCertConfAddReq {
            code: TrimString("Test2".to_string()),
            name: TrimString("akRepeatableTest2".to_string()),
            note: None,
            ak_note: None,
            ak_rule: None,
            sk_note: None,
            sk_rule: None,
            ext: None,
            sk_need: Some(true),
            sk_dynamic: None,
            sk_encrypted: Some(false),
            repeatable: None,
            is_basic: None,
            is_ak_repeatable: Some(false),
            rest_by_kinds: None,
            expire_sec: None,
            coexist_num: None,
            conn_uri: None,
            rel_rbum_domain_id: domain_iam_id.to_string(),
            rel_rbum_item_id: None,
            sk_lock_cycle_sec: None,
            sk_lock_err_times: None,
            sk_lock_duration_sec: None,
        },
        funs,
        context,
    )
    .await?;

    let cert_test_id1 = RbumCertServ::add_rbum(
        &mut RbumCertAddReq {
            ak: "test".into(),
            sk: Some("test".into()),
            vcode: None,
            ext: None,
            start_time: None,
            end_time: None,
            conn_uri: None,
            status: RbumCertStatusKind::Enabled,
            rel_rbum_cert_conf_id: Some(ak_repeatable_true_test_id.clone()),
            rel_rbum_kind: RbumCertRelKind::Item,
            rel_rbum_id: context.owner.to_string(),
            is_outside: false,
        },
        funs,
        context,
    )
    .await?;

    assert!(!RbumCertServ::add_rbum(
        &mut RbumCertAddReq {
            ak: "test".into(),
            sk: Some("test1".into()),
            vcode: None,
            ext: None,
            start_time: None,
            end_time: None,
            conn_uri: None,
            status: RbumCertStatusKind::Enabled,
            rel_rbum_cert_conf_id: Some(ak_repeatable_true_test_id.clone()),
            rel_rbum_kind: RbumCertRelKind::Item,
            rel_rbum_id: context.owner.to_string(),
            is_outside: false,
        },
        funs,
        context,
    )
    .await?
    .is_empty());

    let cert_test_id2 = RbumCertServ::add_rbum(
        &mut RbumCertAddReq {
            ak: "test".into(),
            sk: Some("test".into()),
            vcode: None,
            ext: None,
            start_time: None,
            end_time: None,
            conn_uri: None,
            status: RbumCertStatusKind::Enabled,
            rel_rbum_cert_conf_id: Some(ak_repeatable_false_test_id.clone()),
            rel_rbum_kind: RbumCertRelKind::Item,
            rel_rbum_id: context.owner.to_string(),
            is_outside: false,
        },
        funs,
        context,
    )
    .await?;

    assert!(RbumCertServ::add_rbum(
        &mut RbumCertAddReq {
            ak: "test".into(),
            sk: Some("test".into()),
            vcode: None,
            ext: None,
            start_time: None,
            end_time: None,
            conn_uri: None,
            status: RbumCertStatusKind::Enabled,
            rel_rbum_cert_conf_id: Some(ak_repeatable_false_test_id.clone()),
            rel_rbum_kind: RbumCertRelKind::Item,
            rel_rbum_id: context.owner.to_string(),
            is_outside: false,
        },
        funs,
        context,
    )
    .await
    .is_err());

    //===========modify==============

    RbumCertServ::modify_rbum(
        &cert_test_id1,
        &mut RbumCertModifyReq {
            ak: Some("test".into()),
            sk: None,
            ext: None,
            start_time: None,
            end_time: None,
            conn_uri: None,
            status: None,
        },
        &funs,
        context,
    )
    .await?;

    assert!(RbumCertServ::modify_rbum(
        &cert_test_id2,
        &mut RbumCertModifyReq {
            ak: Some("test".into()),
            sk: None,
            ext: None,
            start_time: None,
            end_time: None,
            conn_uri: None,
            status: None,
        },
        &funs,
        context,
    )
        .await
        .is_err());

    Ok(())
}

async fn test_rbum_cert_basic(context: &TardisContext) -> TardisResult<()> {
    let mut funs = TardisFuns::inst_with_db_conn("".to_string(), None);
    funs.begin().await?;

    info!("【test_rbum_cert】 : Prepare Domain : RbumDomainServ::add_rbum");
    let domain_iam_id = RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: TrimString("iam2".to_string()),
            name: TrimString("IAM".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: Some(RbumScopeLevelKind::L2),
        },
        &funs,
        context,
    )
    .await?;

    let domain_db_id = RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: TrimString("mysql-dev".to_string()),
            name: TrimString("Mysql测试集群".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: Some(RbumScopeLevelKind::L2),
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_rbum_cert】 : Prepare Cert Conf : RbumCertConfServ::add_rbum");
    let cert_conf_user_pwd_id = RbumCertConfServ::add_rbum(
        &mut RbumCertConfAddReq {
            code: TrimString("UserPwd".to_string()),
            name: TrimString("用户名+密码".to_string()),
            note: None,
            ak_note: None,
            ak_rule: Some("^[a-zA-Z0-9]{6,20}$".to_string()),
            sk_note: None,
            sk_rule: Some("^.{8,40}$".to_string()),
            ext: None,
            sk_need: Some(true),
            sk_dynamic: None,
            sk_encrypted: Some(true),
            repeatable: None,
            is_basic: Some(true),
            is_ak_repeatable: None,
            rest_by_kinds: None,
            expire_sec: Some(2),
            coexist_num: None,
            conn_uri: None,
            rel_rbum_domain_id: domain_iam_id.to_string(),
            rel_rbum_item_id: None,
            sk_lock_cycle_sec: None,
            sk_lock_err_times: None,
            sk_lock_duration_sec: None,
        },
        &funs,
        context,
    )
    .await?;

    let cert_conf_ssh_id = RbumCertConfServ::add_rbum(
        &mut RbumCertConfAddReq {
            code: TrimString("MysqlConn".to_string()),
            name: TrimString("mysql ident".to_string()),
            note: None,
            ak_note: None,
            ak_rule: None,
            sk_note: None,
            sk_rule: None,
            ext: None,
            sk_need: Some(true),
            sk_dynamic: None,
            sk_encrypted: Some(false),
            repeatable: None,
            is_basic: Some(false),
            is_ak_repeatable: None,
            rest_by_kinds: None,
            expire_sec: None,
            coexist_num: None,
            conn_uri: None,
            rel_rbum_domain_id: domain_db_id.to_string(),
            rel_rbum_item_id: None,
            sk_lock_cycle_sec: None,
            sk_lock_err_times: None,
            sk_lock_duration_sec: None,
        },
        &funs,
        context,
    )
    .await?;

    // -----------------------------------
    info!("【test_rbum_cert】 : Test Add : RbumCertServ::add_rbum");
    // ak regex check error
    assert!(RbumCertServ::add_rbum(
        &mut RbumCertAddReq {
            ak: TrimString("gdxr".to_string()),
            sk: None,
            ext: None,
            vcode: None,
            start_time: None,
            end_time: None,
            conn_uri: None,
            status: RbumCertStatusKind::Enabled,
            rel_rbum_cert_conf_id: Some("".to_string()),
            rel_rbum_kind: RbumCertRelKind::Item,
            rel_rbum_id: "".to_string(),
            is_outside: false
        },
        &funs,
        context,
    )
    .await
    .is_err());

    // sk cannot be empty
    assert!(RbumCertServ::add_rbum(
        &mut RbumCertAddReq {
            ak: TrimString("gudaoxuri".to_string()),
            sk: None,
            ext: None,
            vcode: None,
            start_time: None,
            end_time: None,
            conn_uri: None,
            status: RbumCertStatusKind::Enabled,
            rel_rbum_cert_conf_id: Some("".to_string()),
            rel_rbum_kind: RbumCertRelKind::Item,
            rel_rbum_id: "".to_string(),
            is_outside: false
        },
        &funs,
        context,
    )
    .await
    .is_err());

    // sk regex check error
    assert!(RbumCertServ::add_rbum(
        &mut RbumCertAddReq {
            ak: TrimString("gudaoxuri".to_string()),
            sk: Some(TrimString("aa".to_string())),
            vcode: None,
            ext: None,
            start_time: None,
            end_time: None,
            conn_uri: None,
            status: RbumCertStatusKind::Enabled,
            rel_rbum_cert_conf_id: Some("".to_string()),
            rel_rbum_kind: RbumCertRelKind::Item,
            rel_rbum_id: "".to_string(),
            is_outside: false
        },
        &funs,
        context,
    )
    .await
    .is_err());

    // cert conf not found
    assert!(RbumCertServ::add_rbum(
        &mut RbumCertAddReq {
            ak: TrimString("gudaoxuri".to_string()),
            sk: Some(TrimString("12345678".to_string())),
            vcode: None,
            ext: None,
            start_time: None,
            end_time: None,
            conn_uri: None,
            status: RbumCertStatusKind::Enabled,
            rel_rbum_cert_conf_id: Some("".to_string()),
            rel_rbum_kind: RbumCertRelKind::Item,
            rel_rbum_id: "".to_string(),
            is_outside: false
        },
        &funs,
        context,
    )
    .await
    .is_err());

    // item not found
    assert!(RbumCertServ::add_rbum(
        &mut RbumCertAddReq {
            ak: TrimString("gudaoxuri".to_string()),
            sk: Some(TrimString("12345678".to_string())),
            vcode: None,
            ext: None,
            start_time: None,
            end_time: None,
            conn_uri: None,
            status: RbumCertStatusKind::Enabled,
            rel_rbum_cert_conf_id: Some(cert_conf_user_pwd_id.to_string()),
            rel_rbum_kind: RbumCertRelKind::Item,
            rel_rbum_id: "11".to_string(),
            is_outside: false,
        },
        &funs,
        context,
    )
    .await
    .is_err());

    let cert_gudaoxuri_id = RbumCertServ::add_rbum(
        &mut RbumCertAddReq {
            ak: TrimString("gudaoxuri".to_string()),
            sk: Some(TrimString("12345678".to_string())),
            vcode: None,
            ext: None,
            start_time: None,
            end_time: None,
            conn_uri: None,
            status: RbumCertStatusKind::Enabled,
            rel_rbum_cert_conf_id: Some(cert_conf_user_pwd_id.to_string()),
            rel_rbum_kind: RbumCertRelKind::Item,
            rel_rbum_id: context.owner.to_string(),
            is_outside: false,
        },
        &funs,
        context,
    )
    .await?;

    // Exist ak
    assert!(RbumCertServ::add_rbum(
        &mut RbumCertAddReq {
            ak: TrimString("gudaoxuri".to_string()),
            sk: Some(TrimString("12345678".to_string())),
            vcode: None,
            ext: None,
            start_time: None,
            end_time: None,
            conn_uri: None,
            status: RbumCertStatusKind::Enabled,
            rel_rbum_cert_conf_id: Some(cert_conf_user_pwd_id.to_string()),
            rel_rbum_kind: RbumCertRelKind::Item,
            rel_rbum_id: context.owner.to_string(),
            is_outside: false
        },
        &funs,
        context,
    )
    .await
    .is_err());

    let cert_root_id = RbumCertServ::add_rbum(
        &mut RbumCertAddReq {
            ak: TrimString("root".to_string()),
            sk: Some(TrimString("12345678".to_string())),
            vcode: None,
            ext: None,
            start_time: None,
            end_time: None,
            conn_uri: None,
            status: RbumCertStatusKind::Enabled,
            rel_rbum_cert_conf_id: Some(cert_conf_ssh_id.to_string()),
            rel_rbum_kind: RbumCertRelKind::Item,
            rel_rbum_id: context.owner.to_string(),
            is_outside: false,
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_rbum_cert】 : Test Get : RbumCertServ::get_rbum");
    let rbum = RbumCertServ::get_rbum(&cert_gudaoxuri_id, &RbumCertFilterReq::default(), &funs, context).await?;
    assert_eq!(rbum.id, cert_gudaoxuri_id);
    assert_eq!(rbum.ak, "gudaoxuri");
    assert!(rbum.end_time.timestamp() - rbum.start_time.timestamp() < 2010);
    let rbum = RbumCertServ::get_rbum(&cert_root_id, &RbumCertFilterReq::default(), &funs, context).await?;
    assert_eq!(rbum.id, cert_root_id);
    assert_eq!(rbum.ak, "root");
    assert_eq!(rbum.start_time.timestamp() + u32::MAX as i64, rbum.end_time.timestamp());

    info!("【test_rbum_cert】 : Test Modify : RbumCertServ::modify_rbum");
    assert!(RbumCertServ::modify_rbum(
        "111",
        &mut RbumCertModifyReq {
            ext: Some("ext".to_string()),
            ak: None,
            sk: None,
            start_time: None,
            end_time: None,
            conn_uri: None,
            status: None
        },
        &funs,
        context
    )
    .await
    .is_err());

    RbumCertServ::modify_rbum(
        &cert_gudaoxuri_id,
        &mut RbumCertModifyReq {
            ext: Some("ext".to_string()),
            ak: None,
            sk: None,
            start_time: None,
            end_time: None,
            conn_uri: None,
            status: None,
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_rbum_cert】 : Test Find : RbumCertServ::paginate_rbums");
    let rbums = RbumCertServ::paginate_rbums(&RbumCertFilterReq::default(), 1, 10, None, None, &funs, context).await?;
    assert_eq!(rbums.page_number, 1);
    assert_eq!(rbums.page_size, 10);
    assert_eq!(rbums.total_size, 2);

    info!("【test_rbum_cert】 : Test Show SK : RbumCertServ::show_sk");
    assert!(RbumCertServ::show_sk("11", &RbumCertFilterReq::default(), &funs, context).await.is_err());
    assert_ne!(RbumCertServ::show_sk(&cert_gudaoxuri_id, &RbumCertFilterReq::default(), &funs, context).await?, "12345678");
    assert_eq!(RbumCertServ::show_sk(&cert_root_id, &RbumCertFilterReq::default(), &funs, context).await?, "12345678");

    info!("【test_rbum_cert】 : Test Reset SK : RbumCertServ::reset_sk");
    assert!(RbumCertServ::reset_sk("11", "111", &RbumCertFilterReq::default(), &funs, context).await.is_err());
    assert!(RbumCertServ::reset_sk(&cert_gudaoxuri_id, "111", &RbumCertFilterReq::default(), &funs, context).await.is_err());
    RbumCertServ::reset_sk(&cert_gudaoxuri_id, "87654321", &RbumCertFilterReq::default(), &funs, context).await?;
    RbumCertServ::reset_sk(&cert_root_id, "87654321", &RbumCertFilterReq::default(), &funs, context).await?;

    info!("【test_rbum_cert】 : Test Change SK : RbumCertServ::change_sk");
    assert!(RbumCertServ::change_sk("11", "11", "111", &RbumCertFilterReq::default(), &funs, context).await.is_err());
    assert!(RbumCertServ::change_sk(&cert_gudaoxuri_id, "11", "111", &RbumCertFilterReq::default(), &funs, context).await.is_err());
    assert!(RbumCertServ::change_sk(&cert_gudaoxuri_id, "111", "12345678", &RbumCertFilterReq::default(), &funs, context).await.is_err());
    assert!(RbumCertServ::change_sk(&cert_gudaoxuri_id, "87654321", "111", &RbumCertFilterReq::default(), &funs, context).await.is_err());
    RbumCertServ::change_sk(&cert_gudaoxuri_id, "87654321", "abcdefgh", &RbumCertFilterReq::default(), &funs, context).await?;
    RbumCertServ::change_sk(&cert_root_id, "87654321", "abcdefgh", &RbumCertFilterReq::default(), &funs, context).await?;

    info!("【test_rbum_cert】 : Test Validate : RbumCertServ::validate");
    assert!(RbumCertServ::validate_by_spec_cert_conf("11", "11", "111", false, "11", &funs).await.is_err());
    assert!(RbumCertServ::validate_by_spec_cert_conf("gudaoxuri", "11", "111", false, "11", &funs).await.is_err());
    assert!(RbumCertServ::validate_by_spec_cert_conf("gudaoxuri", "11", &cert_conf_user_pwd_id, false, "11", &funs).await.is_err());
    assert!(RbumCertServ::validate_by_spec_cert_conf("gudaoxuri", "11", &cert_conf_user_pwd_id, false, &context.own_paths, &funs).await.is_err());
    tardis::tokio::time::sleep(Duration::from_secs(1)).await;
    info!("Test Validate RbumCertServ::validate gudaoxuri abcdefgh");
    assert_eq!(
        RbumCertServ::validate_by_spec_cert_conf("gudaoxuri", "abcdefgh", &cert_conf_user_pwd_id, false, &context.own_paths, &funs).await?.0,
        cert_gudaoxuri_id.to_string()
    );
    info!("Test Validate RbumCertServ::validate root abcdefgh");
    assert_eq!(
        RbumCertServ::validate_by_spec_cert_conf("root", "abcdefgh", &cert_conf_ssh_id, false, &context.own_paths, &funs).await?.0,
        cert_root_id.to_string()
    );
    tardis::tokio::time::sleep(Duration::from_secs(3)).await;
    // Expire
    info!("Test Validate Expire RbumCertServ::validate gudaoxuri abcdefgh");
    assert!(RbumCertServ::validate_by_spec_cert_conf("gudaoxuri", "abcdefgh", &cert_conf_user_pwd_id, false, &context.own_paths, &funs).await.is_err());

    info!("【test_rbum_cert】 : Test Delete : RbumCertServ::delete_rbum");
    RbumCertServ::delete_rbum(&cert_gudaoxuri_id, &funs, context).await?;
    assert!(RbumCertServ::get_rbum(&cert_gudaoxuri_id, &RbumCertFilterReq::default(), &funs, context).await.is_err());
    RbumCertServ::delete_rbum(&cert_root_id, &funs, context).await?;
    assert!(RbumCertServ::get_rbum(&cert_root_id, &RbumCertFilterReq::default(), &funs, context).await.is_err());

    funs.rollback().await?;

    Ok(())
}

async fn test_rbum_cert_sk_dynamic(context: &TardisContext) -> TardisResult<()> {
    let mut funs = TardisFuns::inst_with_db_conn("".to_string(), None);
    funs.begin().await?;

    info!("【test_rbum_cert】 : Prepare Domain : RbumDomainServ::add_rbum");
    let domain_iam_id = RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: TrimString("iam2".to_string()),
            name: TrimString("IAM".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: Some(RbumScopeLevelKind::L2),
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_rbum_cert】 : Prepare Cert Conf : RbumCertConfServ::add_rbum");
    let cert_conf_mail_vcode_id = RbumCertConfServ::add_rbum(
        &mut RbumCertConfAddReq {
            code: TrimString("MailVCode".to_string()),
            name: TrimString("邮箱+验证码".to_string()),
            note: None,
            ak_note: None,
            ak_rule: None,
            sk_note: None,
            sk_rule: None,
            ext: None,
            sk_need: Some(false),
            sk_dynamic: Some(true),
            sk_encrypted: Some(false),
            repeatable: None,
            is_basic: Some(false),
            is_ak_repeatable: None,
            rest_by_kinds: None,
            expire_sec: Some(2),
            coexist_num: None,
            conn_uri: None,
            rel_rbum_domain_id: domain_iam_id.to_string(),
            rel_rbum_item_id: None,
            sk_lock_cycle_sec: None,
            sk_lock_err_times: None,
            sk_lock_duration_sec: None,
        },
        &funs,
        context,
    )
    .await?;

    // -----------------------------------
    info!("【test_rbum_cert】 : Test Add : RbumCertServ::add_rbum with sk_dynamic");
    RbumCertServ::add_rbum(
        &mut RbumCertAddReq {
            ak: TrimString("i@sunisle.org".to_string()),
            sk: None,
            vcode: Some(TrimString("123456".to_string())),
            ext: None,
            start_time: None,
            end_time: None,
            conn_uri: None,
            status: RbumCertStatusKind::Enabled,
            rel_rbum_cert_conf_id: Some(cert_conf_mail_vcode_id.to_string()),
            rel_rbum_kind: RbumCertRelKind::Item,
            rel_rbum_id: context.owner.to_string(),
            is_outside: false,
        },
        &funs,
        context,
    )
    .await?;
    tardis::tokio::time::sleep(Duration::from_secs(1)).await;
    info!("【test_rbum_cert】 : Test Validate : RbumCertServ::validate with sk_dynamic");
    RbumCertServ::validate_by_spec_cert_conf("i@sunisle.org", "123456", &cert_conf_mail_vcode_id, false, &context.own_paths, &funs).await?;
    assert!(RbumCertServ::validate_by_spec_cert_conf("i@sunisle.org", "123456", &cert_conf_mail_vcode_id, false, &context.own_paths, &funs).await.is_err());

    info!("【test_rbum_cert】 : Test Add : RbumCertServ::get_and_delete_vcode_in_cache");
    assert!(RbumCertServ::get_and_delete_vcode_in_cache("i@sunisle.org", &context.own_paths, &funs).await?.is_none());
    RbumCertServ::add_vcode_to_cache("i@sunisle.org", "qqqqq", &context.own_paths, &funs).await?;
    assert_eq!(
        RbumCertServ::get_and_delete_vcode_in_cache("i@sunisle.org", &context.own_paths, &funs).await?.unwrap(),
        "qqqqq"
    );

    info!("【test_rbum_cert】 : Test Validate : RbumCertServ::validate with sk_dynamic");
    RbumCertServ::add_vcode_to_cache("i@sunisle.org", "xxxx", &context.own_paths, &funs).await?;
    RbumCertServ::validate_by_spec_cert_conf("i@sunisle.org", "xxxx", &cert_conf_mail_vcode_id, false, &context.own_paths, &funs).await?;

    info!("【test_rbum_cert】 : Test Validate : RbumCertServ::validate with sk_dynamic & expire");
    RbumCertServ::add_vcode_to_cache("i@sunisle.org", "xxxx", &context.own_paths, &funs).await?;
    tardis::tokio::time::sleep(Duration::from_secs(3)).await;
    assert!(RbumCertServ::validate_by_spec_cert_conf("i@sunisle.org", "xxxx", &cert_conf_mail_vcode_id, false, &context.own_paths, &funs).await.is_err());

    funs.rollback().await?;

    Ok(())
}
