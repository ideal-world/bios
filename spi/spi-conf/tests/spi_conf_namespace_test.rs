use std::{env, io};

use bios_basic::{
    rbum::serv::rbum_kind_serv::RbumKindServ,
    spi::{dto::spi_bs_dto::SpiBsAddReq, spi_constants},
    test::{init_rbum_test_container, test_http_client::TestHttpClient},
};
use bios_spi_conf::{
    conf_constants::DOMAIN_CODE,
    dto::{
        conf_config_dto::{ConfigDescriptor, ConfigPublishRequest, ConfigHistoryListResponse},
        conf_namespace_dto::{NamespaceAttribute, NamespaceItem},
    },
};
use serde::__private::de;
use tardis::{
    basic::{dto::TardisContext, field::TrimString, json, result::TardisResult},
    log::debug,
    serde_json::{json, Value},
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
    let _web_server_hanlde = start_web_server();
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
    test_curd(&mut client).await?;
    // web_server_hanlde.await.unwrap()?;
    drop(container_hold);
    Ok(())
}

pub async fn test_curd(client: &mut TestHttpClient) -> TardisResult<()> {
    client.set_auth(&TardisContext {
        own_paths: "t1/app001".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "app001".to_string(),
        ..Default::default()
    })?;
    // 1. create namespace
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
    // 2. publish a config
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
    // try update
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
    // 3. retrieve config
    let _response = client.get::<String>("/ci/cs/config?namespace_id=public&group=DEFAULT-GROUP&data_id=conf-default").await;
    // 4. get namespace info
    let _response = client.get::<NamespaceItem>("/ci/namespace?namespace_id=public").await;
    assert_eq!(_response.config_count, 1);
    // 4.1 get namespace list
    let _response = client.get::<Vec<NamespaceItem>>("/ci/namespace/list").await;
    assert_eq!(_response.len(), 3);
    // since we have published a config, the config_count should be 1
    // 5. delete config
    client.delete("/ci/cs/config?namespace_id=public&group=DEFAULT-GROUP&data_id=conf-default").await;
    // 6. get namespace info
    let _response = client.get::<NamespaceItem>("/ci/namespace?namespace_id=public").await;
    // since we have deleted the config, the config_count should be 0
    assert_eq!(_response.config_count, 0);
    // 7. update namespace
    let _response = client
        .put::<_, bool>(
            "/ci/namespace",
            &NamespaceAttribute {
                namespace: "test1".to_string(),
                namespace_show_name: "测试命名空间1".to_string(),
                namespace_desc: Some("测试命名空间1-修改".to_string()),
            },
        )
        .await;
    // varify the namespace_desc has been updated
    let response = client.get::<NamespaceItem>("/ci/namespace?namespace_id=test1").await;
    assert_eq!(&response.namespace_desc.unwrap(), "测试命名空间1-修改");

    // 8. delete namespace
    // 8.1 first publish a config
    let _response = client
        .post::<_, bool>(
            "/ci/cs/config",
            &json!( {
                "content": include_str!("./config/conf-default.toml").to_string(),
                "group": "DEFAULT-GROUP".to_string(),
                "data_id": "conf-default".to_string(),
                "schema": "toml",
                "namespace_id": "test1".to_string(),
            }),
        )
        .await;
    // 8.2 delete namespace
    client.delete("/ci/namespace?namespace_id=test1").await;
    // 8.3 verify the namespace has been deleted
    let response = client.get_resp::<Value>("/ci/namespace?namespace_id=test1").await;
    // since namespace has been deleted, response.code should be 404
    assert_eq!(response.code, "404");
    // 8.4 verify the published config has been deleted
    let response = client.get_resp::<Value>("/ci/cs/config?namespace_id=test1&group=DEFAULT-GROUP&data_id=conf-default").await;
    assert_eq!(response.code, "404");
    dbg!(response);

    // 9. test config history
    // 9.1 publish a config
    let _response = client
        .post::<_, bool>(
            "/ci/cs/config",
            &json!( {
                "content": "测试版本1",
                "group": "DEFAULT-GROUP".to_string(),
                "data_id": "conf-default".to_string(),
                "schema": "toml",
                "namespace_id": "public".to_string(),
            }),
        )
        .await;
    // 9.2 update the config
    let _response = client
        .post::<_, bool>(
            "/ci/cs/config",
            &json!( {
                "content": "测试版本2",
                "group": "DEFAULT-GROUP".to_string(),
                "data_id": "conf-default".to_string(),
                "schema": "toml",
                "namespace_id": "public".to_string(),
            }),
        )
        .await;

    // 9.3 get config history
    let response = client.get::<ConfigHistoryListResponse>("/ci/cs/history/list?namespace_id=public&group=DEFAULT-GROUP&data_id=conf-default").await;
    assert_eq!(response.total_count, 5);
    assert_eq!(response.page_items[0].content, "测试版本2");
    assert_eq!(response.page_items[0].op_type, "U");
    assert_eq!(response.page_items[1].content, "测试版本1");
    assert_eq!(response.page_items[1].op_type, "I");
    assert_eq!(response.page_items[2].op_type, "D");
    assert_eq!(response.page_items[3].op_type, "U");
    assert_eq!(response.page_items[4].op_type, "I");

    // 10. test find certain config history
    // 10.1 upload two new config
    let data_id_1 = "conf-history-test-1";
    let _response = client
        .post::<_, bool>(
            "/ci/cs/config",
            &json!( {
                "content": "历史版本测试1",
                "group": "DEFAULT-GROUP".to_string(),
                "data_id": data_id_1.to_string(),
                "schema": "plaintext",
                "namespace_id": "public".to_string(),
            }),
        )
        .await;
    let _response = client
    .post::<_, bool>(
        "/ci/cs/config",
        &json!( {
            "content": "历史版本测试2",
            "group": "DEFAULT-GROUP".to_string(),
            "data_id": "conf-history-test-2".to_string(),
            "schema": "plaintext",
            "namespace_id": "public".to_string(),
        }),
    )
    .await;
    // 10.2 update the first config
    let _response = client
        .post::<_, bool>(
            "/ci/cs/config",
            &json!( {
                "content": "历史版本测试1-修改",
                "group": "DEFAULT-GROUP".to_string(),
                "data_id": data_id_1.to_string(),
                "schema": "plaintext",
                "namespace_id": "public".to_string(),
            }),
        )
        .await;
    // 10.3 find first config history version 1
    let response = client.get::<ConfigHistoryListResponse>(&format!("/ci/cs/history/list?namespace_id=public&group=DEFAULT-GROUP&data_id={data_id_1}")).await;
    assert_eq!(response.total_count, 2);
    // response.page_items[1];
    // let response = client.get::<ConfigHistoryItem>("/ci/cs/history?namespace_id=public&group=DEFAULT-GROUP&data_id=conf-history-test-1&id=1").await;
    // let response = client.get::<ConfigHistoryListResponse>("/ci/cs/history?namespace_id=public&group=DEFAULT-GROUP&data_id=conf-history-test-2").await;
    
    // wait user press enter
    println!("Press enter to continue...");
    let mut line = String::new();
    io::stdin().read_line(&mut line).unwrap();
    Ok(())
}
