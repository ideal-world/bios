use bios_basic::test::test_http_client::TestHttpClient;
use bios_spi_conf::{
    conf_constants::DOMAIN_CODE,
    dto::{
        conf_auth_dto::RegisterResponse,
        conf_config_dto::{ConfigItem, ConfigItemDigest, ConfigListResponse},
        conf_namespace_dto::{NamespaceAttribute, NamespaceItem},
    },
};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    log,
    serde_json::{json, Value},
    tokio, TardisFuns,
};
mod spi_conf_test_common;
use spi_conf_test_common::*;

#[tokio::test]
async fn spi_conf_namespace_test() -> TardisResult<()> {
    std::env::set_var("RUST_LOG", "info,sqlx=off,sea_orm=info,spi_conf_namespace_test=info,bios_spi_conf=TRACE");

    let container_hold = init_tardis().await?;
    start_web_server().await?;
    let mut client = TestHttpClient::new("https://127.0.0.1:8080/spi-conf".to_string());
    client.set_auth(&TardisContext {
        own_paths: "t1/app001".to_string(),
        // ak: "app001".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "app001".to_string(),
        ..Default::default()
    })?;
    let _funs = TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None);
    let RegisterResponse { username, password } = client
        .put(
            "/ci/auth/register_bundle",
            &json!({
                "username": "nacos",
                "backend_service": {
                    "type": "new"
                    // "value": {
                    //     "name": "spi-nacos-app01",
                    // }
                }
            }),
        )
        .await;
    log::info!("username: {username}, password: {password}");
    client.set_auth(&TardisContext {
        own_paths: "t1/app001".to_string(),
        ak: "app001".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "app001".to_string(),
        ..Default::default()
    })?;
    test_register(&mut client).await?;
    test_curd(&mut client).await?;
    test_tags(&mut client).await?;

    // web_server_handle.await.unwrap()?;
    drop(container_hold);
    Ok(())
}

pub async fn test_curd(client: &mut TestHttpClient) -> TardisResult<()> {
    let user = client.context().owner.as_str();
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
    let response = client.get::<ConfigItem>("/ci/cs/config/detail?namespace_id=public&group=DEFAULT-GROUP&data_id=conf-default").await;
    assert_eq!(response.data_id, "conf-default");
    assert_eq!(response.src_user.unwrap(), user);
    // 4. get namespace info
    let _response = client.get::<NamespaceItem>("/ci/namespace?namespace_id=public").await;
    assert_eq!(_response.config_count, 1);
    // 4.1 get namespace list
    let _response = client.get::<Vec<NamespaceItem>>("/ci/namespace/list").await;
    assert_eq!(_response.len(), 4);
    // since we have published a config, the config_count should be 1
    // 5. delete config
    client.delete("/ci/cs/config?namespace_id=public&group=DEFAULT-GROUP&data_id=conf-default").await;
    // 6. get namespace info
    let _response = client.get::<NamespaceItem>("/ci/namespace?namespace_id=public").await;
    // since we have deleted the config, the config_count should be 0
    assert_eq!(_response.config_count, 0);
    // 7. update namespace
    // 7.1
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
    // verify the namespace_desc has been updated
    let response = client.get::<NamespaceItem>("/ci/namespace?namespace_id=test1").await;
    assert_eq!(&response.namespace_desc.unwrap(), "测试命名空间1-修改");
    // 7.2 modify namespace does not exist
    let response = client
        .put_resp::<_, bool>(
            "/ci/namespace",
            &NamespaceAttribute {
                namespace: "not-exsist-namespace".to_string(),
                namespace_show_name: "测试命名空间1".to_string(),
                namespace_desc: Some("测试命名空间1-修改".to_string()),
            },
        )
        .await;
    assert!(response.code.contains("404"));
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
    // skip verify because it will panic when 404 is returned. it won't be fixup until we can ban uniform error mw on some distinct api
    #[allow(unreachable_code)]
    'skip: {
        break 'skip;
        // 8.3 verify the namespace has been deleted
        let response = client.get_resp::<Value>("/ci/namespace?namespace_id=test1").await;
        // since namespace has been deleted, response.code should be 404
        assert_eq!(response.code, "404");
        // 8.4 verify the published config has been deleted
        let response = client.get_resp::<Value>("/ci/cs/config?namespace_id=test1&group=DEFAULT-GROUP&data_id=conf-default").await;
        assert_eq!(response.code, "404");
    }

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
    let response = client.get::<ConfigListResponse>("/ci/cs/history/list?namespace_id=public&group=DEFAULT-GROUP&data_id=conf-default").await;
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
    let response = client.get::<ConfigListResponse>(&format!("/ci/cs/history/list?namespace_id=public&group=DEFAULT-GROUP&data_id={data_id_1}")).await;
    assert_eq!(response.total_count, 2);
    let his_id_1 = &response.page_items[1].id;
    let his_id_2 = &response.page_items[0].id;
    let response_1 = client.get::<ConfigItem>(&format!("/ci/cs/history?namespace_id=public&group=DEFAULT-GROUP&data_id=conf-history-test-1&id={his_id_1}")).await;
    assert_eq!(response_1.content, "历史版本测试1");
    // src_user should be app001
    assert_eq!(response_1.src_user.unwrap(), "app001");
    // 10.4 find first config history previous to version 2 (should be version 1)
    let response_prev_2 = client
        .get::<ConfigItem>(&format!(
            "/ci/cs/history/previous?namespace_id=public&group=DEFAULT-GROUP&data_id=conf-history-test-1&id={his_id_2}"
        ))
        .await;
    assert_eq!(response_prev_2.content, "历史版本测试1");
    assert_eq!(response_prev_2.id, *his_id_1);
    // 10.5 find first config history previous to version 1 (should not found)
    let response_prev_1 = client
        .get_resp::<ConfigItem>(&format!(
            "/ci/cs/history/previous?namespace_id=public&group=DEFAULT-GROUP&data_id=conf-history-test-1&id={his_id_1}"
        ))
        .await;
    assert_eq!(response_prev_1.code, "404");
    // 11. test get config by namespace
    // 11.1 create a namespace
    const NAMESPACE_ID: &str = "test-get-config-by-namespace";
    let _response = client
        .post::<_, bool>(
            "/ci/namespace",
            &NamespaceAttribute {
                namespace: NAMESPACE_ID.into(),
                namespace_show_name: "测试命名空间1".to_string(),
                namespace_desc: Some("测试命名空间1".to_string()),
            },
        )
        .await;
    // namespace should have 0 configs, verifies that the namespace is empty
    let response = client.get::<Vec<ConfigItemDigest>>(&format!("/ci/cs/history/configs?namespace_id={}", NAMESPACE_ID)).await;
    assert_eq!(response.len(), 0);
    // 11.2 create 3 config in the namespace
    for i in 0..3 {
        let _response = client
            .post::<_, bool>(
                "/ci/cs/config",
                &json!( {
                    "content": format!("测试命名空间1-配置{}", i),
                    "group": "DEFAULT-GROUP".to_string(),
                    "data_id": format!("conf-{}", i),
                    "schema": "plaintext",
                    "namespace_id": NAMESPACE_ID.to_string(),
                }),
            )
            .await;
    }
    // 11.3 get all config in the namespace
    let response = client.get::<Vec<ConfigItemDigest>>(&format!("/ci/cs/history/configs?namespace_id={}", NAMESPACE_ID)).await;
    assert_eq!(response.len(), 3);
    // those configs should be sorted by create time
    assert_eq!(response[0].data_id, "conf-2");
    assert_eq!(response[1].data_id, "conf-1");
    assert_eq!(response[2].data_id, "conf-0");

    // 12 test .env file
    const ENV_TEST_NAMESPACE_ID: &str = "test config";
    let _response = client
        .post::<_, bool>(
            "/ci/namespace",
            &NamespaceAttribute {
                namespace: ENV_TEST_NAMESPACE_ID.into(),
                namespace_show_name: "测试环境变量命名空间".to_string(),
                namespace_desc: Some("测试环境变量命名空间".to_string()),
            },
        )
        .await;
    let _response = client
        .post::<_, bool>(
            "/ci/cs/config",
            &json!( {
                "content": r#"
TYPE=ALPHA
VALUE=123
URL=http://www.baidu.com
# this is a comment
"#,
                "group": "DEFAULT-GROUP".to_string(),
                "data_id": ".env".to_string(),
                "schema": "env",
                "namespace_id": NAMESPACE_ID.to_string(),
            }),
        )
        .await;
    let config_to_be_render = r#"
[conf]
type=$ENV{TYPE}
value=$ENV{VALUE}
url=$ENV{URL}
"#;
    let _response = client
        .post::<_, bool>(
            "/ci/cs/config",
            &json!( {
                "content": config_to_be_render,
                "namespace_id": NAMESPACE_ID.to_string(),
                "group": "DEFAULT-GROUP".to_string(),
                "data_id": "conf-env".to_string(),
                "schema": "toml",
            }),
        )
        .await;
    let response = client.get::<ConfigItem>(&format!("/ci/cs/config/detail?namespace_id={NAMESPACE_ID}&group=DEFAULT-GROUP&data_id=conf-env")).await;
    assert_eq!(
        response.content,
        r#"
[conf]
type=ALPHA
value=123
url=http://www.baidu.com
"#
    );
    // UPDATE the .env file
    let _response = client
        .post::<_, bool>(
            "/ci/cs/config",
            &json!( {
                "content": r#"
TYPE=BETA
VALUE=456
URL=http://www.google.com
"#,
                "group": "DEFAULT-GROUP".to_string(),
                "data_id": ".env".to_string(),
                "schema": "env",
                "namespace_id": NAMESPACE_ID.to_string(),
            }),
        )
        .await;
    let response = client.get::<ConfigItem>(&format!("/ci/cs/config/detail?namespace_id={NAMESPACE_ID}&group=DEFAULT-GROUP&data_id=conf-env")).await;
    assert_eq!(
        response.content,
        r#"
[conf]
type=BETA
value=456
url=http://www.google.com
"#
    );

    Ok(())
}

pub async fn test_tags(client: &mut TestHttpClient) -> TardisResult<()> {
    const DATA_ID: &str = "conf-tag-test";
    // 1. publish a config with tags
    let _response = client
        .post::<_, bool>(
            "/ci/cs/config",
            &json!( {
                "content": "for tag test",
                "group": "DEFAULT-GROUP".to_string(),
                "data_id": DATA_ID,
                "schema": "toml",
                "namespace_id": "public".to_string(),
                "config_tags": ["tag1", "tag2"],
            }),
        )
        .await;
    // 2. verify the tags
    let response = client.get::<ConfigItem>(&format!("/ci/cs/config/detail?namespace_id=public&group=DEFAULT-GROUP&data_id={DATA_ID}")).await;
    assert!(response.config_tags.contains(&"tag1".to_string()));
    assert!(response.config_tags.contains(&"tag2".to_string()));
    // 3. update the config with new tags
    let _response = client
        .post::<_, bool>(
            "/ci/cs/config",
            &json!( {
                "content": "for tag test",
                "group": "DEFAULT-GROUP".to_string(),
                "data_id": DATA_ID,
                "schema": "toml",
                "namespace_id": "public".to_string(),
                "config_tags": ["tag2", "tag3", "tag4"],
            }),
        )
        .await;
    // 4. verify the tags
    let response = client.get::<ConfigItem>(&format!("/ci/cs/config/detail?namespace_id=public&group=DEFAULT-GROUP&data_id={DATA_ID}")).await;
    assert!(!response.config_tags.contains(&"tag1".to_string()));
    assert!(response.config_tags.contains(&"tag2".to_string()));
    assert!(response.config_tags.contains(&"tag3".to_string()));
    // 5. check if history has tags
    let response = client.get::<ConfigListResponse>(&format!("/ci/cs/history/list?namespace_id=public&group=DEFAULT-GROUP&data_id={DATA_ID}")).await;
    assert_eq!(response.total_count, 2);
    // wait_press_enter();
    assert!(response.page_items[0].config_tags.contains(&"tag2".to_string()));
    assert!(response.page_items[0].config_tags.contains(&"tag3".to_string()));
    assert!(!response.page_items[0].config_tags.contains(&"tag1".to_string()));
    assert!(response.page_items[1].config_tags.contains(&"tag1".to_string()));
    assert!(response.page_items[1].config_tags.contains(&"tag2".to_string()));
    // 6. search by tags
    let _response = client
        .post::<_, bool>(
            "/ci/cs/config",
            &json!( {
                "content": "for tag test",
                "group": "DEFAULT-GROUP".to_string(),
                "data_id": format!("{}-{}", DATA_ID, 2),
                "schema": "toml",
                "namespace_id": "public".to_string(),
                "config_tags": ["tag2", "tag3"],
            }),
        )
        .await;
    let response = client.get::<ConfigListResponse>("/ci/cs/configs?tags=tag2,tag3").await;
    assert_eq!(response.total_count, 2);
    assert!(response.page_items.iter().any(|item| item.data_id == DATA_ID));
    assert!(response.page_items.iter().any(|item| item.data_id == format!("{}-{}", DATA_ID, 2)));
    assert!(response.page_items.iter().all(|item| item.config_tags.contains(&"tag2".to_owned()) && item.config_tags.contains(&"tag3".to_owned())));
    let response = client.get::<ConfigListResponse>(&format!("/ci/cs/configs?tags=tag2&data_id={DATA_ID}&mode=fuzzy")).await;
    assert_eq!(response.total_count, 2);
    let response = client.get::<ConfigListResponse>(&format!("/ci/cs/configs?tags=tag4&data_id={DATA_ID}&mode=fuzzy")).await;
    assert_eq!(response.total_count, 1);
    Ok(())
}

pub async fn test_register(client: &mut TestHttpClient) -> TardisResult<()> {
    let RegisterResponse { username, password } = client.post("/ci/auth/register", &json!({})).await;
    log::info!("username: {username}, password: {password}");
    let resp = client.post_resp::<_, RegisterResponse>("/ci/auth/register", &json!({ "username": username })).await;
    // should be 409 conflict
    assert!(resp.code.contains("409"));
    Ok(())
}
#[allow(dead_code)]
pub fn wait_press_enter() {
    println!("Press ENTER to continue...");
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
}
