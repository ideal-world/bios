use tardis::basic::result::TardisResult;
use tardis::{testcontainers, tokio, TardisFuns};

use bios_basic::rbum::rbum_initializer::get_first_account_context;
use bios_iam::iam_constants;

mod test_basic;
mod test_cs_tenant;

#[tokio::test]
async fn test_iam() -> TardisResult<()> {
    let docker = testcontainers::clients::Cli::default();
    let _x = test_basic::init(&docker).await?;
    let cxt = get_first_account_context(
        iam_constants::RBUM_KIND_SCHEME_IAM_ACCOUNT,
        &bios_basic::Components::Iam.to_string(),
        &TardisFuns::inst_with_db_conn("".to_string()),
    )
    .await?
    .unwrap();
    test_cs_tenant::test(&cxt).await?;
    Ok(())
}
