use std::time::Duration;

use tardis::basic::result::TardisResult;
use tardis::tokio::time::sleep;
use tardis::{testcontainers, tokio, TardisFuns};

use bios_iam::iam_constants;
use bios_iam::iam_test_helper::BIOSWebTestClient;

mod test_basic;
mod test_iam_scenes_app;
mod test_iam_scenes_common;
mod test_iam_scenes_passport;
mod test_iam_scenes_system;
mod test_iam_scenes_tenant;

#[tokio::test]
async fn test_iam_api() -> TardisResult<()> {
    let docker = testcontainers::clients::Cli::default();
    let _x = test_basic::init(&docker).await?;

    let (sysadmin_name, sysadmin_password) = bios_iam::iam_initializer::init_db(iam_constants::get_tardis_inst()).await?.unwrap();

    tokio::spawn(async move {
        let web_server = TardisFuns::web_server();
        bios_iam::iam_initializer::init(web_server).await.unwrap();
        web_server.start().await.unwrap();
    });

    sleep(Duration::from_millis(500)).await;

    let mut client = BIOSWebTestClient::new("https://localhost:8080/iam".to_string());

    test_iam_scenes_passport::test(&sysadmin_name, &sysadmin_password, &mut client).await?;
    let (sysadmin_name, sysadmin_password) = init_data().await?;
    test_iam_scenes_system::test(&sysadmin_name, &sysadmin_password, &mut client).await?;
    let (sysadmin_name, sysadmin_password) = init_data().await?;
    test_iam_scenes_tenant::test(&sysadmin_name, &sysadmin_password, &mut client).await?;
    let (sysadmin_name, sysadmin_password) = init_data().await?;
    test_iam_scenes_app::test(&sysadmin_name, &sysadmin_password, &mut client).await?;
    let (sysadmin_name, sysadmin_password) = init_data().await?;
    test_iam_scenes_common::test(&sysadmin_name, &sysadmin_password, &mut client).await?;

    Ok(())
}

async fn init_data() -> TardisResult<(String, String)> {
    let funs = iam_constants::get_tardis_inst();
    bios_iam::iam_initializer::truncate_data(&funs).await?;
    bios_iam::iam_initializer::init_rbum_data(&funs).await
}
