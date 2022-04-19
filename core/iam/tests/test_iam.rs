use std::time::Duration;

use tardis::{TardisFuns, testcontainers, tokio};
use tardis::basic::result::TardisResult;
use tardis::tokio::time::sleep;

use bios_basic::rbum::rbum_initializer::get_first_account_context;
use bios_iam::iam_constants;

mod test_basic;
mod test_cp_all;
mod test_cs_account;
mod test_cs_tenant;
mod test_ct_account;
mod test_ct_app;
mod test_ct_basic;
mod test_ct_cert_conf;
mod test_ct_cert;
mod test_ct_http_res;
mod test_ct_role;
mod test_ct_tenant;

#[tokio::test]
async fn test_iam() -> TardisResult<()> {
    let docker = testcontainers::clients::Cli::default();
    let _x = test_basic::init(&docker).await?;

    let funs = iam_constants::get_tardis_inst();
    let (sysadmin_name, sysadmin_password) = bios_iam::iam_initializer::init_db(funs).await?.unwrap();

    sleep(Duration::from_secs(1)).await;

    let cxt = get_first_account_context(
        iam_constants::RBUM_KIND_SCHEME_IAM_ACCOUNT,
        &bios_basic::Components::Iam.to_string(),
        &TardisFuns::inst_with_db_conn("".to_string()),
    )
    .await?
    .unwrap();
    test_cp_all::test((&sysadmin_name, &sysadmin_password), &cxt).await?;
    test_cs_tenant::test(&cxt).await?;
    test_cs_account::test(&cxt).await?;
    let (context1, context2) = test_ct_basic::test(&cxt).await?;
    test_ct_tenant::test(&context1, &context2).await?;
    test_ct_role::test(&context1, &context2).await?;
    test_ct_app::test(&context1, &context2).await?;
    test_ct_account::test(&context1, &context2).await?;
    test_ct_http_res::test(&context1, &context2).await?;
    test_ct_cert_conf::test(&context1, &context2).await?;
    test_ct_cert::test(&context1, &context2).await?;
    Ok(())
}
