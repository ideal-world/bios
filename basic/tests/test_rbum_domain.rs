use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::TardisFuns;

use bios_basic::rbum::dto::rbum_domain_dto::{RbumDomainAddReq, RbumDomainModifyReq};
use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_domain_serv::RbumDomainServ;

pub async fn test(context: &TardisContext) -> TardisResult<()> {
    let mut funs = TardisFuns::inst_with_db_conn("");
    funs.begin().await?;

    info!("【test_rbum_domin】 : Test Add : RbumDomainServ::add_rbum");
    let id = RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: TrimString("mysql_dev".to_string()),
            name: TrimString("Mysql测试集群".to_string()),
            note: Some("...".to_string()),
            icon: Some("...".to_string()),
            sort: None,
            scope_level: RbumScopeLevelKind::L2,
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_rbum_domin】 : Test Get : RbumDomainServ::get_rbum");
    let rbum = RbumDomainServ::get_rbum(&id, &RbumBasicFilterReq::default(), &funs, context).await?;
    assert_eq!(rbum.id, id);
    assert_eq!(rbum.code, "mysql_dev");
    assert_eq!(rbum.name, "Mysql测试集群");
    assert_eq!(rbum.icon, "...");
    assert_eq!(rbum.note, "...");

    info!("【test_rbum_domin】 : Test Modify : RbumDomainServ::modify_rbum");
    RbumDomainServ::modify_rbum(
        &id,
        &mut RbumDomainModifyReq {
            code: None,
            name: None,
            note: None,
            icon: Some(".".to_string()),
            sort: None,
            scope_level: None,
        },
        &funs,
        context,
    )
    .await?;

    info!("【test_rbum_domin】 : Test Find : RbumDomainServ::paginate_rbums");
    let rbums = RbumDomainServ::paginate_rbums(
        &RbumBasicFilterReq {
            scope_level: Some(RbumScopeLevelKind::L2),
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
    assert_eq!(rbums.records.get(0).unwrap().icon, ".");

    info!("【test_rbum_domin】 : Test Delete : RbumDomainServ::delete_rbum");
    RbumDomainServ::delete_rbum(&id, &funs, context).await?;
    assert!(RbumDomainServ::get_rbum(&id, &RbumBasicFilterReq::default(), &funs, context).await.is_err());

    funs.rollback().await?;

    Ok(())
}
