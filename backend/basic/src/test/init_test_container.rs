//! Init test container
use std::env;

use tardis::basic::result::TardisResult;
use tardis::test::test_container::TardisTestContainer;
use tardis::testcontainers::core::Mount;
use tardis::testcontainers::runners::AsyncRunner;
use tardis::testcontainers::{ContainerAsync, ImageExt};
use tardis::TardisFuns;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::rabbitmq::RabbitMq;
use testcontainers_modules::redis::Redis;

pub struct LifeHold {
    pub reldb: ContainerAsync<Postgres>,
    pub redis: ContainerAsync<Redis>,
    pub rabbit: ContainerAsync<RabbitMq>,
}

pub async fn postgres_custom(init_script_path: Option<&str>) -> TardisResult<ContainerAsync<Postgres>> {
    let mut postgres = Postgres::default().with_tag("alpine").with_env_var("POSTGRES_PASSWORD", "123456").with_env_var("POSTGRES_DB", "test");
    postgres = if let Some(init_script_path) = init_script_path {
        let path = env::current_dir()
            .expect("[Tardis.Test_Container] Current path get error")
            .join(std::path::Path::new(init_script_path))
            .to_str()
            .unwrap_or_else(|| panic!("[Tardis.Test_Container] Script Path [{init_script_path}] get error"))
            .to_string();
        postgres.with_mount(Mount::volume_mount(path, "/docker-entrypoint-initdb.d/"))
    } else {
        postgres
    };
    let postgres = postgres.start().await?;
    Ok(postgres)
}

pub async fn init(sql_init_path: Option<String>) -> TardisResult<LifeHold> {
    let reldb_container = postgres_custom(sql_init_path.as_deref()).await?;
    let port = reldb_container.get_host_port_ipv4(5432).await?;
    let url = format!("postgres://postgres:123456@127.0.0.1:{port}/test");
    env::set_var("TARDIS_FW.DB.URL", url);

    let redis_container = TardisTestContainer::redis_custom().await?;
    let port = redis_container.get_host_port_ipv4(6379).await?;
    let url = format!("redis://127.0.0.1:{port}/0");
    env::set_var("TARDIS_FW.CACHE.URL", url);

    // TODO remove
    let rabbit_container = TardisTestContainer::rabbit_custom().await?;
    let port = rabbit_container.get_host_port_ipv4(5672).await?;
    let url = format!("amqp://guest:guest@127.0.0.1:{port}/%2f");
    env::set_var("TARDIS_FW.MQ.URL", url);

    TardisFuns::init(Some("tests/config")).await?;
    Ok(LifeHold {
        reldb: reldb_container,
        redis: redis_container,
        rabbit: rabbit_container,
    })
}
