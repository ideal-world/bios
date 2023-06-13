use std::env;
use std::time::Duration;

use bios_basic::test::init_rbum_test_container;
use bios_basic::test::test_http_client::TestHttpClient;
use bios_mw_flow::{flow_constants, flow_initializer};
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::tokio::time::sleep;
use tardis::{testcontainers, tokio, TardisFuns};

mod test_flow_scenes_fsm;

#[tokio::test]
async fn test_flow_api() -> TardisResult<()> {
    env::set_var("RUST_LOG", "debug,test_flow_api=trace,sqlx::query=off");

    let docker = testcontainers::clients::Cli::default();
    let _x = init_rbum_test_container::init(&docker, None).await?;

    let funs = flow_constants::get_tardis_inst();
    flow_initializer::init_db(funs).await?;

    let web_server = TardisFuns::web_server();
    flow_initializer::init(web_server).await.unwrap();

    tokio::spawn(async move {
        web_server.start().await.unwrap();
    });

    sleep(Duration::from_millis(500)).await;

    let mut client = TestHttpClient::new("https://localhost:8080/flow".to_string());

    test_flow_scenes_fsm::test(&mut client).await?;
    init_data().await?;

    Ok(())
}

async fn init_data() -> TardisResult<()> {
    let funs = flow_constants::get_tardis_inst();
    flow_initializer::truncate_data(&funs).await?;
    let ctx = TardisContext {
        own_paths: "".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "".to_string(),
        ..Default::default()
    };
    flow_initializer::init_rbum_data(&funs, &ctx).await?;
    Ok(())
}
