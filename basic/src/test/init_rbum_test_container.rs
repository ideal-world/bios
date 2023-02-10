use std::env;

use tardis::basic::result::TardisResult;
use tardis::test::test_container::TardisTestContainer;
use tardis::testcontainers::clients::Cli;
use tardis::testcontainers::images::generic::GenericImage;
use tardis::testcontainers::images::redis::Redis;
use tardis::testcontainers::Container;
use tardis::TardisFuns;

pub struct LifeHold<'a> {
    pub reldb: Container<'a, GenericImage>,
    pub redis: Container<'a, Redis>,
    pub rabbit: Container<'a, GenericImage>,
}

pub async fn init(docker: &Cli, sql_init_path: Option<String>) -> TardisResult<LifeHold<'_>> {
    // let reldb_container = TardisTestContainer::mysql_custom(None, docker);
    // let port = reldb_container.get_host_port_ipv4(3306);
    // let url = format!("mysql://root:123456@localhost:{}/test", port);
    let reldb_container = TardisTestContainer::postgres_custom(sql_init_path.as_deref(), docker);
    let port = reldb_container.get_host_port_ipv4(5432);
    let url = format!("postgres://postgres:123456@localhost:{port}/test");
    env::set_var("TARDIS_FW.DB.URL", url);

    let redis_container = TardisTestContainer::redis_custom(docker);
    let port = redis_container.get_host_port_ipv4(6379);
    let url = format!("redis://127.0.0.1:{port}/0");
    env::set_var("TARDIS_FW.CACHE.URL", url);

    let rabbit_container = TardisTestContainer::rabbit_custom(docker);
    let port = rabbit_container.get_host_port_ipv4(5672);
    let url = format!("amqp://guest:guest@127.0.0.1:{port}/%2f");
    env::set_var("TARDIS_FW.MQ.URL", url);

    TardisFuns::init("tests/config").await?;

    Ok(LifeHold {
        reldb: reldb_container,
        redis: redis_container,
        rabbit: rabbit_container,
    })
}
