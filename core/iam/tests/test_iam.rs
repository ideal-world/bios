use tardis::{TardisFuns, testcontainers, tokio};
use tardis::basic::result::TardisResult;

use bios_basic::rbum::rbum_initializer::get_first_account_context;
use bios_iam::iam_constants;

mod test_basic;
mod test_cp_all;
mod test_cs_tenant;
mod test_cs_account;

#[tokio::test]
async fn test_iam() -> TardisResult<()> {
    let docker = testcontainers::clients::Cli::default();
    let _x = test_basic::init(&docker).await?;

    let funs = iam_constants::get_tardis_inst();
    let (sysadmin_name,sysadmin_password) = bios_iam::iam_initializer::init_db(funs).await?
        .unwrap();

    let cxt = get_first_account_context(
        iam_constants::RBUM_KIND_SCHEME_IAM_ACCOUNT,
        &bios_basic::Components::Iam.to_string(),
        &TardisFuns::inst_with_db_conn("".to_string()),
    )
    .await?
    .unwrap();
    test_cp_all::test((&sysadmin_name,&sysadmin_password),&cxt).await?;
    test_cs_tenant::test(&cxt).await?;
    test_cs_account::test(&cxt).await?;
    Ok(())
}
