use tardis::basic::result::TardisResult;
use tardis::tokio;

mod test_basic;
mod test_cs_tenant;

#[tokio::test]
async fn test_rbum() -> TardisResult<()> {
    let docker = testcontainers::clients::Cli::default();
    let _x = test_basic::init(&docker).await?;
    test_cs_tenant::test().await?;
    Ok(())
}
