use tardis::basic::result::TardisResult;
use tardis::tokio;
use tardis::TardisFuns;

use bios_basic::rbum::dto::filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::dto::rbum_kind_dto::{RbumKindAddReq, RbumKindModifyReq};
use bios_basic::rbum::enumeration::RbumScopeKind;
use bios_basic::rbum::serv::rbum_kind_serv;

mod test_basic;

#[tokio::test]
async fn test_rbum() -> TardisResult<()> {
    let docker = testcontainers::clients::Cli::default();
    let _x = test_basic::init(&docker).await?;
    test_rbum_kind().await
}

async fn test_rbum_kind() -> TardisResult<()> {
    let context = bios_basic::rbum::initializer::get_sys_admin_context().await;
    let mut tx = TardisFuns::reldb().conn();
    tx.begin().await?;

    // add_rbum_kind
    let id = rbum_kind_serv::add_rbum_kind(
        &RbumKindAddReq {
            id: "task".to_string(),
            rel_app_id: None,
            scope_kind: RbumScopeKind::APP,
            name: "代办任务".to_string(),
            note: "".to_string(),
            icon: "".to_string(),
            sort: 0,
            ext_table_name: "todo".to_string(),
        },
        &tx,
        &context,
    )
    .await?;

    // peek_rbum_kind
    let rbum_kind = rbum_kind_serv::peek_rbum_kind(&id, &RbumBasicFilterReq::default(), &tx, &context).await?;
    assert_eq!(rbum_kind.id, id);
    assert_eq!(rbum_kind.name, "代办任务");
    assert_eq!(rbum_kind.ext_table_name, "todo");

    // modify_rbum_kind
    rbum_kind_serv::modify_rbum_kind(
        &id,
        &RbumKindModifyReq {
            rel_app_id: None,
            scope_kind: Some(RbumScopeKind::GLOBAL),
            name: Some("代办事项".to_string()),
            note: Some("代办事项说明".to_string()),
            icon: None,
            sort: None,
            ext_table_name: None,
        },
        &tx,
        &context,
    )
    .await?;

    // get_rbum_kind
    let rbum_kind = rbum_kind_serv::get_rbum_kind(&id, &RbumBasicFilterReq::default(), &tx, &context).await?;
    assert_eq!(rbum_kind.id, id);
    assert_eq!(rbum_kind.scope_kind, RbumScopeKind::GLOBAL);
    assert_eq!(rbum_kind.name, "代办事项");
    assert_eq!(rbum_kind.note, "代办事项说明");

    // find_rbum_kinds
    let rbum_kind = rbum_kind_serv::find_rbum_kinds(&RbumBasicFilterReq::default(), 1, 2, None, &tx, &context).await?;
    assert_eq!(rbum_kind.page_number, 1);
    assert_eq!(rbum_kind.page_size, 2);
    assert_eq!(rbum_kind.total_size, 4);
    let rbum_kind = rbum_kind_serv::find_rbum_kinds(
        &RbumBasicFilterReq {
            rel_cxt_app: true,
            rel_cxt_tenant: true,
            rel_cxt_creator: true,
            rel_cxt_updater: false,
            scope_kind: Some(RbumScopeKind::GLOBAL),
            kind_id: None,
            domain_id: None,
            disabled: false,
        },
        1,
        4,
        Some(true),
        &tx,
        &context,
    )
    .await?;
    println!("================{:#?}", rbum_kind);
    assert_eq!(rbum_kind.page_number, 1);
    assert_eq!(rbum_kind.page_size, 4);
    assert_eq!(rbum_kind.total_size, 4);
    assert!(rbum_kind.records.iter().any(|i| i.name == "代办事项"));
    assert_eq!(rbum_kind.records.get(0).unwrap().creator_id, context.account_id.to_string());
    assert_eq!(rbum_kind.records.get(0).unwrap().updater_id, context.account_id.to_string());
    assert_eq!(rbum_kind.records.get(0).unwrap().rel_app_id, context.app_id.to_string());
    assert_eq!(rbum_kind.records.get(0).unwrap().rel_tenant_id, context.tenant_id.to_string());

    tx.commit().await?;

    Ok(())
}
