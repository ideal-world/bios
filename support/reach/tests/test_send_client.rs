// std::env::set_var("PROFILE", "prod");
use std::{collections::HashMap, time::Duration};

use bios_basic::rbum::dto::rbum_item_dto::RbumItemAddReq;
use serde::Deserialize;
use tardis::{
    basic::result::TardisResult,
    crypto::{crypto_base64::TardisCryptoBase64, crypto_digest::TardisCryptoDigest},
    log,
    serde_json::{self, json},
    testcontainers, tokio, TardisFuns,
};

mod test_reach_common;
use bios_reach::{
    client::{GenericTemplate, SendChannel},
    consts::*,
    dto::*,
    invoke,
};
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
    let client = invoke::Client::new("http://localhost:8080/reach", ctx, &funs);
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
    let client = invoke::Client::new("http://localhost:8080/reach", ctx, &funs);
    client.mail_pwd_send(&mail, &content, "测试", &()).await?;
    // wait for send
    tokio::time::sleep(Duration::from_secs(10)).await;
    drop(holder);
    Ok(())
}
