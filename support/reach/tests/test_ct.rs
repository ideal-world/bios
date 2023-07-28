use tardis::{basic::result::TardisResult, testcontainers, tokio};

mod test_reach_common;
use test_reach_common::*;

#[tokio::test]
pub async fn test_ct_api() -> TardisResult<()> {
    std::env::set_var("RUST_LOG", "info,sqlx=off,sea_orm=debug,spi_conf_namespace_test=DEBUG,bios_spi_conf=TRACE");
    let docker = testcontainers::clients::Cli::default();
    let holder = init_tardis(&docker).await?;
    wait_for_press();
    drop(holder);
    Ok(())
}
