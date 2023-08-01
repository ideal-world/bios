use bios_basic::{rbum::rbum_config::RbumConfig, test::test_http_client::TestHttpClient};
use bios_reach::consts::DOMAIN_CODE;
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
    let web_server = TardisFuns::web_server();
    bios_reach::init(web_server).await?;
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
