use std::env;

use tardis::basic::result::TardisResult;
use tardis::test::test_container::TardisTestContainer;
use tardis::testcontainers::ContainerAsync;
use tardis::TardisFuns;
use testcontainers_modules::redis::Redis;

pub struct LifeHold {
    pub redis: ContainerAsync<Redis>,
}

pub async fn init() -> TardisResult<LifeHold> {
    let redis_container = TardisTestContainer::redis_custom().await?;
    let port = redis_container.get_host_port_ipv4(6379).await?;
    let url = format!("redis://127.0.0.1:{port}/0",);
    env::set_var("TARDIS_FW.CACHE.URL", url);

    TardisFuns::init(Some("tests/config")).await?;

    Ok(LifeHold { redis: redis_container })
}
