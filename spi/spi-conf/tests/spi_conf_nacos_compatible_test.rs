use std::{collections::HashMap, env};

use bios_basic::{
    rbum::serv::rbum_kind_serv::RbumKindServ,
    spi::{dto::spi_bs_dto::SpiBsAddReq, spi_constants},
    test::{init_rbum_test_container, test_http_client::TestHttpClient},
};
use bios_spi_conf::conf_constants::DOMAIN_CODE;
use tardis::{
    basic::{dto::TardisContext, field::TrimString, result::TardisResult},
    log, testcontainers, tokio,
    web::web_resp::Void,
    TardisFuns,
};

mod spi_conf_test_common;
use spi_conf_test_common::*;

#[tokio::test]
async fn spi_conf_namespace_test() -> TardisResult<()> {
    std::env::set_var("RUST_LOG", "info,tardis=debug,spi_conf_listener_test=debug,sqlx=off,sea_orm=off,bios_spi_conf=DEBUG");
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
    client.set_auth(&TardisContext {
        own_paths: "t1/app001".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "app001".to_string(),
        ..Default::default()
    })?;
    test_tardis_compatibility(&client).await?;
    // web_server_hanlde.await.unwrap()?;
    drop(container_hold);
    Ok(())
}

async fn test_tardis_compatibility(_client: &TestHttpClient) -> TardisResult<()> {
    use tardis::config::config_nacos::nacos_client::*;
    let ctx = TardisContext {
        own_paths: "t1/app001".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "app001".to_string(),
        ..Default::default()
    };
    let ctx_base64 = &TardisFuns::crypto.base64.encode(&TardisFuns::json.obj_to_string(&ctx)?);
    let mut headers = reqwest::header::HeaderMap::new();
    headers.append(TardisFuns::fw_config().web_server.context_conf.context_header_name.as_str(), ctx_base64.parse().unwrap());
    let client = reqwest::ClientBuilder::default().danger_accept_invalid_certs(true).default_headers(headers).build().unwrap();
    let mut nacos_client = NacosClient::new_with_client("https://localhost:8080/spi-conf/nacos", client);

    // nacos_client.
    // client.login("", password)
    let data_id = "default-config";
    let group = "spi-conf-test";
    log::info!("login to nacosmocker");
    nacos_client.login("nacosmocker", "nacosmocker").await.expect("fail to login");
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
    let config = nacos_client.get_config(&config_descriptor).await.expect("fail to get config");
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

    let login_url = "https://localhost:8080/spi-conf/nacos/v1/auth/login";
    let mut form = HashMap::new();
    form.insert("password", "nacosmocker");
    form.insert("username", "nacosmocker");
    let resp = nacos_client.post(login_url).form(&form).send().await?;
    log::info!("response: {resp:#?}");

    let value = resp.json::<tardis::serde_json::Value>().await?;
    let token = value.get("accessToken").expect("missing accessToken").as_str().expect("access_token should be string");
    let namespace_url = "https://localhost:8080/spi-conf/nacos/v1/console/namespaces";
    let mut form = HashMap::new();
    form.insert("customNamespaceId", "test-namespace-1");
    form.insert("namespaceName", "测试命名空间1");
    // publish
    let resp = nacos_client.post(namespace_url).query(&[("accessToken", token)]).form(&form).send().await?;
    log::info!("response: {resp:#?}");
    let success = resp.json::<bool>().await?;
    assert!(success);
    // edit
    let mut form = HashMap::new();
    form.insert("namespace", "test-namespace-1");
    form.insert("namespaceShowName", "测试命名空间1-修改");
    let resp = nacos_client.put(namespace_url).query(&[("accessToken", token)]).form(&form).send().await?;
    // let info = resp.text().await?;
    // log::info!("response: {info}");
    let success = resp.json::<bool>().await?;
    assert!(success);

    // delete
    let mut form = HashMap::new();
    form.insert("namespaceId", "test-namespace-1");
    let resp = nacos_client.delete(namespace_url).query(&[("accessToken", token)]).form(&form).send().await?;
    // let info = resp.text().await?;
    // log::info!("response: {info}");
    let success = resp.json::<bool>().await?;
    assert!(success);
    wait_press_enter();
    Ok(())
}

fn wait_press_enter() {
    use std::io::*;
    let mut buf = String::new();
    println!("Press 'Enter' to continue");
    stdin().read_line(&mut buf).unwrap();
}