use tardis::basic::result::TardisResult;
use tardis::tokio;

mod test_basic;
mod test_rbum_cert;
mod test_rbum_domain;
mod test_rbum_item;
mod test_rbum_kind;
mod test_rbum_rel;
mod test_rbum_set;

#[tokio::test]
async fn test_rbum() -> TardisResult<()> {
    let docker = testcontainers::clients::Cli::default();
    let _x = test_basic::init(&docker).await?;
    let cxt = test_basic::init_test_data().await?;
    test_rbum_domain::test(&cxt).await?;
    test_rbum_kind::test(&cxt).await?;
    test_rbum_item::test(&cxt).await?;
    test_rbum_cert::test(&cxt).await?;
    test_rbum_rel::test(&cxt).await?;
    test_rbum_set::test(&cxt).await?;
    Ok(())
}
