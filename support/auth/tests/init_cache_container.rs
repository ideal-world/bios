use std::env;

use tardis::basic::result::TardisResult;
use tardis::test::test_container::TardisTestContainer;
use tardis::testcontainers::clients::Cli;
use tardis::testcontainers::images::redis::Redis;
use tardis::testcontainers::Container;
use tardis::TardisFuns;

pub struct LifeHold<'a> {
    pub redis: Container<'a, Redis>,
}

pub async fn init(docker: &Cli) -> TardisResult<LifeHold<'_>> {
    let redis_container = TardisTestContainer::redis_custom(docker);
    let port = redis_container.get_host_port_ipv4(6379);
    let url = format!("redis://127.0.0.1:{port}/0",);
    env::set_var("TARDIS_FW.CACHE.URL", url);

    TardisFuns::init("tests/config").await?;

    Ok(LifeHold { redis: redis_container })
}
