use std::env;
use std::time::Duration;

use bios_basic::rbum::rbum_config::RbumConfig;
use bios_basic::rbum::serv::rbum_kind_serv::RbumKindServ;
use bios_basic::spi::dto::spi_bs_dto::SpiBsAddReq;
use bios_basic::spi::spi_constants;
use bios_basic::test::init_rbum_test_container;
use bios_basic::test::test_http_client::TestHttpClient;
use bios_spi_reldb::reldb_constants::DOMAIN_CODE;
use bios_spi_reldb::reldb_initializer;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::tokio::time::sleep;
use tardis::web::web_resp::Void;
use tardis::{testcontainers, tokio, TardisFuns};
mod test_reldb_exec;

#[tokio::test]
async fn test_reldb() -> TardisResult<()> {
    let docker = testcontainers::clients::Cli::default();
    let _x = init_rbum_test_container::init(&docker).await?;

    env::set_var("RUST_LOG", "debug,test_reldb=trace,sqlx::query=off");

    init_data().await?;

    Ok(())
}

async fn init_data() -> TardisResult<()> {
    // Initialize RBUM
    bios_basic::rbum::rbum_initializer::init(DOMAIN_CODE, RbumConfig::default()).await?;

    let web_server = TardisFuns::web_server();
    // Initialize SPI reldb
    reldb_initializer::init(web_server).await.unwrap();

    tokio::spawn(async move {
        web_server.start().await.unwrap();
    });

    sleep(Duration::from_millis(500)).await;

    let funs = TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None);
    let kind_id = RbumKindServ::get_rbum_kind_id_by_code(spi_constants::SPI_PG_KIND_CODE, &funs).await?.unwrap();
    let ctx = TardisContext {
        own_paths: "".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "".to_string(),
        ..Default::default()
    };

    let mut client = TestHttpClient::new(format!("https://localhost:8080/{}", DOMAIN_CODE));

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
                ext: "{\"max_connections\":20,\"min_connections\":1}".to_string(),
                private: false,
                disabled: None,
            },
        )
        .await;

    let _: Void = client.put(&format!("/ci/manage/bs/{}/rel/app001", bs_id), &Void {}).await;

    test_reldb_exec::test(&mut client).await?;

    Ok(())
}
