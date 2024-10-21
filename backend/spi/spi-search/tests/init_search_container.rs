use std::env;

use tardis::basic::result::TardisResult;
use tardis::test::test_container::TardisTestContainer;
use tardis::testcontainers::core::{Mount, WaitFor};
use tardis::testcontainers::runners::AsyncRunner;
use tardis::testcontainers::{ContainerAsync, GenericImage, ImageExt};
use tardis::TardisFuns;
use testcontainers_modules::elastic_search::ElasticSearch;
use testcontainers_modules::redis::Redis;

pub struct LifeHold {
    pub search: ContainerAsync<GenericImage>,
    pub redis: ContainerAsync<Redis>,
    pub es: ContainerAsync<ElasticSearch>,
}

pub async fn init() -> TardisResult<LifeHold> {
    let root_path = "";
    // let root_path = "spi/spi-search/";
    env::set_var("RUST_LOG", "debug,test_rbum=trace,sqlx::query=off");

    let reldb_container = postgres_custom(Some(&format!("{}config", root_path))).await;
    let port = reldb_container.get_host_port_ipv4(5432).await?;
    let url = format!("postgres://postgres:123456@127.0.0.1:{}/test", port);
    env::set_var("TARDIS_FW.DB.URL", url);

    let redis_container = TardisTestContainer::redis_custom().await?;
    let port = redis_container.get_host_port_ipv4(6379).await?;
    let url = format!("redis://127.0.0.1:{}/0", port);
    env::set_var("TARDIS_FW.CACHE.URL", url);

    let es_container = TardisTestContainer::es_custom().await?;
    let port = es_container.get_host_port_ipv4(9200).await?;
    let url = format!("http://elastic:123456@127.0.0.1:{}", port);
    env::set_var("TARDIS_FW.ES.URL", url);

    let mq_container = TardisTestContainer::rabbit_custom().await?;
    let port = mq_container.get_host_port_ipv4(5672).await?;
    let url = format!("amqp://guest:guest@127.0.0.1:{port}/%2f");
    std::env::set_var("TARDIS_FW.MQ.URL", url);

    TardisFuns::init(Some(&format!("{}tests/config", root_path))).await?;

    Ok(LifeHold {
        search: reldb_container,
        redis: redis_container,
        es: es_container,
    })
}

pub async fn postgres_custom<'a>(init_script_path: Option<&str>) -> ContainerAsync<GenericImage> {
    if let Some(init_script_path) = init_script_path {
        let path = env::current_dir()
            .expect("[Tardis.Test_Container] Current path get error")
            .join(std::path::Path::new(init_script_path))
            .to_str()
            .unwrap_or_else(|| panic!("[Tardis.Test_Container] Script Path [{}] get error", init_script_path))
            .to_string();
        GenericImage::new("abcfy2/zhparser", "15")
            .with_wait_for(WaitFor::message_on_stderr("database system is ready to accept connections"))
            .with_mount(Mount::bind_mount(path, "/docker-entrypoint-initdb.d/"))
            .with_env_var("POSTGRES_PASSWORD", "123456")
            .with_env_var("POSTGRES_DB", "test")
            .start()
            .await
            .expect("zhparser started")
        // .with_volume(path, "/docker-entrypoint-initdb.d/")
    } else {
        GenericImage::new("abcfy2/zhparser", "15")
            .with_wait_for(WaitFor::message_on_stderr("database system is ready to accept connections"))
            .with_env_var("POSTGRES_PASSWORD", "123456")
            .with_env_var("POSTGRES_DB", "test")
            .start()
            .await
            .expect("zhparser started")
    }
}
