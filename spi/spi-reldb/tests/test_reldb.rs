use std::env;
use std::time::Duration;

use bios_basic::rbum::rbum_config::RbumConfig;
use bios_basic::rbum::serv::rbum_kind_serv::RbumKindServ;
use bios_basic::spi::api::spi_ci_bs_api;
use bios_basic::spi::dto::spi_bs_dto::SpiBsAddReq;
use bios_basic::test::init_rbum_test_container;
use bios_basic::test::test_http_client::TestHttpClient;
use bios_spi_reldb::dto::reldb_exec_dto::{ReldbDmlReq, ReldbDmlResp};
use bios_spi_reldb::reldb_constants::DOMAIN_CODE;
use bios_spi_reldb::reldb_initializer;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::tokio::time::sleep;
use tardis::web::web_resp::Void;
use tardis::{testcontainers, tokio, TardisFuns};

#[tokio::test]
async fn test_iam_serv() -> TardisResult<()> {
    let docker = testcontainers::clients::Cli::default();
    let _x = init_rbum_test_container::init(&docker).await?;

    init_data().await?;

    Ok(())
}

async fn init_data() -> TardisResult<()> {
    bios_basic::rbum::rbum_initializer::init(DOMAIN_CODE, RbumConfig::default()).await?;

    tokio::spawn(async move {
        let web_server = TardisFuns::web_server();
        web_server.add_module(DOMAIN_CODE, (spi_ci_bs_api::SpiCiBsApi)).await;
        reldb_initializer::init(web_server).await.unwrap();
        web_server.start().await.unwrap();
    });

    let funs = TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None);
    let kind_id = RbumKindServ::get_rbum_kind_id_by_code("spi-pg", &funs).await?.unwrap();
    let ctx = TardisContext {
        own_paths: "".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "".to_string(),
        ..Default::default()
    };

    sleep(Duration::from_millis(500)).await;

    let mut client = TestHttpClient::new("https://localhost:8080/spi-reldb".to_string());

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
                ext: "".to_string(),
                private: false,
                disabled: None,
            },
        )
        .await;

    let _: Void = client.put(&format!("/ci/manage/bs/{}/rel/app001", bs_id), &Void {}).await;

    client.set_auth(&TardisContext {
        own_paths: "t1/a1".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "app001".to_string(),
        ..Default::default()
    })?;

    let reldb_dml_resp: ReldbDmlResp = client
        .post(
            "/ci/exec/dml",
            &ReldbDmlReq {
                sql: "create table test_table (id int)".to_string(),
                params: TardisFuns::json.str_to_json("{}}")?,
                tx_id: None,
            },
        )
        .await;

    Ok(())
}
