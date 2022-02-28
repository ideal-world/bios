use std::env;

use lazy_static::lazy_static;
use tardis::basic::config::NoneConfig;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm::*;
use tardis::TardisFuns;
use tardis::test::test_container::TardisTestContainer;
use tardis::tokio;
use testcontainers::clients;

use bios_com_iam::iam::domain::iam_tenant;
use bios_com_iam::rbum::domain::{rbum_item, rbum_kind};
use bios_com_iam::rbum::dto::filer_dto::RbumBasicFilterReq;
use bios_com_iam::rbum::dto::rbum_item_dto::RbumItemAddReq;
use bios_com_iam::rbum::dto::rbum_kind_dto::{RbumKindAddReq, RbumKindModifyReq};
use bios_com_iam::rbum::enumeration::RbumScopeKind;
use bios_com_iam::rbum::serv::{rbum_item_serv, rbum_kind_serv};

#[tokio::test]
async fn test_rbum() -> TardisResult<()> {
    let docker = clients::Cli::default();

    env::set_var("TARDIS_CACHE.ENABLED", "false");
    env::set_var("TARDIS_MQ.ENABLED", "false");

    let mysql_container = TardisTestContainer::mysql_custom(None, &docker);
    let port = mysql_container.get_host_port(3306).expect("Test port acquisition error");
    let url = format!("mysql://root:123456@localhost:{}/test", port);
    env::set_var("TARDIS_DB.URL", url);

    // let redis_container = TardisTestContainer::redis_custom(&docker);
    // let port = redis_container.get_host_port(6379).expect("Test port acquisition error");
    // let url = format!("redis://127.0.0.1:{}/0", port);
    // env::set_var("TARDIS_CACHE.URL", url);
    //
    // let rabbit_container = TardisTestContainer::rabbit_custom(&docker);
    // let port = rabbit_container.get_host_port(5672).expect("Test port acquisition error");
    // let url = format!("amqp://guest:guest@127.0.0.1:{}/%2f", port);
    // env::set_var("TARDIS_MQ.URL", url);

    env::set_var("RUST_LOG", "debug");
    TardisFuns::init::<NoneConfig>("").await?;

    prepare().await?;

    test_rbum_kind().await
}

lazy_static! {
    pub static ref CXT_DEFAULT: TardisContext = TardisContext {
        app_id: "app1".to_string(),
        tenant_id: "tenant1".to_string(),
        ak: "ak1".to_string(),
        account_id: "account1".to_string(),
        token: "token1".to_string(),
        token_kind: "default".to_string(),
        roles: vec![],
        groups: vec![],
    };
}

pub async fn prepare() -> TardisResult<()> {
    let tx = TardisFuns::reldb().conn().begin().await?;
    let db_kind = TardisFuns::reldb().backend();
    TardisFuns::reldb().create_table(&rbum_kind::ActiveModel::create_table_statement(db_kind), &tx).await?;
    TardisFuns::reldb().create_index(&rbum_kind::ActiveModel::create_index_statement(), &tx).await?;
    TardisFuns::reldb().create_table(&rbum_item::ActiveModel::create_table_statement(db_kind), &tx).await?;
    TardisFuns::reldb().create_index(&rbum_item::ActiveModel::create_index_statement(), &tx).await?;

    rbum_kind_serv::add_rbum_kind(
        &RbumKindAddReq {
            id: iam_tenant::RBUM_KIND_ID.to_string(),
            rel_app_id: None,
            scope_kind: RbumScopeKind::APP,
            name: "Tenant".to_string(),
            note: "".to_string(),
            icon: "".to_string(),
            sort: 0,
            ext_table_name: "iam_tenant".to_string(),
        },
        &tx,
        &CXT_DEFAULT,
    )
    .await?;

    rbum_kind_serv::add_rbum_kind(
        &RbumKindAddReq {
            id: "app".to_string(),
            rel_app_id: None,
            scope_kind: RbumScopeKind::APP,
            name: "App".to_string(),
            note: "".to_string(),
            icon: "".to_string(),
            sort: 0,
            ext_table_name: "iam_app".to_string(),
        },
        &tx,
        &CXT_DEFAULT,
    )
    .await?;

    rbum_kind_serv::add_rbum_kind(
        &RbumKindAddReq {
            id: "account".to_string(),
            rel_app_id: None,
            scope_kind: RbumScopeKind::APP,
            name: "Account".to_string(),
            note: "".to_string(),
            icon: "".to_string(),
            sort: 0,
            ext_table_name: "iam_account".to_string(),
        },
        &tx,
        &CXT_DEFAULT,
    )
    .await?;

    rbum_item_serv::add_rbum_item(
        "account1","account",
        &RbumItemAddReq {
            scope_kind: RbumScopeKind::APP,
            disabled: false,
            name: "钢铁侠".to_string(),
            uri_part: "".to_string(),
            icon: "".to_string(),
            sort: 0,
            rel_rbum_domain_id: "iam".to_string(),
        },None,
        &tx,
        &CXT_DEFAULT,
    )
    .await?;

    rbum_item_serv::add_rbum_item(
        "tenant1",iam_tenant::RBUM_KIND_ID,
        &RbumItemAddReq {
            scope_kind: RbumScopeKind::APP,
            disabled: false,
            name: "默认租户".to_string(),
            uri_part: "".to_string(),
            icon: "".to_string(),
            sort: 0,
            rel_rbum_domain_id: "iam".to_string(),
        },None,
        &tx,
        &CXT_DEFAULT,
    )
    .await?;

    rbum_item_serv::add_rbum_item(
        "app1","app",
        &RbumItemAddReq {
            scope_kind: RbumScopeKind::APP,
            disabled: false,
            name: "IAM应用".to_string(),
            uri_part: "".to_string(),
            icon: "".to_string(),
            sort: 0,
            rel_rbum_domain_id: "iam".to_string(),
        },None,
        &tx,
        &CXT_DEFAULT,
    )
    .await?;

    tx.commit().await?;
    Ok(())
}

async fn test_rbum_kind() -> TardisResult<()> {
    let tx = TardisFuns::reldb().conn().begin().await?;

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
        &CXT_DEFAULT,
    )
    .await?;

    // peek_rbum_kind
    let rbum_kind = rbum_kind_serv::peek_rbum_kind(&id, &RbumBasicFilterReq::default(), &tx, &CXT_DEFAULT).await?;
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
        &CXT_DEFAULT,
    )
    .await?;

    // get_rbum_kind
    let rbum_kind = rbum_kind_serv::get_rbum_kind(&id, &RbumBasicFilterReq::default(), &tx, &CXT_DEFAULT).await?;
    assert_eq!(rbum_kind.id, id);
    assert_eq!(rbum_kind.scope_kind, RbumScopeKind::GLOBAL);
    assert_eq!(rbum_kind.name, "代办事项");
    assert_eq!(rbum_kind.note, "代办事项说明");

    // find_rbum_kinds
    let rbum_kind = rbum_kind_serv::find_rbum_kinds(&RbumBasicFilterReq::default(), 1, 2, &tx, &CXT_DEFAULT).await?;
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
        2,
        &tx,
        &CXT_DEFAULT,
    )
    .await?;
    assert_eq!(rbum_kind.page_number, 1);
    assert_eq!(rbum_kind.page_size, 2);
    assert_eq!(rbum_kind.total_size, 1);
    assert_eq!(rbum_kind.records.get(0).unwrap().name, "代办事项");
    assert_eq!(rbum_kind.records.get(0).unwrap().creator_id, "account1");
    assert_eq!(rbum_kind.records.get(0).unwrap().creator_name, "钢铁侠");
    assert_eq!(rbum_kind.records.get(0).unwrap().updater_id, "account1");
    assert_eq!(rbum_kind.records.get(0).unwrap().updater_name, "钢铁侠");
    assert_eq!(rbum_kind.records.get(0).unwrap().rel_app_id, "app1");
    assert_eq!(rbum_kind.records.get(0).unwrap().rel_app_name, "IAM应用");
    assert_eq!(rbum_kind.records.get(0).unwrap().rel_tenant_id, "tenant1");
    assert_eq!(rbum_kind.records.get(0).unwrap().rel_tenant_name, "默认租户");

    tx.commit().await?;

    Ok(())
}
