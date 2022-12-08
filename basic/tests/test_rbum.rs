use tardis::basic::result::TardisResult;
use tardis::{testcontainers, tokio};

mod test_basic;
mod test_rbum_cert;
mod test_rbum_domain;
mod test_rbum_event;
mod test_rbum_item;
mod test_rbum_kind;
mod test_rbum_rel;
mod test_rbum_set;
mod test_scope;

#[tokio::test]
async fn test_rbum() -> TardisResult<()> {
    let docker = testcontainers::clients::Cli::default();
    let _x = test_basic::init(&docker).await?;
    let ctx = test_basic::init_test_data().await?;
    test_scope::test().await?;
    test_rbum_domain::test(&ctx).await?;
    test_rbum_kind::test(&ctx).await?;
    test_rbum_item::test(&ctx).await?;
    test_rbum_cert::test(&ctx).await?;
    test_rbum_rel::test(&ctx).await?;
    test_rbum_set::test(&ctx).await?;
    test_rbum_event::test().await?;
    Ok(())
}
