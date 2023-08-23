use std::env;
#[cfg(feature = "web-server")]
use std::time::Duration;

use bios_auth::auth_initializer;
use tardis::basic::result::TardisResult;
#[cfg(feature = "web-server")]
use tardis::tokio::time::sleep;
#[cfg(feature = "web-server")]
use tardis::TardisFuns;
use tardis::{testcontainers, tokio};
mod init_cache_container;
#[cfg(feature = "web-server")]
mod test_auth_encrypt;
mod test_auth_init;
mod test_auth_match;
#[cfg(feature = "web-server")]
mod test_auth_req;
mod test_auth_res;

#[tokio::test]
async fn test_auth() -> TardisResult<()> {
    env::set_var("RUST_LOG", "debug,bios_auth=trace");
    test_auth_res::test_res()?;

    let docker = testcontainers::clients::Cli::default();
    let _x = init_cache_container::init(&docker).await?;

    test_auth_match::test_match().await?;

    auth_initializer::crypto_init().await?;
    test_auth_init::test_init().await?;

    #[cfg(feature = "web-server")]
    {
        let web_server = TardisFuns::web_server();
        auth_initializer::init_api(web_server).await?;
        tokio::spawn(async move {
            web_server.start().await.unwrap();
        });
        sleep(Duration::from_millis(500)).await;

        test_auth_req::test_req().await?;
        test_auth_encrypt::test_encrypt().await?;
    }
    Ok(())
}
