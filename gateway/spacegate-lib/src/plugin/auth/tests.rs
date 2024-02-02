use std::env;

use bios_auth::auth_constants;

use super::*;
use spacegate_shell::config::gateway_dto::SgParameters;
use spacegate_shell::http::{Method, Uri, Version};
use spacegate_shell::hyper::{self, Body, StatusCode};
use tardis::basic::dto::TardisContext;
use tardis::crypto::crypto_sm2_4::{TardisCryptoSm2, TardisCryptoSm2PrivateKey};
use tardis::{
    test::test_container::TardisTestContainer,
    testcontainers::{self, clients::Cli, Container},
    tokio,
};
use testcontainers_modules::redis::Redis;

#[tokio::test]
async fn test() {
    env::set_var("RUST_LOG", "info,bios_spacegate=trace,bios_auth=trace,tardis=trace");
    tracing_subscriber::fmt::init();

    let docker = testcontainers::clients::Cli::default();
    let _x = docker_init(&docker).await.unwrap();

    test_auth_plugin_ctx().await;
    test_auth_plugin_crypto().await;
    test_auth_plugin_strict_security_mode_crypto().await;
}

async fn test_auth_plugin_ctx() {
    log::info!("========test_auth_plugin_ctx=====");

    let mut filter_auth = SgFilterAuth {
        cache_url: env::var("TARDIS_FW.CACHE.URL").unwrap(),
        ..Default::default()
    };

    filter_auth
        .init(&SgPluginFilterInitDto {
            gateway_name: "".to_string(),
            gateway_parameters: SgParameters {
                redis_url: None,
                log_level: None,
                lang: None,
                ignore_tls_verification: None,
            },
            http_route_rules: vec![],
            attached_level: spacegate_shell::plugins::filters::SgAttachedLevel::Gateway,
        })
        .await
        .unwrap();

    let cache_client = TardisFuns::cache_by_module_or_default(auth_constants::DOMAIN_CODE);

    let mut header = HeaderMap::new();
    header.insert("Bios-Token", "aaa".parse().unwrap());
    let ctx = SgRoutePluginContext::new_http(
        Method::POST,
        Uri::from_static("http://sg.idealworld.group/test1"),
        Version::HTTP_11,
        header,
        Body::from("test"),
        "127.0.0.1:8080".parse().unwrap(),
        "".to_string(),
        None,
    );
    let (is_ok, mut before_filter_ctx) = filter_auth.on_req("", ctx).await.unwrap();
    assert!(!is_ok);
    let req_body = before_filter_ctx.response.take_body_into_bytes().await.unwrap();
    let req_body = String::from_utf8_lossy(&req_body).to_string();
    assert!(!req_body.is_empty());
    assert_eq!(req_body, "{\"code\":\"401-gateway-cert-error\",\"message\":\"[Auth] Token [aaa] is not legal\"}");

    cache_client.set(&format!("{}tokenxxx", filter_auth.auth_config.cache_key_token_info), "default,accountxxx").await.unwrap();
    cache_client
        .hset(
            &format!("{}accountxxx", filter_auth.auth_config.cache_key_account_info),
            "",
            "{\"own_paths\":\"\",\"owner\":\"account1\",\"roles\":[\"r001\"],\"groups\":[\"g001\"]}",
        )
        .await
        .unwrap();

    let mut header = HeaderMap::new();
    header.insert("Bios-Token", "tokenxxx".parse().unwrap());
    let ctx = SgRoutePluginContext::new_http(
        Method::POST,
        Uri::from_static("http://sg.idealworld.group/test1"),
        Version::HTTP_11,
        header,
        Body::from("test"),
        "127.0.0.1:8080".parse().unwrap(),
        "".to_string(),
        None,
    );
    let (is_ok, before_filter_ctx) = filter_auth.on_req("", ctx).await.unwrap();
    assert!(is_ok);
    let ctx = decode_context(before_filter_ctx.request.get_headers());

    assert_eq!(ctx.own_paths, "");
    assert_eq!(ctx.owner, "account1");
    assert_eq!(ctx.roles, vec!["r001"]);
    assert_eq!(ctx.groups, vec!["g001"]);

    cache_client.set(&format!("{}tokenxxx", filter_auth.auth_config.cache_key_token_info), "default,accountxxx").await.unwrap();
    cache_client
        .hset(
            &format!("{}accountxxx", filter_auth.auth_config.cache_key_account_info),
            "",
            "{\"own_paths\":\"tenant1\",\"owner\":\"account1\",\"roles\":[\"r001\"],\"groups\":[\"g001\"]}",
        )
        .await
        .unwrap();
    let mut header = HeaderMap::new();
    header.insert("Bios-Token", "tokenxxx".parse().unwrap());
    let ctx = SgRoutePluginContext::new_http(
        Method::POST,
        Uri::from_static("http://sg.idealworld.group/test1"),
        Version::HTTP_11,
        header,
        Body::from("test"),
        "127.0.0.1:8080".parse().unwrap(),
        "".to_string(),
        None,
    );
    let (is_ok, before_filter_ctx) = filter_auth.on_req("", ctx).await.unwrap();
    assert!(is_ok);
    let ctx = decode_context(before_filter_ctx.request.get_headers());

    assert_eq!(ctx.own_paths, "tenant1");
    assert_eq!(ctx.owner, "account1");
    assert_eq!(ctx.roles, vec!["r001"]);
    assert_eq!(ctx.groups, vec!["g001"]);
}

async fn test_auth_plugin_crypto() {
    log::info!("========test_auth_plugin_crypto=====");

    let mut filter_auth = SgFilterAuth {
        cache_url: env::var("TARDIS_FW.CACHE.URL").unwrap(),
        ..Default::default()
    };

    filter_auth
        .init(&SgPluginFilterInitDto {
            gateway_name: "".to_string(),
            gateway_parameters: SgParameters {
                redis_url: None,
                log_level: None,
                lang: None,
                ignore_tls_verification: None,
            },
            http_route_rules: vec![],
            attached_level: spacegate_shell::plugins::filters::SgAttachedLevel::Gateway,
        })
        .await
        .unwrap();

    let ctx = SgRoutePluginContext::new_http(
        Method::GET,
        Uri::from_str(&format!("http://sg.idealworld.group{}", filter_auth.fetch_server_config_path)).unwrap(),
        Version::HTTP_11,
        HeaderMap::new(),
        Body::from(""),
        "127.0.0.1:8080".parse().unwrap(),
        "".to_string(),
        None,
    );
    let (_, mut before_filter_ctx) = filter_auth.on_req("", ctx).await.unwrap();
    let mut server_config_resp = before_filter_ctx.build_response().await.unwrap();
    let data: Value = serde_json::from_str(&String::from_utf8_lossy(
        &hyper::body::to_bytes(server_config_resp.body_mut()).await.unwrap().iter().cloned().collect::<Vec<u8>>(),
    ))
    .unwrap();

    let pub_key = data["data"]["pub_key"].as_str().unwrap();
    let server_sm2 = TardisCryptoSm2 {};
    let server_public_key = server_sm2.new_public_key_from_public_key(pub_key).unwrap();

    let front_pri_key = TardisFuns::crypto.sm2.new_private_key().unwrap();
    let front_pub_key = TardisFuns::crypto.sm2.new_public_key(&front_pri_key).unwrap();

    let test_body_value = r#"test_body_value!@#$%^&*():"中文测试"#;
    //dont need to decrypt
    let header = HeaderMap::new();
    let ctx = SgRoutePluginContext::new_http(
        Method::POST,
        Uri::from_static("http://sg.idealworld.group/test1"),
        Version::HTTP_11,
        header,
        Body::from(test_body_value),
        "127.0.0.1:8080".parse().unwrap(),
        "".to_string(),
        None,
    );
    let (is_ok, mut before_filter_ctx) = filter_auth.on_req("", ctx).await.unwrap();
    assert!(is_ok);
    let req_body = before_filter_ctx.request.dump_body().await.unwrap();
    assert!(!req_body.is_empty());
    let req_body = req_body.to_vec();
    let req_body = String::from_utf8(req_body).unwrap();
    assert_eq!(req_body, test_body_value.to_string());

    //=========request GET============
    let mut header = HeaderMap::new();
    let (_crypto_data, bios_crypto_value) = crypto_req("", server_public_key.serialize().unwrap().as_ref(), front_pub_key.serialize().unwrap().as_ref(), true);
    header.insert("Bios-Crypto", bios_crypto_value.parse().unwrap());
    let ctx = SgRoutePluginContext::new_http(
        Method::GET,
        Uri::from_static("http://sg.idealworld.group/test1"),
        Version::HTTP_11,
        header,
        Body::empty(),
        "127.0.0.1:8080".parse().unwrap(),
        "".to_string(),
        None,
    );
    let (is_ok, mut before_filter_ctx) = filter_auth.on_req("", ctx).await.unwrap();
    assert!(is_ok);
    let req_body = before_filter_ctx.request.dump_body().await.unwrap();
    assert!(req_body.is_empty());

    //=========request POST============
    let mut header = HeaderMap::new();
    let (crypto_data, bios_crypto_value) = crypto_req(
        test_body_value,
        server_public_key.serialize().unwrap().as_ref(),
        front_pub_key.serialize().unwrap().as_ref(),
        true,
    );
    header.insert("Bios-Crypto", bios_crypto_value.parse().unwrap());
    let ctx = SgRoutePluginContext::new_http(
        Method::POST,
        Uri::from_static("http://sg.idealworld.group/test1"),
        Version::HTTP_11,
        header,
        Body::from(crypto_data),
        "127.0.0.1:8080".parse().unwrap(),
        "".to_string(),
        None,
    );
    let (is_ok, mut before_filter_ctx) = filter_auth.on_req("", ctx).await.unwrap();
    assert!(is_ok);
    let req_body = before_filter_ctx.request.dump_body().await.unwrap();
    assert!(!req_body.is_empty());
    let req_body = req_body.to_vec();
    let req_body = String::from_utf8(req_body).unwrap();
    assert_eq!(req_body, test_body_value.to_string());

    //======response============
    let mock_resp = r#"mock_resp:test_body_value!@#$%^&*():"中文测试"#;
    let mut header = HeaderMap::new();
    header.insert("Test_Header", "test_header".parse().unwrap());
    let ctx = before_filter_ctx.resp(StatusCode::OK, header, Body::from(mock_resp));

    let (is_ok, mut before_filter_ctx) = filter_auth.resp_filter("", ctx).await.unwrap();
    assert!(is_ok);
    let resp_body = before_filter_ctx.response.dump_body().await.unwrap();
    assert!(!resp_body.is_empty());
    let resp_body = resp_body.to_vec();
    let resp_body = String::from_utf8(resp_body).unwrap();
    let resp_body = crypto_resp(
        &resp_body,
        before_filter_ctx.response.get_headers().get("Bios-Crypto").unwrap().to_str().unwrap(),
        &front_pri_key,
    );
    println!("req_body:{req_body} mock_resp:{mock_resp}");
    assert_eq!(resp_body, mock_resp.to_string());

    filter_auth.destroy().await.unwrap();
}

async fn test_auth_plugin_strict_security_mode_crypto() {
    log::info!("======test_auth_plugin_strict_security_mode_crypto====");

    let mut filter_auth = SgFilterAuth {
        cache_url: env::var("TARDIS_FW.CACHE.URL").unwrap(),
        ..Default::default()
    };
    filter_auth.auth_config.strict_security_mode = true;

    filter_auth
        .init(&SgPluginFilterInitDto {
            gateway_name: "".to_string(),
            gateway_parameters: SgParameters {
                redis_url: None,
                log_level: None,
                lang: None,
                ignore_tls_verification: None,
            },
            http_route_rules: vec![],
            attached_level: spacegate_shell::plugins::filters::SgAttachedLevel::Gateway,
        })
        .await
        .unwrap();

    let ctx = SgRoutePluginContext::new_http(
        Method::GET,
        Uri::from_str(&format!("http://sg.idealworld.group{}", filter_auth.fetch_server_config_path)).unwrap(),
        Version::HTTP_11,
        HeaderMap::new(),
        Body::empty(),
        "127.0.0.1:8080".parse().unwrap(),
        "".to_string(),
        None,
    );
    let (_, mut before_filter_ctx) = filter_auth.on_req("", ctx).await.unwrap();
    let mut server_config_resp = before_filter_ctx.build_response().await.unwrap();
    let data: Value = serde_json::from_str(&String::from_utf8_lossy(
        &hyper::body::to_bytes(server_config_resp.body_mut()).await.unwrap().iter().cloned().collect::<Vec<u8>>(),
    ))
    .unwrap();

    let pub_key = data["data"]["pub_key"].as_str().unwrap();
    let server_sm2 = TardisCryptoSm2 {};
    let server_public_key = server_sm2.new_public_key_from_public_key(pub_key).unwrap();

    let front_pri_key = TardisFuns::crypto.sm2.new_private_key().unwrap();
    let front_pub_key = TardisFuns::crypto.sm2.new_public_key(&front_pri_key).unwrap();

    //=========request GET by apis============
    let true_path = "get_path";
    let body = MixRequestBody {
        method: "GET".to_string(),
        uri: true_path.to_string(),
        body: "".to_string(),
        headers: Default::default(),
        ts: 0.0,
    };
    let mix_body = TardisFuns::json.obj_to_string(&body).unwrap();
    let mut header = HeaderMap::new();
    let (crypto_body, bios_crypto_value) = crypto_req(
        &mix_body,
        server_public_key.serialize().unwrap().as_ref(),
        front_pub_key.serialize().unwrap().as_ref(),
        true,
    );
    header.insert("Bios-Crypto", bios_crypto_value.parse().unwrap());
    header.insert(hyper::header::CONTENT_LENGTH, crypto_body.as_bytes().len().to_string().parse().unwrap());
    let ctx = SgRoutePluginContext::new_http(
        Method::POST,
        Uri::from_str(&format!("http://sg.idealworld.group/{}", filter_auth.mix_replace_url)).unwrap(),
        Version::HTTP_11,
        header,
        Body::from(crypto_body),
        "127.0.0.1:8080".parse().unwrap(),
        "".to_string(),
        None,
    );
    let (is_ok, before_filter_ctx) = filter_auth.on_req("", ctx).await.unwrap();
    assert!(!is_ok);
    assert_eq!(before_filter_ctx.get_action(), &SgRouteFilterRequestAction::Redirect);
    assert_eq!(before_filter_ctx.request.get_uri().path(), &format!("/{}", true_path));
    assert_eq!(before_filter_ctx.request.get_method(), &Method::GET);
    assert_eq!(
        before_filter_ctx.request.get_headers().get(hyper::header::CONTENT_LENGTH),
        Some(&HeaderValue::from_static("0"))
    );
    let (is_ok, mut before_filter_ctx) = filter_auth.on_req("", before_filter_ctx).await.unwrap();
    assert!(is_ok);
    println!("before_filter_ctx=={:?}", before_filter_ctx);
    let req_body = before_filter_ctx.request.dump_body().await.unwrap();
    assert!(req_body.is_empty());

    filter_auth.destroy().await.unwrap();
}

fn decode_context(headers: &HeaderMap) -> TardisContext {
    let config = TardisFuns::cs_config::<AuthConfig>(auth_constants::DOMAIN_CODE);
    let ctx = headers.get(&config.head_key_context).unwrap();
    let ctx = TardisFuns::crypto.base64.decode_to_string(ctx.to_str().unwrap()).unwrap();
    TardisFuns::json.str_to_obj(&ctx).unwrap()
}

fn crypto_req(body: &str, serv_pub_key: &str, front_pub_key: &str, need_crypto_resp: bool) -> (String, String) {
    let pub_key = TardisFuns::crypto.sm2.new_public_key_from_public_key(serv_pub_key).unwrap();

    let sm4_key = TardisFuns::crypto.key.rand_16_bytes();
    let sm4_key_hex = TardisFuns::crypto.hex.encode(sm4_key);
    let sm4_iv = TardisFuns::crypto.key.rand_16_bytes();
    let sm4_iv_hex = TardisFuns::crypto.hex.encode(sm4_key);

    let data = TardisFuns::crypto.sm4.encrypt_cbc(body, sm4_key, sm4_iv).unwrap();
    let sign_data = TardisFuns::crypto.digest.sm3(&data).unwrap();

    let sm4_encrypt = if need_crypto_resp {
        pub_key.encrypt(&format!("{sign_data} {sm4_key_hex} {sm4_iv_hex} {front_pub_key}",)).unwrap()
    } else {
        pub_key.encrypt(&format!("{sign_data} {sm4_key_hex} {sm4_iv_hex}",)).unwrap()
    };
    let base64_encrypt = TardisFuns::crypto.base64.encode(sm4_encrypt);
    (data, base64_encrypt)
}

fn crypto_resp(body: &str, crypto_header: &str, front_pri_key: &TardisCryptoSm2PrivateKey) -> String {
    let decode_base64 = TardisFuns::crypto.base64.decode_to_string(crypto_header).unwrap();
    let decrypt_key = front_pri_key.decrypt(&decode_base64).unwrap();
    let splits: Vec<_> = decrypt_key.split(' ').collect();
    if splits.len() != 3 {
        panic!("splits:{:?}", splits);
    }

    let sign_data = splits[0];
    let sm4_key = splits[1];
    let sm4_iv = splits[2];
    let gen_sign_data = TardisFuns::crypto.digest.sm3(body).unwrap();
    assert_eq!(sign_data, gen_sign_data);
    TardisFuns::crypto.sm4.decrypt_cbc(body, sm4_key, sm4_iv).unwrap()
}

pub struct LifeHold<'a> {
    pub redis: Container<'a, Redis>,
}

async fn docker_init(docker: &Cli) -> TardisResult<LifeHold<'_>> {
    let redis_container = TardisTestContainer::redis_custom(docker);
    let port = redis_container.get_host_port_ipv4(6379);
    let url = format!("redis://127.0.0.1:{port}/0",);
    env::set_var("TARDIS_FW.CACHE.URL", url);

    Ok(LifeHold { redis: redis_container })
}
