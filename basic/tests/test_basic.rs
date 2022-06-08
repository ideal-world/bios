use std::env;

use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::test::test_container::TardisTestContainer;
use tardis::testcontainers::clients::Cli;
use tardis::testcontainers::images::generic::GenericImage;
use tardis::testcontainers::images::redis::Redis;
use tardis::testcontainers::Container;
use tardis::TardisFuns;

use bios_basic::rbum::dto::rbum_domain_dto::RbumDomainAddReq;
use bios_basic::rbum::dto::rbum_item_dto::RbumItemAddReq;
use bios_basic::rbum::dto::rbum_kind_dto::RbumKindAddReq;
use bios_basic::rbum::rbum_config::RbumConfig;
use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_domain_serv::RbumDomainServ;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemServ;
use bios_basic::rbum::serv::rbum_kind_serv::RbumKindServ;

const RBUM_KIND_SCHEME_IAM_TENANT: &str = "iam-tenant";
const RBUM_KIND_SCHEME_IAM_APP: &str = "iam-app";
const RBUM_KIND_SCHEME_IAM_ACCOUNT: &str = "iam-account";
const RBUM_ITEM_NAME_DEFAULT_TENANT: &str = "system";
const RBUM_ITEM_NAME_DEFAULT_APP: &str = "iam";
const RBUM_ITEM_NAME_DEFAULT_ACCOUNT: &str = "sys_admin";

pub struct LifeHold<'a> {
    pub mysql: Container<'a, GenericImage>,
    pub redis: Container<'a, Redis>,
    pub rabbit: Container<'a, GenericImage>,
}

pub async fn init(docker: &Cli) -> TardisResult<LifeHold<'_>> {
    let mysql_container = TardisTestContainer::mysql_custom(None, docker);
    let port = mysql_container.get_host_port_ipv4(3306);
    let url = format!("mysql://root:123456@localhost:{}/test", port);
    env::set_var("TARDIS_FW.DB.URL", url);

    let redis_container = TardisTestContainer::redis_custom(docker);
    let port = redis_container.get_host_port_ipv4(6379);
    let url = format!("redis://127.0.0.1:{}/0", port);
    env::set_var("TARDIS_FW.CACHE.URL", url);

    let rabbit_container = TardisTestContainer::rabbit_custom(docker);
    let port = rabbit_container.get_host_port_ipv4(5672);
    let url = format!("amqp://guest:guest@127.0.0.1:{}/%2f", port);
    env::set_var("TARDIS_FW.MQ.URL", url);

    env::set_var("RUST_LOG", "debug");
    TardisFuns::init("tests/config").await?;

    bios_basic::rbum::rbum_initializer::init("", RbumConfig::default()).await?;

    Ok(LifeHold {
        mysql: mysql_container,
        redis: redis_container,
        rabbit: rabbit_container,
    })
}

pub async fn init_test_data() -> TardisResult<TardisContext> {
    let mut funs = TardisFuns::inst_with_db_conn("".to_string());
    funs.begin().await?;

    let cxt = TardisContext {
        own_paths: "".to_string(),
        owner: "".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
    };

    let kind_tenant_id = RbumKindServ::add_rbum(
        &mut RbumKindAddReq {
            code: TrimString(RBUM_KIND_SCHEME_IAM_TENANT.to_string()),
            name: TrimString(RBUM_KIND_SCHEME_IAM_TENANT.to_string()),
            note: None,
            icon: None,
            sort: None,
            ext_table_name: Some(RBUM_KIND_SCHEME_IAM_TENANT.to_string().to_lowercase()),
            scope_level: Some(RbumScopeLevelKind::Root),
        },
        &funs,
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
            scope_level: Some(RbumScopeLevelKind::Root),
        },
        &funs,
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
            scope_level: Some(RbumScopeLevelKind::Root),
        },
        &funs,
        &cxt,
    )
    .await?;

    let domain_iam_id = RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: TrimString("iam".to_string()),
            name: TrimString("IAM".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: Some(RbumScopeLevelKind::Root),
        },
        &funs,
        &cxt,
    )
    .await?;

    let tenant_id = RbumItemServ::add_rbum(
        &mut RbumItemAddReq {
            code: None,
            name: TrimString(RBUM_ITEM_NAME_DEFAULT_TENANT.to_string()),
            disabled: None,
            rel_rbum_kind_id: kind_tenant_id.clone(),
            rel_rbum_domain_id: domain_iam_id.clone(),
            scope_level: Some(RbumScopeLevelKind::L2),
            id: Some(TrimString(TardisFuns::field.nanoid_len(4))),
        },
        &funs,
        &cxt,
    )
    .await?;

    let app_id = RbumItemServ::add_rbum(
        &mut RbumItemAddReq {
            code: None,
            name: TrimString(RBUM_ITEM_NAME_DEFAULT_APP.to_string()),
            disabled: None,
            rel_rbum_kind_id: kind_app_id.clone(),
            rel_rbum_domain_id: domain_iam_id.clone(),
            scope_level: Some(RbumScopeLevelKind::L2),
            id: Some(TrimString(TardisFuns::field.nanoid_len(4))),
        },
        &funs,
        &cxt,
    )
    .await?;

    let account_id = RbumItemServ::add_rbum(
        &mut RbumItemAddReq {
            code: None,
            name: TrimString(RBUM_ITEM_NAME_DEFAULT_ACCOUNT.to_string()),
            disabled: None,
            rel_rbum_kind_id: kind_account_id.clone(),
            rel_rbum_domain_id: domain_iam_id.clone(),
            scope_level: Some(RbumScopeLevelKind::Root),
            id: None,
        },
        &funs,
        &cxt,
    )
    .await?;

    funs.commit().await?;
    Ok(TardisContext {
        own_paths: format!("{}/{}", tenant_id, app_id),
        owner: account_id.to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
    })
}
