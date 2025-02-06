use std::env;
use std::time::Duration;

use bios_basic::rbum::rbum_config::RbumConfig;
use bios_basic::rbum::serv::rbum_kind_serv::RbumKindServ;
use bios_basic::spi::dto::spi_bs_dto::SpiBsAddReq;
use bios_basic::spi::spi_constants;
use bios_basic::test::init_test_container;
use bios_basic::test::test_http_client::TestHttpClient;
use bios_iam::iam_test_helper::BIOSWebTestClient;
use bios_iam::{iam_constants, iam_initializer};
use bios_mw_flow::{flow_constants, flow_initializer};
use bios_spi_kv::{kv_constants, kv_initializer};
use bios_spi_search::{search_constants, search_initializer};
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::tokio::time::sleep;
use tardis::web::web_resp::Void;
use tardis::{tokio, TardisFuns};

mod mock_api;
mod test_flow_scenes_fsm;

#[tokio::test]
async fn test_flow_api() -> TardisResult<()> {
    env::set_var("RUST_LOG", "debug,tardis=trace,bios_mw_event=trace,test_event=trace,sqlx::query=off");

    let _x = init_test_container::init(None).await?;

    let web_server = TardisFuns::web_server();
    iam_initializer::init(&web_server).await.unwrap();
    let (sysadmin_name, sysadmin_password) = init_iam().await?;
    flow_initializer::init(&web_server).await.unwrap();
    init_flow_data().await?;
    kv_initializer::init(&web_server).await.unwrap();
    search_initializer::init(&web_server).await.unwrap();

    web_server.add_module("mock", mock_api::MockApi).await;

    tokio::spawn(async move {
        web_server.start().await.unwrap();
    });

    sleep(Duration::from_millis(500)).await;

    let mut search_client = TestHttpClient::new(format!("https://127.0.0.1:8080/{}", search_constants::DOMAIN_CODE));
    let mut flow_client = TestHttpClient::new(format!("https://127.0.0.1:8080/{}", flow_constants::DOMAIN_CODE));
    let mut iam_client = BIOSWebTestClient::new("https://127.0.0.1:8080/iam".to_string());

    init_spi_kv().await?;
    init_spi_search().await?;

    test_flow_scenes_fsm::test(&mut flow_client, &mut search_client, &mut iam_client, sysadmin_name, sysadmin_password).await?;
    truncate_flow_data().await?;

    Ok(())
}

async fn init_flow_data() -> TardisResult<()> {
    let funs = flow_constants::get_tardis_inst();
    let ctx = TardisContext {
        own_paths: "".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "".to_string(),
        ..Default::default()
    };
    flow_initializer::init_flow_model(&funs, &ctx).await?;

    Ok(())
}
async fn truncate_flow_data() -> TardisResult<()> {
    let funs = flow_constants::get_tardis_inst();
    flow_initializer::truncate_data(&funs).await?;

    Ok(())
}

async fn init_spi_kv() -> TardisResult<()> {
    // Initialize RBUM
    bios_basic::rbum::rbum_initializer::init(kv_constants::DOMAIN_CODE, RbumConfig::default()).await?;

    let funs = TardisFuns::inst_with_db_conn(kv_constants::DOMAIN_CODE.to_string(), None);
    let kind_id = RbumKindServ::get_rbum_kind_id_by_code(spi_constants::SPI_PG_KIND_CODE, &funs).await?.unwrap();
    let ctx = TardisContext {
        own_paths: "".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "".to_string(),
        ..Default::default()
    };

    let mut client = TestHttpClient::new(format!("https://127.0.0.1:8080/{}", kv_constants::DOMAIN_CODE));

    client.set_auth(&ctx)?;

    let bs_id: String = client
        .post(
            "/ci/manage/bs",
            &SpiBsAddReq {
                name: TrimString("test-spi".to_string()),
                kind_id: TrimString(kind_id),
                conn_uri: env::var("TARDIS_FW.DB.URL").unwrap(),
                ak: TrimString("".to_string()),
                sk: TrimString("".to_string()),
                ext: "{\"max_connections\":20,\"min_connections\":10}".to_string(),
                private: false,
                disabled: None,
            },
        )
        .await;

    let _: Void = client.put(&format!("/ci/manage/bs/{}/rel/u001", bs_id), &Void {}).await;

    Ok(())
}

async fn init_spi_search() -> TardisResult<()> {
    // Initialize RBUM
    bios_basic::rbum::rbum_initializer::init(search_constants::DOMAIN_CODE, RbumConfig::default()).await?;

    let funs = TardisFuns::inst_with_db_conn(search_constants::DOMAIN_CODE.to_string(), None);
    let kind_id = RbumKindServ::get_rbum_kind_id_by_code(spi_constants::SPI_PG_KIND_CODE, &funs).await?.unwrap();
    let ctx = TardisContext {
        own_paths: "".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "".to_string(),
        ..Default::default()
    };

    let mut client = TestHttpClient::new(format!("https://127.0.0.1:8080/{}", search_constants::DOMAIN_CODE));

    client.set_auth(&ctx)?;

    let bs_id: String = client
        .post(
            "/ci/manage/bs",
            &SpiBsAddReq {
                name: TrimString("test-spi".to_string()),
                kind_id: TrimString(kind_id),
                conn_uri: env::var("TARDIS_FW.DB.URL").unwrap(),
                ak: TrimString("".to_string()),
                sk: TrimString("".to_string()),
                ext: "{\"max_connections\":20,\"min_connections\":10}".to_string(),
                private: false,
                disabled: None,
            },
        )
        .await;

    let _: Void = client.put(&format!("/ci/manage/bs/{}/rel/u001", bs_id), &Void {}).await;

    Ok(())
}

async fn init_iam() -> TardisResult<(String, String)> {
    let funs = iam_constants::get_tardis_inst();
    iam_initializer::truncate_data(&funs).await?;
    iam_initializer::init_rbum_data(&funs).await
}
