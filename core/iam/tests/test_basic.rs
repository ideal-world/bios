use std::env;

use tardis::basic::result::TardisResult;
use tardis::test::test_container::TardisTestContainer;
use tardis::testcontainers::clients::Cli;
use tardis::testcontainers::images::generic::GenericImage;
use tardis::testcontainers::images::redis::Redis;
use tardis::testcontainers::Container;
use tardis::TardisFuns;

pub struct LifeHold<'a> {
    pub mysql: Container<'a, GenericImage>,
    pub redis: Container<'a, Redis>,
}

pub async fn init(docker: &'_ Cli) -> TardisResult<LifeHold<'_>> {
    let mysql_container = TardisTestContainer::mysql_custom(None, docker);
    let port = mysql_container.get_host_port(3306);
    let url = format!("mysql://root:123456@localhost:{}/test", port);
    env::set_var("TARDIS_FW.DB.URL", url);

    let redis_container = TardisTestContainer::redis_custom(docker);
    let port = redis_container.get_host_port(6379);
    let url = format!("redis://127.0.0.1:{}/0", port);
    env::set_var("TARDIS_FW.CACHE.URL", url);
    //
    // let rabbit_container = TardisTestContainer::rabbit_custom(docker);
    // let port = rabbit_container.get_host_port(5672).expect("Test port acquisition error");
    // let url = format!("amqp://guest:guest@127.0.0.1:{}/%2f", port);
    // env::set_var("TARDIS_FW.MQ.URL", url);

    env::set_var("RUST_LOG", "debug");
    TardisFuns::init("tests/config").await?;

    bios_iam::iam_initializer::init_db().await?;

    Ok(LifeHold {
        mysql: mysql_container,
        redis: redis_container,
    })
}
