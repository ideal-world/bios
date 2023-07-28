use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    testcontainers, tokio,
};

mod test_reach_common;
use bios_reach::{consts::*, invoke};
use test_reach_common::*;
#[tokio::test]
pub async fn test_ct_api() -> TardisResult<()> {
    std::env::set_var("RUST_LOG", "info,sqlx=off,sea_orm=debug,spi_conf_namespace_test=DEBUG,bios_spi_conf=TRACE");
    let docker = testcontainers::clients::Cli::default();
    let holder = init_tardis(&docker).await?;
    let ctx = TardisContext {
        owner: "test-reach".to_string(),
        ..Default::default()
    };
    let funs = get_tardis_inst();
    let invoke_config = invoke::Config {
        base_url: "https://localhost:8080".to_string(),
    };
    let client = invoke::Client::new(&invoke_config, &ctx, &funs);
    client.mail_pwd_send("test_mail", "hello", "hello from test").await?;
    wait_for_press();
    drop(holder);
    Ok(())
}
