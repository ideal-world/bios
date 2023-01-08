use std::env;
use std::time::Duration;

use bios_auth::auth_initializer;
use tardis::basic::result::TardisResult;
use tardis::tokio::time::sleep;
use tardis::{testcontainers, tokio, TardisFuns};
mod init_cache_container;

#[tokio::test]
async fn test_auth() -> TardisResult<()> {
    let docker = testcontainers::clients::Cli::default();
    let _x = init_cache_container::init(&docker).await?;

    env::set_var("RUST_LOG", "debug,test_reldb=trace,sqlx::query=off");

    init_data().await?;
    Ok(())
}

async fn init_data() -> TardisResult<()> {
    let web_server = TardisFuns::web_server();
    // Initialize Auth
    auth_initializer::init(web_server).await.unwrap();

    tokio::spawn(async move {
        web_server.start().await.unwrap();
    });

    sleep(Duration::from_millis(500)).await;

    // TODO

    Ok(())
}
