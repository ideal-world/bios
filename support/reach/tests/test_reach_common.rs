use std::sync::OnceLock;

use bios_basic::rbum::dto::rbum_filer_dto::{RbumKindFilterReq, RbumBasicFilterReq};
use bios_basic::rbum::dto::rbum_item_dto::RbumItemAddReq;
use bios_basic::rbum::dto::rbum_kind_dto::RbumKindAddReq;
use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_domain_serv::RbumDomainServ;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemServ;
use bios_basic::rbum::serv::rbum_kind_serv::RbumKindServ;
use bios_basic::{rbum::rbum_config::RbumConfig, test::test_http_client::TestHttpClient};
use bios_reach::consts::{DOMAIN_CODE, REACH_INIT_OWNER, RBUM_KIND_CODE_REACH_MESSAGE};
use tardis::rand;
use tardis::testcontainers::images::{generic::GenericImage, redis::Redis};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    test::test_container::TardisTestContainer,
    testcontainers::{clients::Cli, Container},
    TardisFuns,
};
pub struct Holder<'d> {
    pub db: Container<'d, GenericImage>,
    pub cache: Container<'d, Redis>,
    pub mq: Container<'d, GenericImage>,
}

pub const TEST_OWNER: &str = "test-reach";
pub fn get_test_ctx() -> &'static TardisContext {
    static TEST_CTX: OnceLock<TardisContext> = OnceLock::new();
    TEST_CTX.get_or_init(||{
        TardisContext {
            owner: TEST_OWNER.to_string(),
            ..Default::default()
        }
    })
}

#[allow(dead_code)]
pub async fn init_tardis(docker: &Cli) -> TardisResult<Holder> {
    let reldb_container = TardisTestContainer::postgres_custom(None, docker);
    let port = reldb_container.get_host_port_ipv4(5432);
    let url = format!("postgres://postgres:123456@localhost:{port}/test");
    std::env::set_var("TARDIS_FW.DB.URL", url);
    let redis_container = TardisTestContainer::redis_custom(docker);
    let port = redis_container.get_host_port_ipv4(6379);
    let url = format!("redis://127.0.0.1:{port}/0");
    std::env::set_var("TARDIS_FW.CACHE.URL", url);
    let rabbit_container = TardisTestContainer::rabbit_custom(docker);
    let port = rabbit_container.get_host_port_ipv4(5672);
    let url = format!("amqp://guest:guest@127.0.0.1:{port}/%2f");
    std::env::set_var("TARDIS_FW.MQ.URL", url);
    let holder = Holder {
        db: reldb_container,
        cache: redis_container,
        mq: rabbit_container,
    };
    TardisFuns::init(Some("tests/config")).await?;
    bios_basic::rbum::rbum_initializer::init(DOMAIN_CODE, RbumConfig::default()).await?;
    bios_basic::rbum::rbum_initializer::init("", RbumConfig::default()).await?;
    let web_server = TardisFuns::web_server();
    bios_reach::init(web_server).await?;
    let funs = TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None);
    let ctx = TardisContext {
        owner: REACH_INIT_OWNER.into(),
        ..Default::default()
    };
    let rel_rbum_kind_id = RbumKindServ::find_one_rbum(&RbumKindFilterReq {
        basic: RbumBasicFilterReq {
            code: Some(RBUM_KIND_CODE_REACH_MESSAGE.into()),
            ..Default::default()
        },
        ..Default::default()
    }, &funs, &ctx).await?.expect("fail to find kind").id;
    let rel_rbum_domain_id = RbumDomainServ::find_one_rbum(&RbumBasicFilterReq {
        code: Some(DOMAIN_CODE.into()),
        ..Default::default()
    }, &funs, &ctx).await?.expect("fail to find domain").id;
    RbumItemServ::add_rbum(&mut RbumItemAddReq {
        code: Some("reach-test".into()),
        name: "reach-test".into(),
        scope_level: Some(RbumScopeLevelKind::Root),
        id: Some(TEST_OWNER.into()),
        rel_rbum_kind_id,
        rel_rbum_domain_id,
        disabled: None,
    }, &funs, &ctx).await?;
    web_server.start().await?;
    Ok(holder)
}

#[allow(dead_code)]
pub async fn start_web_server() -> TardisResult<()> {
    TardisFuns::web_server().start().await
}

#[allow(dead_code)]
pub fn get_client(url: &str, ctx: &TardisContext) -> TestHttpClient {
    let mut client: TestHttpClient = TestHttpClient::new(url.into());
    client.set_auth(ctx).unwrap();
    client
}

#[allow(dead_code)]
pub fn wait_for_press() {
    use std::io::*;
    println!("Press any key to continue");
    stdin().read_line(&mut String::new()).expect("fail to read from stdin");
}

#[allow(dead_code)]
pub fn random_string(size: usize) -> String {
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    thread_rng().sample_iter(&Alphanumeric).take(size).map(|x| x as char).collect()
}


