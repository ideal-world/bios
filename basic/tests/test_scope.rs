use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::TardisFuns;

use bios_basic::rbum::constants;
use bios_basic::rbum::dto::filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::dto::rbum_domain_dto::RbumDomainAddReq;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_domain_serv::RbumDomainServ;

pub async fn test() -> TardisResult<()> {
    let mut tx = TardisFuns::reldb().conn();
    tx.begin().await?;

    info!("【test_scope】 : Prepare");
    let s0 = TardisContext {
        scope_ids: "".to_string(),
        ak: "".to_string(),
        account_id: "".to_string(),
        token: "".to_string(),
        token_kind: "".to_string(),
        roles: vec![],
        groups: vec![],
    };

    let s1 = TardisContext {
        scope_ids: TardisFuns::field.nanoid_len(constants::RBUM_SCOPE_L1_LEN),
        ak: "".to_string(),
        account_id: "".to_string(),
        token: "".to_string(),
        token_kind: "".to_string(),
        roles: vec![],
        groups: vec![],
    };

    let s2 = TardisContext {
        scope_ids: format!("{}{}", s1.scope_ids, TardisFuns::field.nanoid_len(constants::RBUM_SCOPE_L2_LEN)),
        ak: "".to_string(),
        account_id: "".to_string(),
        token: "".to_string(),
        token_kind: "".to_string(),
        roles: vec![],
        groups: vec![],
    };

    let s3 = TardisContext {
        scope_ids: format!("{}{}", s2.scope_ids, TardisFuns::field.nanoid_len(constants::RBUM_SCOPE_L3_LEN)),
        ak: "".to_string(),
        account_id: "".to_string(),
        token: "".to_string(),
        token_kind: "".to_string(),
        roles: vec![],
        groups: vec![],
    };

    RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            uri_authority: TrimString("s0_l3".to_string()),
            name: TrimString("scope_test_s0_l3".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: 3,
        },
        &tx,
        &s0,
    )
    .await?;

    RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            uri_authority: TrimString("s0_l2".to_string()),
            name: TrimString("scope_test_s0_l2".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: 2,
        },
        &tx,
        &s0,
    )
    .await?;

    RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            uri_authority: TrimString("s0_l1".to_string()),
            name: TrimString("scope_test_s0_l1".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: 1,
        },
        &tx,
        &s0,
    )
    .await?;

    RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            uri_authority: TrimString("s0_l0".to_string()),
            name: TrimString("scope_test_s0_l0".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: 0,
        },
        &tx,
        &s0,
    )
    .await?;

    RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            uri_authority: TrimString("s1_l3".to_string()),
            name: TrimString("scope_test_s1_l3".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: 3,
        },
        &tx,
        &s1,
    )
    .await?;

    RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            uri_authority: TrimString("s1_l2".to_string()),
            name: TrimString("scope_test_s1_l2".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: 2,
        },
        &tx,
        &s1,
    )
    .await?;

    RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            uri_authority: TrimString("s1_l1".to_string()),
            name: TrimString("scope_test_s1_l1".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: 1,
        },
        &tx,
        &s1,
    )
    .await?;

    RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            uri_authority: TrimString("s1_l0".to_string()),
            name: TrimString("scope_test_s1_l0".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: 0,
        },
        &tx,
        &s1,
    )
    .await?;

    RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            uri_authority: TrimString("s2_l3".to_string()),
            name: TrimString("scope_test_s2_l3".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: 3,
        },
        &tx,
        &s2,
    )
    .await?;

    RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            uri_authority: TrimString("s2_l2".to_string()),
            name: TrimString("scope_test_s2_l2".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: 2,
        },
        &tx,
        &s2,
    )
    .await?;

    RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            uri_authority: TrimString("s2_l1".to_string()),
            name: TrimString("scope_test_s2_l1".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: 1,
        },
        &tx,
        &s2,
    )
    .await?;

    RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            uri_authority: TrimString("s2_l0".to_string()),
            name: TrimString("scope_test_s2_l0".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: 0,
        },
        &tx,
        &s2,
    )
    .await?;

    RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            uri_authority: TrimString("s3_l3".to_string()),
            name: TrimString("scope_test_s3_l3".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: 3,
        },
        &tx,
        &s3,
    )
    .await?;

    RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            uri_authority: TrimString("s3_l2".to_string()),
            name: TrimString("scope_test_s3_l2".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: 2,
        },
        &tx,
        &s3,
    )
    .await?;

    RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            uri_authority: TrimString("s3_l1".to_string()),
            name: TrimString("scope_test_s3_l1".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: 1,
        },
        &tx,
        &s3,
    )
    .await?;

    RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            uri_authority: TrimString("s3_l0".to_string()),
            name: TrimString("scope_test_s3_l0".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: 0,
        },
        &tx,
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
        &tx,
        &s0,
    )
    .await?;
    info!("{}:{:?}", s0.scope_ids, rbums.iter().map(|r| r.name.as_str()).collect::<Vec<&str>>());
    assert_eq!(rbums.len(), 16);

    let rbums = RbumDomainServ::find_rbums(
        &RbumBasicFilterReq {
            name: Some("scope_test_%".to_string()),
            ..Default::default()
        },
        None,
        None,
        &tx,
        &s1,
    )
    .await?;
    info!("{}:{:?}", s1.scope_ids, rbums.iter().map(|r| r.name.as_str()).collect::<Vec<&str>>());
    assert_eq!(rbums.len(), 13);

    let rbums = RbumDomainServ::find_rbums(
        &RbumBasicFilterReq {
            name: Some("scope_test_%".to_string()),
            ..Default::default()
        },
        None,
        None,
        &tx,
        &s2,
    )
    .await?;
    info!("{}:{:?}", s2.scope_ids, rbums.iter().map(|r| r.name.as_str()).collect::<Vec<&str>>());
    assert_eq!(rbums.len(), 11);

    let rbums = RbumDomainServ::find_rbums(
        &RbumBasicFilterReq {
            name: Some("scope_test_%".to_string()),
            ..Default::default()
        },
        None,
        None,
        &tx,
        &s3,
    )
    .await?;
    info!("{}:{:?}", s3.scope_ids, rbums.iter().map(|r| r.name.as_str()).collect::<Vec<&str>>());
    assert_eq!(rbums.len(), 10);

    let rbums = RbumDomainServ::find_rbums(
        &RbumBasicFilterReq {
            name: Some("scope_test_%".to_string()),
            ..Default::default()
        },
        None,
        None,
        &tx,
        &TardisContext {
            scope_ids: "xxx".to_string(),
            ak: "".to_string(),
            account_id: "".to_string(),
            token: "".to_string(),
            token_kind: "".to_string(),
            roles: vec![],
            groups: vec![],
        },
    )
    .await?;
    info!("xxx:{:?}", rbums.iter().map(|r| r.name.as_str()).collect::<Vec<&str>>());
    assert_eq!(rbums.len(), 4);

    let rbums = RbumDomainServ::find_rbums(
        &RbumBasicFilterReq {
            name: Some("scope_test_%".to_string()),
            ..Default::default()
        },
        None,
        None,
        &tx,
        &TardisContext {
            scope_ids: format!("{}x", s3.scope_ids),
            ak: "".to_string(),
            account_id: "".to_string(),
            token: "".to_string(),
            token_kind: "".to_string(),
            roles: vec![],
            groups: vec![],
        },
    )
    .await?;
    info!("{}x:{:?}", s3.scope_ids, rbums.iter().map(|r| r.name.as_str()).collect::<Vec<&str>>());
    assert_eq!(rbums.len(), 10);

    tx.rollback().await?;

    Ok(())
}
