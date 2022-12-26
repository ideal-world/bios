use std::env;

use tardis::basic::result::TardisResult;
use tardis::test::test_container::TardisTestContainer;
use tardis::testcontainers::clients::Cli;
use tardis::testcontainers::core::WaitFor;
use tardis::testcontainers::images::generic::GenericImage;
use tardis::testcontainers::images::redis::Redis;
use tardis::testcontainers::{images, Container};
use tardis::TardisFuns;

pub struct LifeHold<'a> {
    pub search: Container<'a, GenericImage>,
    pub redis: Container<'a, Redis>,
}

pub async fn init(docker: &Cli) -> TardisResult<LifeHold<'_>> {
    let reldb_container = postgres_custom(Some("init_script.sql"), docker);
    let port = reldb_container.get_host_port_ipv4(5432);
    let url = format!("postgres://postgres:123456@localhost:{}/test", port);
    env::set_var("TARDIS_FW.DB.URL", url);

    let redis_container = TardisTestContainer::redis_custom(docker);
    let port = redis_container.get_host_port_ipv4(6379);
    let url = format!("redis://127.0.0.1:{}/0", port);
    env::set_var("TARDIS_FW.CACHE.URL", url);

    env::set_var("RUST_LOG", "debug,test_rbum=trace,sqlx::query=off");
    TardisFuns::init("tests/config").await?;

    // TardisFuns::reldb().conn().execute_one("CREATE EXTENSION zhparser", vec![]).await?;
    // TardisFuns::reldb().conn().execute_one("CREATE TEXT SEARCH CONFIGURATION tfs_zh_cfg (PARSER = zhparser)", vec![]).await?;
    // TardisFuns::reldb().conn().execute_one("ALTER TEXT SEARCH CONFIGURATION tfs_zh_cfg ADD MAPPING FOR n,v,a,i,e,l WITH simple", vec![]).await?;

    Ok(LifeHold {
        search: reldb_container,
        redis: redis_container,
    })
}

pub fn postgres_custom<'a>(init_script_path: Option<&str>, docker: &'a Cli) -> Container<'a, GenericImage> {
    if let Some(init_script_path) = init_script_path {
        let path = env::current_dir()
            .expect("[Tardis.Test_Container] Current path get error")
            .join(std::path::Path::new(init_script_path))
            .to_str()
            .unwrap_or_else(|| panic!("[Tardis.Test_Container] Script Path [{}] get error", init_script_path))
            .to_string();
        docker.run(
            images::generic::GenericImage::new("abcfy2/zhparser", "15")
                .with_env_var("POSTGRES_PASSWORD", "123456")
                .with_env_var("POSTGRES_DB", "test")
                .with_volume(path, "/docker-entrypoint-initdb.d/")
                .with_wait_for(WaitFor::message_on_stderr("database system is ready to accept connections")),
        )
    } else {
        docker.run(
            images::generic::GenericImage::new("abcfy2/zhparser", "15")
                .with_env_var("POSTGRES_PASSWORD", "123456")
                .with_env_var("POSTGRES_DB", "test")
                .with_wait_for(WaitFor::message_on_stderr("database system is ready to accept connections")),
        )
    }
}
