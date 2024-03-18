use std::time::Duration;

use bios_basic::rbum::rbum_config::RbumConfig;
use bios_basic::rbum::serv::rbum_kind_serv::RbumKindServ;
use bios_basic::spi::dto::spi_bs_dto::SpiBsAddReq;
use bios_basic::test::init_rbum_test_container;
use bios_basic::test::test_http_client::TestHttpClient;
use bios_spi_object::object_constants::{self, DOMAIN_CODE};
use bios_spi_object::object_initializer;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::test::test_container::TardisTestContainer;
use tardis::tokio::time::sleep;
use tardis::web::web_resp::Void;
use tardis::{testcontainers, tokio, TardisFuns};
mod test_object_obj;

#[tokio::test]
async fn test_object() -> TardisResult<()> {
    let docker = testcontainers::clients::Cli::default();
    let minio = TardisTestContainer::minio_custom(&docker);
    let minio_url = format!("http://127.0.0.1:{}", minio.get_host_port_ipv4(9000));
    let _x = init_rbum_test_container::init(&docker, None).await?;

    init_data(&minio_url).await?;

    Ok(())
}

async fn init_data(minio_url: &str) -> TardisResult<()> {
    // Initialize RBUM
    bios_basic::rbum::rbum_initializer::init(DOMAIN_CODE, RbumConfig::default()).await?;

    let web_server = TardisFuns::web_server();
    // Initialize SPI object
    object_initializer::init(&web_server).await.unwrap();

    tokio::spawn(async move {
        web_server.start().await.unwrap();
    });

    sleep(Duration::from_millis(500)).await;

    let funs = TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None);
    let kind_id = RbumKindServ::get_rbum_kind_id_by_code(object_constants::SPI_S3_KIND_CODE, &funs).await?.unwrap();
    let ctx = TardisContext {
        own_paths: "".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "".to_string(),
        ..Default::default()
    };

    let mut client = TestHttpClient::new(format!("https://127.0.0.1:8080/{}", DOMAIN_CODE));

    client.set_auth(&ctx)?;

    let bs_id: String = client
        .post(
            "/ci/manage/bs",
            &SpiBsAddReq {
                name: TrimString("test-spi".to_string()),
                kind_id: TrimString(kind_id),
                conn_uri: minio_url.to_string(),
                ak: TrimString("minioadmin".to_string()),
                sk: TrimString("minioadmin".to_string()),
                ext: r#"{"region":"us-east-1"}"#.to_string(),
                private: false,
                disabled: None,
            },
        )
        .await;

    let _: Void = client.put(&format!("/ci/manage/bs/{}/rel/app001", bs_id), &Void {}).await;

    test_object_obj::test(&mut client).await?;

    Ok(())
}
