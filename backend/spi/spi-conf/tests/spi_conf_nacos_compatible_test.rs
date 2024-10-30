use std::{collections::HashMap, env};

use bios_basic::test::test_http_client::TestHttpClient;
use bios_spi_conf::{
    conf_constants::DOMAIN_CODE,
    dto::conf_auth_dto::{RegisterRequest, RegisterResponse},
};
use tardis::web::reqwest::{self, header::HeaderName};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    log,
    serde_json::json,
    tokio,
    web::web_resp::TardisResp,
    TardisFuns,
};

mod spi_conf_test_common;
use spi_conf_test_common::*;
const SCHEMA: &str = "https";
#[tokio::test(flavor = "multi_thread")]
async fn spi_conf_namespace_test() -> TardisResult<()> {
    std::env::set_var(
        "RUST_LOG",
        "info,tardis=debug,spi_conf_listener_test=debug,sqlx=off,sea_orm=debug,bios_spi_conf=DEBUG,poem_grpc=TRACE,tonic=TRACE",
    );
    std::env::set_var("PROFILE", "nacos");

    let container_hold = init_tardis().await?;
    let _web_server_handle = start_web_server().await;
    let tardis_ctx = TardisContext::default();
    let mut client = TestHttpClient::new(format!("{SCHEMA}://127.0.0.1:8080/spi-conf"));
    client.set_auth(&tardis_ctx)?;
    let RegisterResponse { username, password } = client
        .put(
            "/ci/auth/register_bundle",
            &json!({
                "app_tenant_id": "app001",
                "username": "nacos",
                "backend_service": {
                    "type": "new",
                    "value": {
                        "name": "spi-nacos-app01",
                        "conn_uri": env::var("TARDIS_FW.DB.URL").unwrap(),
                    }
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

    // test register

    test_tardis_compatibility(&client).await?;
    TardisFuns::web_server().await;
    drop(container_hold);
    Ok(())
}

async fn test_tardis_compatibility(_test_client: &TestHttpClient) -> TardisResult<()> {
    use tardis::config::config_nacos::nacos_client::*;
    let config = TardisFuns::fw_config();
    let ctx = TardisContext {
        own_paths: "t1/app001".to_string(),
        ak: "app001".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "app001".to_string(),
        ..Default::default()
    };
    let ctx_base64 = &TardisFuns::crypto.base64.encode(TardisFuns::json.obj_to_string(&ctx)?);
    let mut headers = reqwest::header::HeaderMap::new();
    let web_server_config = config.web_server();
    let context_header_name = web_server_config.context_conf.context_header_name.clone();
    headers.append(HeaderName::from_bytes(context_header_name.as_bytes()).expect("should be ok"), ctx_base64.parse().unwrap());
    let client = reqwest::ClientBuilder::default().danger_accept_invalid_certs(true).default_headers(headers).build().unwrap();
    let mut nacos_client = NacosClient::new_with_client(format!("{SCHEMA}://127.0.0.1:8080/spi-conf-nacos/nacos"), client);
    // register
    let resp = nacos_client.reqwest_client.post(format!("{SCHEMA}://127.0.0.1:8080/spi-conf/ci/auth/register")).json(&RegisterRequest::default()).send().await?;
    let resp = resp.json::<TardisResp<RegisterResponse>>().await?;

    let auth = resp.data.expect("error in register");

    let data_id = "default-config";
    let group = "spi-conf-test";
    log::info!("login to nacos mocker");
    nacos_client.login(&auth.username, &auth.password).await.expect("fail to login");
    // temporary don't supports
    // nacos_client.login("nacosmocker", "nacosmocker").await.expect("fail to login");
    let config_descriptor = NacosConfigDescriptor {
        data_id,
        group,
        tenant: Default::default(),
        md5: Default::default(),
    };
    log::info!("publish config");
    const CONFIG_CONTENT: &str = "config content for test usage emoji🔧中文，双引号\"\">><<&gt;&lt;<>";
    const CONFIG_CONTENT_2: &str = "config content 2";
    let success = nacos_client.publish_config(&config_descriptor, &mut CONFIG_CONTENT.as_bytes()).await.expect("fail to publish config");
    assert!(success);
    log::info!("get config");
    let config_by_basic_auth_resp = nacos_client
        .reqwest_client
        .get(format!("{SCHEMA}://127.0.0.1:8080/spi-conf-nacos/nacos/v1/cs/configs"))
        .query(&config_descriptor)
        .basic_auth(&auth.username, Some(&auth.password))
        .send()
        .await?;
    let config_by_basic_auth = config_by_basic_auth_resp.text().await?;
    log::info!("config_by_basic_auth: {}", &config_by_basic_auth);
    let config = nacos_client.get_config(&config_descriptor).await.expect("fail to get config");
    assert_eq!(&config_by_basic_auth, &config);
    assert_eq!(CONFIG_CONTENT, &config);
    log::info!("delete config");
    let success = nacos_client.delete_config(&config_descriptor).await.unwrap();
    assert!(success);
    log::info!("get deleted config, should be an error");
    let err = nacos_client.get_config(&config_descriptor).await.expect_err("shouldn't get a deleted config");
    log::error!("{err}");
    log::info!("publish config");
    let success = nacos_client.publish_config(&config_descriptor, &mut CONFIG_CONTENT.as_bytes()).await.expect("fail to publish config");
    assert!(success);
    let changed = nacos_client.listen_config(&config_descriptor).await.unwrap();
    assert!(!changed);
    let success = nacos_client.publish_config(&config_descriptor, &mut CONFIG_CONTENT_2.as_bytes()).await.expect("fail to publish config");
    assert!(success);
    let changed = nacos_client.listen_config(&config_descriptor).await.unwrap();
    assert!(changed);

    // namespace api
    log::info!("test namespace api");
    // register a new account
    let username = "nacosmocker";
    let password = "nacosmocker";
    let resp = nacos_client
        .reqwest_client
        .post(format!("{SCHEMA}://127.0.0.1:8080/spi-conf/ci/auth/register"))
        .json(&RegisterRequest {
            username: Some(username.into()),
            password: Some(password.into()),
        })
        .send()
        .await?;
    let resp = resp.json::<TardisResp<RegisterResponse>>().await?;
    let auth = resp.data.expect("error in register");
    assert_eq!(username, auth.username);
    assert_eq!(password, auth.password);
    let login_url = format!("{SCHEMA}://127.0.0.1:8080/spi-conf-nacos/nacos/v1/auth/login");
    let mut form = HashMap::new();
    form.insert("password", username);
    form.insert("username", password);
    let resp = nacos_client.reqwest_client.post(login_url).form(&form).send().await?;
    log::info!("response: {resp:#?}");

    let value = resp.json::<tardis::serde_json::Value>().await?;
    let token = value.get("accessToken").expect("missing accessToken").as_str().expect("access_token should be string");
    let namespace_url = format!("{SCHEMA}://127.0.0.1:8080/spi-conf-nacos/nacos/v1/console/namespaces");
    let mut form = HashMap::new();
    form.insert("customNamespaceId", "test-namespace-1");
    form.insert("namespaceName", "测试命名空间1");
    form.insert("username", username);
    form.insert("password", password);
    // publish
    let resp = nacos_client.reqwest_client.post(&namespace_url).form(&form).send().await?;
    log::info!("response: {resp:#?}");
    let success = resp.json::<bool>().await?;
    assert!(success);
    // edit
    let mut form = HashMap::new();
    form.insert("namespace", "test-namespace-1");
    form.insert("namespaceShowName", "测试命名空间1-修改");
    let resp = nacos_client.reqwest_client.put(&namespace_url).query(&[("accessToken", token)]).form(&form).send().await?;
    // let info = resp.text().await?;
    // log::info!("response: {info}");
    let success = resp.json::<bool>().await?;
    assert!(success);

    // delete
    let mut form = HashMap::new();
    form.insert("namespaceId", "test-namespace-1");
    let resp = nacos_client.reqwest_client.delete(&namespace_url).query(&[("accessToken", token)]).form(&form).send().await?;
    // let info = resp.text().await?;
    // log::info!("response: {info}");
    let success = resp.json::<bool>().await?;
    assert!(success);

    let _resp = nacos_client
        .publish_config(
            &NacosConfigDescriptor::new("hc-db.yaml", "hc", &(Default::default())),
            &mut std::fs::File::open("tests/config/conf-nacos.toml").expect("fail to open"),
        )
        .await
        .expect("publish failed");
    Ok(())
}
