use std::env;

use tardis::basic::config::NoneConfig;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::test::test_container::TardisTestContainer;
use tardis::tokio;
use tardis::TardisFuns;
use testcontainers::clients;

use bios_com_iam::domain::{rbum_item, rbum_kind};
use bios_com_iam::dto::rbum_kind_dto::RbumKindAddReq;
use bios_com_iam::enumeration::RbumScopeKind;
use bios_com_iam::service::rbum_kind_serv;

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

    test_rbum_kind().await
}

async fn test_rbum_kind() -> TardisResult<()> {
    TardisFuns::reldb().create_table_from_entity(rbum_kind::Entity).await?;
    TardisFuns::reldb().create_table_from_entity(rbum_item::Entity).await?;

    let cxt = TardisContext {
        app_id: "a1".to_string(),
        tenant_id: "t1".to_string(),
        ak: "ak1".to_string(),
        account_id: "acc1".to_string(),
        token: "token1".to_string(),
        token_kind: "default".to_string(),
        roles: vec![],
        groups: vec![],
    };

    let id = rbum_kind_serv::add_rbum_kind(
        &RbumKindAddReq {
            code: "tenant".to_string(),
            name: "Tenant".to_string(),
            note: "".to_string(),
            icon: "".to_string(),
            sort: 0,
            ext_table_name: "iam_tenant_r".to_string(),
        },
        &cxt,
    )
    .await?;
    let rbum_kind = rbum_kind_serv::get_rbum_kind(&id, &cxt).await?;
    assert_eq!(rbum_kind.id, id);
    assert_eq!(rbum_kind.name, "Tenant");
    Ok(())
}
