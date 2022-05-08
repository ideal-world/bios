use std::time::Duration;

use tardis::basic::result::TardisResult;
use tardis::tokio::time::sleep;
use tardis::{testcontainers, tokio, TardisFuns};

use bios_basic::rbum::rbum_initializer::get_first_account_context;
use bios_iam::iam_constants;

mod test_basic;
mod test_ca_app;
mod test_ca_basic;
mod test_cc_account;
mod test_cc_attr;
mod test_cp_all;
mod test_cs_tenant;
mod test_ct_app;
mod test_ct_basic;
mod test_ct_tenant;

#[tokio::test]
async fn test_iam() -> TardisResult<()> {
    let docker = testcontainers::clients::Cli::default();
    let _x = test_basic::init(&docker).await?;

    let funs = iam_constants::get_tardis_inst();
    let (sysadmin_name, sysadmin_password) = bios_iam::iam_initializer::init_db(funs).await?.unwrap();

    sleep(Duration::from_secs(1)).await;

    let system_admin_context = get_first_account_context(
        iam_constants::RBUM_KIND_SCHEME_IAM_ACCOUNT,
        &bios_basic::Components::Iam.to_string(),
        &TardisFuns::inst_with_db_conn("".to_string()),
    )
    .await?
    .unwrap();

    test_cp_all::test((&sysadmin_name, &sysadmin_password), &system_admin_context).await?;

    test_cs_tenant::test(&system_admin_context).await?;

    let (tenant1_admin_context, tenant2_admin_context) = test_ct_basic::test(&system_admin_context).await?;
    test_ct_tenant::test(&tenant1_admin_context, &tenant2_admin_context).await?;
    test_ct_app::test(&tenant1_admin_context, &tenant2_admin_context).await?;

    let (app1_admin_context, app2_admin_context, tenant3_admin_context) = test_ca_basic::test(&system_admin_context).await?;
    test_ca_app::test(&app1_admin_context, &app2_admin_context).await?;

    test_cc_account::test(
        &system_admin_context,
        &tenant1_admin_context,
        &tenant3_admin_context,
        &app1_admin_context,
        &app2_admin_context,
    )
    .await?;

    test_cc_attr::test(
        &system_admin_context,
        &tenant1_admin_context,
        &tenant3_admin_context,
        &app1_admin_context,
        &app2_admin_context,
    )
    .await?;

    // test_cs_account::test(&system_admin_context).await?;
    // test_ct_role::test(&tenant1_admin_context, &tenant2_admin_context).await?;
    // test_ct_account::test(&tenant1_admin_context, &tenant2_admin_context).await?;
    // test_ct_res::test(&tenant1_admin_context, &tenant2_admin_context).await?;
    // test_ct_cert_conf::test(&tenant1_admin_context, &tenant2_admin_context).await?;
    // test_ct_cert::test(&tenant1_admin_context, &tenant2_admin_context).await?;
    // test_ct_set::test(&tenant1_admin_context, &tenant2_admin_context).await?;
    // test_ca_role::test(&app1_admin_context, &app2_admin_context).await?;
    // test_ca_account::test(&app1_admin_context, &app2_admin_context).await?;
    // test_ca_res::test(&app1_admin_context, &app2_admin_context).await?;
    // test_ca_set::test(&app1_admin_context, &app2_admin_context).await?;
    // test_ct_ca_attr::test(&tenant3_admin_context, &app1_admin_context, &app2_admin_context, &tenant2_admin_context).await?;

    Ok(())
}
