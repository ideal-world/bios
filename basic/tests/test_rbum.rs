use tardis::basic::result::TardisResult;
use tardis::tokio;

mod test_basic;
mod test_rbum_domain;

#[tokio::test]
async fn test_rbum() -> TardisResult<()> {
    let docker = testcontainers::clients::Cli::default();
    let _x = test_basic::init(&docker).await?;
    test_rbum_domain::test_rbum_domain().await
}
