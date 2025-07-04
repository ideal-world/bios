use std::env;
use std::time::Duration;

use bios_basic::rbum::serv::rbum_kind_serv::RbumKindServ;
use bios_basic::spi::dto::spi_bs_dto::SpiBsAddReq;
use bios_basic::spi::spi_constants;
use bios_basic::test::init_test_container;
use bios_basic::test::test_http_client::TestHttpClient;
use bios_spi_log::log_constants::{DOMAIN_CODE, SPI_PG_V2_KIND_CODE};
use bios_spi_log::log_initializer;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::tokio::time::sleep;
use tardis::web::web_resp::Void;
use tardis::{tokio, TardisFuns};
mod test_log_item;

#[tokio::test]
async fn test_log() -> TardisResult<()> {
    env::set_var("RUST_LOG", "debug,test_log=trace,sqlx::query=off");

    let _x = init_test_container::init(None).await?;

    init_data().await?;

    Ok(())
}

async fn init_data() -> TardisResult<()> {
    let web_server = TardisFuns::web_server();
    // Initialize SPI Log
    log_initializer::init(&web_server).await.unwrap();

    tokio::spawn(async move {
        web_server.start().await.unwrap();
    });

    sleep(Duration::from_millis(500)).await;

    let funs = TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None);
    let kind_id = RbumKindServ::get_rbum_kind_id_by_code(spi_constants::SPI_PG_KIND_CODE, &funs).await?.unwrap();
    let kind_v2_id = RbumKindServ::get_rbum_kind_id_by_code(SPI_PG_V2_KIND_CODE, &funs).await?.unwrap();
    let ctx = TardisContext {
        own_paths: "".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "".to_string(),
        ..Default::default()
    };

    let mut client = TestHttpClient::new(format!("http://127.0.0.1:8080/{}", DOMAIN_CODE));

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

    let bs_v2_id: String = client
        .post(
            "/ci/manage/bs",
            &SpiBsAddReq {
                name: TrimString("test-spi".to_string()),
                kind_id: TrimString(kind_v2_id),
                conn_uri: env::var("TARDIS_FW.DB.URL").unwrap(),
                ak: TrimString("".to_string()),
                sk: TrimString("".to_string()),
                ext: "{\"max_connections\":20,\"min_connections\":10}".to_string(),
                private: false,
                disabled: None,
            },
        )
        .await;

    let app001 = "app001";
    let app002 = "app002";
    let _: Void = client.put(&format!("/ci/manage/bs/{}/rel/{}", bs_id, app001), &Void {}).await;
    let _: Void = client.put(&format!("/ci/manage/bs/{}/rel/{}", bs_v2_id, app002), &Void {}).await;

    test_log_item::test_v1(app001, &mut client).await?;
    test_log_item::test_v2(app002, &mut client).await?;
    Ok(())
}
