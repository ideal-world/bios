use std::env;

use tardis::basic::result::TardisResult;
use tardis::test::test_container::TardisTestContainer;
use tardis::TardisFuns;
use tardis::{testcontainers, tokio};

mod config;
mod initializer;

///
/// Visit: http://127.0.0.1:8081/
///
#[tokio::main]
async fn main() -> TardisResult<()> {
    let docker = testcontainers::clients::Cli::default();

    let mysql_container = TardisTestContainer::mysql_custom(None, &docker);
    let port = mysql_container.get_host_port(3306);
    let url = format!("mysql://root:123456@localhost:{}/test", port);
    env::set_var("TARDIS_FW.DB.URL", url);

    let redis_container = TardisTestContainer::redis_custom(&docker);
    let port = redis_container.get_host_port(6379);
    let url = format!("redis://127.0.0.1:{}/0", port);
    env::set_var("TARDIS_FW.CACHE.URL", url);

    let rabbit_container = TardisTestContainer::rabbit_custom(&docker);
    let port = rabbit_container.get_host_port(5672);
    let url = format!("amqp://guest:guest@127.0.0.1:{}/%2f", port);
    env::set_var("TARDIS_FW.MQ.URL", url);

    env::set_var("RUST_LOG", "info");
    TardisFuns::init("services/bios-all/config").await?;
    let web_server = TardisFuns::web_server();
    initializer::init(web_server).await?;
    web_server.start().await
}
