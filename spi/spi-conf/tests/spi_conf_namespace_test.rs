use std::env;

use bios_basic::{
    rbum::serv::rbum_kind_serv::RbumKindServ,
    spi::{dto::spi_bs_dto::SpiBsAddReq, spi_constants},
    test::{init_rbum_test_container, test_http_client::TestHttpClient},
};
use bios_spi_conf::{
    conf_constants::DOMAIN_CODE,
    dto::{
        conf_config_dto::{ConfigDescriptor, ConfigPublishRequest},
        conf_namespace_dto::{NamespaceAttribute, NamespaceItem},
    },
};
use serde::__private::de;
use tardis::{
    basic::{dto::TardisContext, field::TrimString, json, result::TardisResult},
    log::debug,
    serde_json::json,
    testcontainers, tokio,
    web::web_resp::{TardisResp, Void},
    TardisFuns,
};
mod spi_conf_test_common;
use spi_conf_test_common::*;

#[tokio::test]
async fn spi_conf_namespace_test() -> TardisResult<()> {
    std::env::set_var("RUST_LOG", "info,sqlx=off,sea_orm=debug,spi_conf_namespace_test=DEBUG,bios_spi_conf=TRACE");
    let docker = testcontainers::clients::Cli::default();
    let container_hold = init_rbum_test_container::init(&docker, None).await?;
    init_tardis().await?;
    let web_server_hanlde = start_web_server();
    let tardis_ctx = TardisContext::default();
    let mut client = TestHttpClient::new("https://localhost:8080/spi-conf".to_string());
    client.set_auth(&tardis_ctx)?;
    let funs = TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None);
    let kind_id = RbumKindServ::get_rbum_kind_id_by_code(spi_constants::SPI_PG_KIND_CODE, &funs).await?.unwrap();
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
    let _: Void = client.put(&format!("/ci/manage/bs/{}/rel/app001", bs_id), &Void {}).await;
    test(&mut client).await?;
    web_server_hanlde.await.unwrap()?;
    drop(container_hold);
    Ok(())
}

pub async fn test(client: &mut TestHttpClient) -> TardisResult<()> {
    client.set_auth(&TardisContext {
        own_paths: "t1/app001".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "app001".to_string(),
        ..Default::default()
    })?;
    let _response = client
        .post::<_, bool>(
            "/ci/namespace",
            &NamespaceAttribute {
                namespace: "test1".to_string(),
                namespace_show_name: "测试命名空间1".to_string(),
                namespace_desc: Some("测试命名空间1".to_string()),
            },
        )
        .await;
    let _response = client
        .post::<_, bool>(
            "/ci/namespace",
            &NamespaceAttribute {
                namespace: "test2".to_string(),
                namespace_show_name: "测试命名空间2".to_string(),
                namespace_desc: Some("测试命名空间2".to_string()),
            },
        )
        .await;
    let _response = client
        .post::<_, bool>(
            "/ci/cs/config",
            &json!( {
                "content": include_str!("./config/conf-default.toml").to_string(),
                "group": "DEFAULT-GROUP".to_string(),
                "data_id": "conf-default".to_string(),
                "schema": "toml",
            }),
        )
        .await;
    let _response = client.get::<String>("/ci/cs/config?namespace_id=public&group=DEFAULT-GROUP&data_id=conf-default").await;
    let _response = client.get::<NamespaceItem>("/ci/namespace?namespace_id=public").await;
    assert_eq!(_response.config_count, 1);
    client.delete("/ci/cs/config?namespace_id=public&group=DEFAULT-GROUP&data_id=conf-default").await;
    let _response = client.get::<NamespaceItem>("/ci/namespace?namespace_id=public").await;
    assert_eq!(_response.config_count, 0);

    Ok(())
}
