use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::TardisFuns;

use bios_basic::rbum::dto::filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::dto::rbum_domain_dto::{RbumDomainAddReq, RbumDomainModifyReq};
use bios_basic::rbum::enumeration::RbumScopeKind;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_domain_serv::RbumDomainServ;

pub async fn test() -> TardisResult<()> {
    let context = bios_basic::rbum::initializer::get_sys_admin_context().await?;
    let mut tx = TardisFuns::reldb().conn();
    tx.begin().await?;

    // Test Add
    let id = RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            uri_authority: TrimString("mysql_dev".to_string()),
            name: TrimString("Mysql测试集群".to_string()),
            note: Some("...".to_string()),
            icon: Some("...".to_string()),
            sort: None,
            scope_kind: None,
        },
        &tx,
        &context,
    )
    .await?;

    // Test Get
    let rbum = RbumDomainServ::get_rbum(&id, &RbumBasicFilterReq::default(), &tx, &context).await?;
    assert_eq!(rbum.id, id);
    assert_eq!(rbum.uri_authority, "mysql_dev");
    assert_eq!(rbum.name, "Mysql测试集群");
    assert_eq!(rbum.icon, "...");
    assert_eq!(rbum.note, "...");

    // Test Modify
    RbumDomainServ::modify_rbum(
        &id,
        &mut RbumDomainModifyReq {
            uri_authority: None,
            name: None,
            note: None,
            icon: Some(".".to_string()),
            sort: None,
            scope_kind: None,
        },
        &tx,
        &context,
    )
    .await?;

    // Test Find
    let rbums = RbumDomainServ::find_rbums(
        &RbumBasicFilterReq {
            scope_kind: Some(RbumScopeKind::App),
            ..Default::default()
        },
        1,
        10,
        None,
        &tx,
        &context,
    )
    .await?;
    assert_eq!(rbums.page_number, 1);
    assert_eq!(rbums.page_size, 10);
    assert_eq!(rbums.total_size, 1);
    assert_eq!(rbums.records.get(0).unwrap().icon, ".");

    // Test Delete
    RbumDomainServ::delete_rbum(&id, &tx, &context).await?;
    assert!(RbumDomainServ::get_rbum(&id, &RbumBasicFilterReq::default(), &tx, &context).await.is_err());

    tx.rollback().await?;

    Ok(())
}
