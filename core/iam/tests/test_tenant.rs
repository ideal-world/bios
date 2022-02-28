use std::env;

use tardis::basic::config::NoneConfig;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm::*;
use tardis::test::test_container::TardisTestContainer;
use tardis::tokio;
use tardis::TardisFuns;
use testcontainers::clients;

use bios_com_iam::iam::console_system::dto::iam_cs_tenant_dto::{IamCsTenantAddReq, IamCsTenantModifyReq};
use bios_com_iam::iam::console_system::serv::iam_cs_tenant_serv;
use bios_com_iam::iam::domain::iam_tenant;
use bios_com_iam::rbum::dto::rbum_item_dto::{RbumItemAddReq, RbumItemModifyReq};
use bios_com_iam::rbum::enumeration::RbumScopeKind;

use crate::test_rbum::CXT_DEFAULT;

mod test_rbum;

#[tokio::test]
async fn test_tenant() -> TardisResult<()> {
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

    test_rbum::prepare().await?;

    let tx = TardisFuns::reldb().conn().begin().await?;

    TardisFuns::reldb().create_table(&iam_tenant::ActiveModel::create_table_statement(TardisFuns::reldb().backend()), &tx).await?;

    // add_iam_tenant
    let id = iam_cs_tenant_serv::add_iam_tenant(
        &IamCsTenantAddReq {
            basic: RbumItemAddReq {
                id: "test1".to_string(),
                rel_app_id: None,
                scope_kind: RbumScopeKind::APP,
                disabled: false,
                name: "测试记录".to_string(),
                uri_part: "".to_string(),
                icon: "".to_string(),
                sort: 0,
                // TODO remove
                rel_rbum_kind_id: iam_tenant::RBUM_KIND_ID.to_string(),
                rel_rbum_domain_id: "iam".to_string(),
            },
        },
        &tx,
        &CXT_DEFAULT,
    )
    .await?;

    // peek_iam_tenant
    let tenant = iam_cs_tenant_serv::peek_iam_tenant(&id, &tx, &CXT_DEFAULT).await?;
    assert_eq!(tenant.basic.id, id);
    assert_eq!(tenant.basic.name, "测试记录");

    // modify_iam_tenant
    iam_cs_tenant_serv::modify_iam_tenant(
        &id,
        &IamCsTenantModifyReq {
            basic: RbumItemModifyReq {
                name: Some("测试记录2".to_string()),
                uri_part: None,
                icon: None,
                sort: None,
                rel_app_id: None,
                scope_kind: None,
                disabled: None,
            },
        },
        &tx,
        &CXT_DEFAULT,
    )
    .await?;

    // get_iam_tenant
    let tenant = iam_cs_tenant_serv::get_iam_tenant(&id, &tx, &CXT_DEFAULT).await?;
    assert_eq!(tenant.basic.id, id);
    assert_eq!(tenant.basic.name, "测试记录2");

    // find_iam_tenants
    let tenants = iam_cs_tenant_serv::find_iam_tenants(1, 2, &tx, &CXT_DEFAULT).await?;
    assert_eq!(tenants.page_number, 1);
    assert_eq!(tenants.page_size, 2);
    assert_eq!(tenants.total_size, 2);

    tx.commit().await?;

    Ok(())
}
