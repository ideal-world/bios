use std::env;

use tardis::basic::config::NoneConfig;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::test::test_container::TardisTestContainer;
use tardis::TardisFuns;
use testcontainers::clients::Cli;
use testcontainers::images::generic::GenericImage;
use testcontainers::images::redis::Redis;
use testcontainers::Container;

use bios_basic::rbum::dto::rbum_domain_dto::RbumDomainAddReq;
use bios_basic::rbum::dto::rbum_item_dto::RbumItemAddReq;
use bios_basic::rbum::dto::rbum_kind_dto::RbumKindAddReq;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_domain_serv::RbumDomainServ;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemServ;
use bios_basic::rbum::serv::rbum_kind_serv::RbumKindServ;

const RBUM_KIND_SCHEME_IAM_TENANT: &str = "iam_tenant";
const RBUM_KIND_SCHEME_IAM_APP: &str = "iam_app";
const RBUM_KIND_SCHEME_IAM_ACCOUNT: &str = "iam_account";
const RBUM_ITEM_NAME_DEFAULT_TENANT: &str = "system";
const RBUM_ITEM_NAME_DEFAULT_APP: &str = "iam";
const RBUM_ITEM_NAME_DEFAULT_ACCOUNT: &str = "sys_admin";

pub struct LifeHold<'a> {
    pub mysql: Container<'a, Cli, GenericImage>,
    pub redis: Container<'a, Cli, Redis>,
}

pub async fn init<'a>(docker: &'a Cli) -> TardisResult<LifeHold<'a>> {
    env::set_var("TARDIS_CACHE.ENABLED", "false");
    env::set_var("TARDIS_MQ.ENABLED", "false");

    let mysql_container = TardisTestContainer::mysql_custom(None, &docker);
    let port = mysql_container.get_host_port(3306).expect("Test port acquisition error");
    let url = format!("mysql://root:123456@localhost:{}/test", port);
    env::set_var("TARDIS_DB.URL", url);

    let redis_container = TardisTestContainer::redis_custom(&docker);
    let port = redis_container.get_host_port(6379).expect("Test port acquisition error");
    let url = format!("redis://127.0.0.1:{}/0", port);
    env::set_var("TARDIS_CACHE.URL", url);
    //
    // let rabbit_container = TardisTestContainer::rabbit_custom(&docker);
    // let port = rabbit_container.get_host_port(5672).expect("Test port acquisition error");
    // let url = format!("amqp://guest:guest@127.0.0.1:{}/%2f", port);
    // env::set_var("TARDIS_MQ.URL", url);

    env::set_var("RUST_LOG", "debug");
    TardisFuns::init::<NoneConfig>("").await?;

    bios_basic::rbum::rbum_initializer::init_db().await?;

    Ok(LifeHold {
        mysql: mysql_container,
        redis: redis_container,
    })
}

pub async fn init_test_data() -> TardisResult<TardisContext> {
    let mut tx = TardisFuns::reldb().conn();
    tx.begin().await?;

    let cxt = TardisContext {
        own_paths: "".to_string(),
        ak: "".to_string(),
        token: "".to_string(),
        token_kind: "".to_string(),
        roles: vec![],
        groups: vec![],
        account_id: "".to_string(),
    };

    let kind_tenant_id = RbumKindServ::add_rbum(
        &mut RbumKindAddReq {
            code: TrimString(RBUM_KIND_SCHEME_IAM_TENANT.to_string()),
            name: TrimString(RBUM_KIND_SCHEME_IAM_TENANT.to_string()),
            note: None,
            icon: None,
            sort: None,
            ext_table_name: Some(RBUM_KIND_SCHEME_IAM_TENANT.to_string().to_lowercase()),
            scope_level: 0,
        },
        &tx,
        &cxt,
    )
    .await?;

    let kind_app_id = RbumKindServ::add_rbum(
        &mut RbumKindAddReq {
            code: TrimString(RBUM_KIND_SCHEME_IAM_APP.to_string()),
            name: TrimString(RBUM_KIND_SCHEME_IAM_APP.to_string()),
            note: None,
            icon: None,
            sort: None,
            ext_table_name: Some(RBUM_KIND_SCHEME_IAM_APP.to_string().to_lowercase()),
            scope_level: 0,
        },
        &tx,
        &cxt,
    )
    .await?;

    let kind_account_id = RbumKindServ::add_rbum(
        &mut RbumKindAddReq {
            code: TrimString(RBUM_KIND_SCHEME_IAM_ACCOUNT.to_string()),
            name: TrimString(RBUM_KIND_SCHEME_IAM_ACCOUNT.to_string()),
            note: None,
            icon: None,
            sort: None,
            ext_table_name: Some(RBUM_KIND_SCHEME_IAM_ACCOUNT.to_string().to_lowercase()),
            scope_level: 0,
        },
        &tx,
        &cxt,
    )
    .await?;

    let domain_iam_id = RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: TrimString(bios_basic::Components::Iam.to_string()),
            name: TrimString(bios_basic::Components::Iam.to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: 0,
        },
        &tx,
        &cxt,
    )
    .await?;

    let tenant_id = RbumItemServ::add_rbum(
        &mut RbumItemAddReq {
            code: None,
            name: TrimString(RBUM_ITEM_NAME_DEFAULT_TENANT.to_string()),
            icon: None,
            sort: None,
            disabled: None,
            rel_rbum_kind_id: kind_tenant_id.clone(),
            rel_rbum_domain_id: domain_iam_id.clone(),
            scope_level: 2,
            id: Some(TrimString(TardisFuns::field.nanoid_len(4))),
        },
        &tx,
        &cxt,
    )
    .await?;

    let app_id = RbumItemServ::add_rbum(
        &mut RbumItemAddReq {
            code: None,
            name: TrimString(RBUM_ITEM_NAME_DEFAULT_APP.to_string()),
            icon: None,
            sort: None,
            disabled: None,
            rel_rbum_kind_id: kind_app_id.clone(),
            rel_rbum_domain_id: domain_iam_id.clone(),
            scope_level: 2,
            id: Some(TrimString(TardisFuns::field.nanoid_len(4))),
        },
        &tx,
        &cxt,
    )
    .await?;

    let account_id = RbumItemServ::add_rbum(
        &mut RbumItemAddReq {
            code: None,
            name: TrimString(RBUM_ITEM_NAME_DEFAULT_ACCOUNT.to_string()),
            icon: None,
            sort: None,
            disabled: None,
            rel_rbum_kind_id: kind_account_id.clone(),
            rel_rbum_domain_id: domain_iam_id.clone(),
            scope_level: 0,
            id: None,
        },
        &tx,
        &cxt,
    )
    .await?;

    tx.commit().await?;
    Ok(TardisContext {
        own_paths: format!("{}/{}", tenant_id, app_id),
        ak: "".to_string(),
        token: "".to_string(),
        token_kind: "".to_string(),
        roles: vec![],
        groups: vec![],
        account_id: account_id.to_string(),
    })
}
