use std::env;

use tardis::basic::config::NoneConfig;
use tardis::basic::result::TardisResult;
use tardis::test::test_container::TardisTestContainer;
use tardis::TardisFuns;
use testcontainers::clients::Cli;
use testcontainers::images::generic::GenericImage;
use testcontainers::images::redis::Redis;
use testcontainers::Container;

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

    bios_iam::initializer::init_db().await?;

    Ok(LifeHold {
        mysql: mysql_container,
        redis: redis_container,
    })
}
