use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::TardisFuns;

use bios_basic::rbum::dto::rbum_domain_dto::RbumDomainAddReq;
use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_domain_serv::RbumDomainServ;

pub async fn test() -> TardisResult<()> {
    let mut funs = TardisFuns::inst_with_db_conn("".to_string());
    funs.begin().await?;

    info!("【test_scope】 : Prepare");
    let s0 = TardisContext {
        own_paths: "".to_string(),
        ak: "".to_string(),
        owner: "".to_string(),
        roles: vec![],
        groups: vec![],
    };

    let s1 = TardisContext {
        own_paths: TardisFuns::field.nanoid_len(4),
        ak: "".to_string(),
        owner: "".to_string(),
        roles: vec![],
        groups: vec![],
    };

    let s2 = TardisContext {
        own_paths: format!("{}/{}", s1.own_paths, TardisFuns::field.nanoid_len(4)),
        ak: "".to_string(),
        owner: "".to_string(),
        roles: vec![],
        groups: vec![],
    };

    let s3 = TardisContext {
        own_paths: format!("{}/{}", s2.own_paths, TardisFuns::field.nanoid_len(4)),
        ak: "".to_string(),
        owner: "".to_string(),
        roles: vec![],
        groups: vec![],
    };

    RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: TrimString("s0-l3".to_string()),
            name: TrimString("scope_test_s0-l3".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: Some(RbumScopeLevelKind::L3),
        },
        &funs,
        &s0,
    )
    .await?;

    RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: TrimString("s0-l2".to_string()),
            name: TrimString("scope_test_s0-l2".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: Some(RbumScopeLevelKind::L2),
        },
        &funs,
        &s0,
    )
    .await?;

    RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: TrimString("s0-l1".to_string()),
            name: TrimString("scope_test_s0-l1".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: Some(RbumScopeLevelKind::L1),
        },
        &funs,
        &s0,
    )
    .await?;

    RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: TrimString("s0-l0".to_string()),
            name: TrimString("scope_test_s0-l0".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: Some(RbumScopeLevelKind::Root),
        },
        &funs,
        &s0,
    )
    .await?;

    RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: TrimString("s1-l3".to_string()),
            name: TrimString("scope_test_s1-l3".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: Some(RbumScopeLevelKind::L3),
        },
        &funs,
        &s1,
    )
    .await?;

    RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: TrimString("s1-l2".to_string()),
            name: TrimString("scope_test_s1-l2".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: Some(RbumScopeLevelKind::L2),
        },
        &funs,
        &s1,
    )
    .await?;

    RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: TrimString("s1-l1".to_string()),
            name: TrimString("scope_test_s1-l1".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: Some(RbumScopeLevelKind::L1),
        },
        &funs,
        &s1,
    )
    .await?;

    RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: TrimString("s1-l0".to_string()),
            name: TrimString("scope_test_s1-l0".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: Some(RbumScopeLevelKind::Root),
        },
        &funs,
        &s1,
    )
    .await?;

    RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: TrimString("s2-l3".to_string()),
            name: TrimString("scope_test_s2-l3".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: Some(RbumScopeLevelKind::L3),
        },
        &funs,
        &s2,
    )
    .await?;

    RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: TrimString("s2-l2".to_string()),
            name: TrimString("scope_test_s2-l2".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: Some(RbumScopeLevelKind::L2),
        },
        &funs,
        &s2,
    )
    .await?;

    RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: TrimString("s2-l1".to_string()),
            name: TrimString("scope_test_s2-l1".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: Some(RbumScopeLevelKind::L1),
        },
        &funs,
        &s2,
    )
    .await?;

    RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: TrimString("s2-l0".to_string()),
            name: TrimString("scope_test_s2-l0".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: Some(RbumScopeLevelKind::Root),
        },
        &funs,
        &s2,
    )
    .await?;

    RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: TrimString("s3-l3".to_string()),
            name: TrimString("scope_test_s3-l3".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: Some(RbumScopeLevelKind::L3),
        },
        &funs,
        &s3,
    )
    .await?;

    RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: TrimString("s3-l2".to_string()),
            name: TrimString("scope_test_s3-l2".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: Some(RbumScopeLevelKind::L2),
        },
        &funs,
        &s3,
    )
    .await?;

    RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: TrimString("s3-l1".to_string()),
            name: TrimString("scope_test_s3-l1".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: Some(RbumScopeLevelKind::L1),
        },
        &funs,
        &s3,
    )
    .await?;

    RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: TrimString("s3-l0".to_string()),
            name: TrimString("scope_test_s3-l0".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: Some(RbumScopeLevelKind::Root),
        },
        &funs,
        &s3,
    )
    .await?;

    info!("【test_scope】 : Test");
    let rbums = RbumDomainServ::find_rbums(
        &RbumBasicFilterReq {
            name: Some("scope_test_%".to_string()),
            ..Default::default()
        },
        None,
        None,
        &funs,
        &s0,
    )
    .await?;
    info!(
        "{}:{:#?}",
        s0.own_paths,
        rbums.iter().map(|r| format!("{}:{}", r.name, r.own_paths)).collect::<Vec<String>>()
    );
    assert_eq!(rbums.len(), 7);

    let rbums = RbumDomainServ::find_rbums(
        &RbumBasicFilterReq {
            name: Some("scope_test_".to_string()),
            ..Default::default()
        },
        None,
        None,
        &funs,
        &s1,
    )
    .await?;
    info!(
        "{}:{:#?}",
        s1.own_paths,
        rbums.iter().map(|r| format!("{}:{}", r.name, r.own_paths)).collect::<Vec<String>>()
    );
    assert_eq!(rbums.len(), 10);

    let rbums = RbumDomainServ::find_rbums(
        &RbumBasicFilterReq {
            name: Some("scope_test_".to_string()),
            ..Default::default()
        },
        None,
        None,
        &funs,
        &s2,
    )
    .await?;
    info!(
        "{}:{:#?}",
        s2.own_paths,
        rbums.iter().map(|r| format!("{}:{}", r.name, r.own_paths)).collect::<Vec<String>>()
    );
    assert_eq!(rbums.len(), 13);

    let rbums = RbumDomainServ::find_rbums(
        &RbumBasicFilterReq {
            name: Some("scope_test_".to_string()),
            ..Default::default()
        },
        None,
        None,
        &funs,
        &s3,
    )
    .await?;
    info!(
        "{}:{:#?}",
        s3.own_paths,
        rbums.iter().map(|r| format!("{}:{}", r.name, r.own_paths)).collect::<Vec<String>>()
    );
    assert_eq!(rbums.len(), 16);

    let rbums = RbumDomainServ::find_rbums(
        &RbumBasicFilterReq {
            name: Some("scope_test_".to_string()),
            ..Default::default()
        },
        None,
        None,
        &funs,
        &TardisContext {
            own_paths: "xxx".to_string(),
            ak: "".to_string(),
            owner: "".to_string(),
            roles: vec![],
            groups: vec![],
        },
    )
    .await?;
    info!("xxx:{:#?}", rbums.iter().map(|r| format!("{}:{}", r.name, r.own_paths)).collect::<Vec<String>>());
    assert_eq!(rbums.len(), 5);

    let rbums = RbumDomainServ::find_rbums(
        &RbumBasicFilterReq {
            name: Some("scope_test_".to_string()),
            ..Default::default()
        },
        None,
        None,
        &funs,
        &TardisContext {
            own_paths: format!("{}/x", s3.own_paths),
            ak: "".to_string(),
            owner: "".to_string(),
            roles: vec![],
            groups: vec![],
        },
    )
    .await?;
    info!(
        "{}/x:{:#?}",
        s3.own_paths,
        rbums.iter().map(|r| format!("{}:{}", r.name, r.own_paths)).collect::<Vec<String>>()
    );
    assert_eq!(rbums.len(), 16);

    funs.rollback().await?;

    Ok(())
}
