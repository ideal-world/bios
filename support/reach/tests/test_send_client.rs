// std::env::set_var("PROFILE", "prod");
use serde::Deserialize;
use std::time::Duration;
use tardis::{basic::result::TardisResult, testcontainers, tokio};

mod test_reach_common;
use bios_reach::{reach_consts::*, reach_invoke};
use test_reach_common::*;
#[derive(Deserialize)]
pub struct TestConfig {
    phone: String,
    mail: String,
    code: String,
    content: String,
}

impl TestConfig {
    fn load() -> Self {
        toml::from_slice(include_bytes!("config/test-send-client.toml")).expect("invalid config")
    }
}

#[tokio::test(flavor = "multi_thread")]
pub async fn test_hw_sms() -> TardisResult<()> {
    let TestConfig { phone, code, .. } = TestConfig::load();
    std::env::set_var("PROFILE", "prod");
    std::env::set_var("RUST_LOG", "info,sqlx=off,sea_orm=debug,tardis=TRACE,bios_reach=trace,reqwest=trace");

    let docker = testcontainers::clients::Cli::default();
    let holder = init_tardis(&docker).await?;
    let ctx = get_test_ctx();
    let funs = get_tardis_inst();
    let client = reach_invoke::Client::new("http://localhost:8080/reach", ctx, &funs);
    // client.pwd_send(&phone, &code, &()).await?;

    client.vcode_send(&phone, &code, &()).await?;
    // wait for send
    tokio::time::sleep(Duration::from_secs(10)).await;
    drop(holder);
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
pub async fn test_mail() -> TardisResult<()> {
    let TestConfig { content, mail, .. } = TestConfig::load();
    std::env::set_var("PROFILE", "prod");
    std::env::set_var("RUST_LOG", "info,sqlx=off,sea_orm=debug,tardis=TRACE,bios_reach=trace,reqwest=trace");

    let docker = testcontainers::clients::Cli::default();
    let holder = init_tardis(&docker).await?;
    let ctx = get_test_ctx();
    let funs = get_tardis_inst();
    let client = reach_invoke::Client::new("http://localhost:8080/reach", ctx, &funs);
    client.mail_pwd_send(&mail, &content, "测试", &()).await?;
    // wait for send
    tokio::time::sleep(Duration::from_secs(10)).await;
    drop(holder);
    Ok(())
}
