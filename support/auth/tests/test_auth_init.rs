use std::time::Duration;

use bios_auth::{auth_config::AuthConfig, auth_constants::DOMAIN_CODE, auth_initializer, serv::auth_res_serv};
use tardis::{basic::result::TardisResult, chrono::Utc, tokio::time::sleep, web::poem_openapi::types::Type, TardisFuns};

pub async fn test_init() -> TardisResult<()> {
    let config = TardisFuns::cs_config::<AuthConfig>(DOMAIN_CODE);
    let cache_client = TardisFuns::cache_by_module_or_default(DOMAIN_CODE);
    cache_client
        .hset(
            &config.cache_key_res_info,
            "iam-res://iam-serv/p1?a=1##get",
            r###"{"auth":{"accounts":"#acc1#"},"need_crypto_req":false,"need_crypto_resp":false,"need_double_auth":false,"need_login":false}"###,
        )
        .await?;
    cache_client
        .hset(
            &config.cache_key_res_info,
            "iam-res://iam-serv/p1?a=2##get",
            r###"{"auth":{"accounts":"#acc2#"},"need_crypto_req":false,"need_crypto_resp":false,"need_double_auth":false,"need_login":false}"###,
        )
        .await?;
    cache_client
        .hset(
            &config.cache_key_res_info,
            "iam-res://iam-serv/p1?a=3##get",
            r###"{"auth":{"accounts":"#acc3#"},"need_crypto_req":false,"need_crypto_resp":false,"need_double_auth":false,"need_login":false}"###,
        )
        .await?;
    cache_client
        .hset(
            &config.cache_key_res_info,
            "iam-res://iam-serv/p1?a=4##get",
            r###"{"auth":{"accounts":"#acc4#"},"need_crypto_req":false,"need_crypto_resp":false,"need_double_auth":false,"need_login":false}"###,
        )
        .await?;
    cache_client
        .hset(
            &config.cache_key_res_info,
            "iam-res://iam-serv/p1?a=5##get",
            r###"{"auth":{"accounts":"#acc5#"},"need_crypto_req":false,"need_crypto_resp":false,"need_double_auth":false,"need_login":false}"###,
        )
        .await?;

    cache_client
        .hset(
            &config.cache_key_res_info,
            "iam-res://iam-serv/need_double_auth?a=1##get",
            r###"{"auth":{"accounts":"#acc5#"},"need_crypto_req":false,"need_crypto_resp":false,"need_double_auth":true,"need_login":false}"###,
        )
        .await?;
    cache_client
        .hset(
            &config.cache_key_res_info,
            "iam-res://iam-serv/p1?a=6##get",
            r###"{"auth":{"accounts":"#acc6#"},"need_crypto_req":false,"need_crypto_resp":false,"need_double_auth":false,"need_login":false}"###,
        )
        .await?;
    cache_client
        .hset(
            &config.cache_key_res_info,
            "iam-res://iam-serv/p2?a=1##get",
            r###"{"auth":null,"need_crypto_req":true,"need_crypto_resp":true,"need_double_auth":true,"need_login":false}"###,
        )
        .await?;
    cache_client
        .hset(
            &config.cache_key_res_info,
            "iam-res://iam-serv/p2?a=1##*",
            r###"{"auth":null,"need_crypto_req":true,"need_crypto_resp":false,"need_double_auth":true,"need_login":false}"###,
        )
        .await?;
    cache_client
        .hset(
            &config.cache_key_res_info,
            "iam-res://iam-serv/p2?a=2##get",
            r###"{"auth":{"tenant":"#*#","st":1685407354,"et":2685407354},"need_crypto_req":true,"need_crypto_resp":true,"need_double_auth":true,"need_login":false}"###,
        )
        .await?;

    auth_initializer::init_data().await?;

    assert_eq!(
        auth_res_serv::get_res_json()?["children"]["iam-res"]["children"]["iam-serv"]["children"]["p1"]["children"]["?"]["children"]["a=1"]["children"]["$"]["children"]["get"]
            ["leaf_info"]["uri"]
            .as_str()
            .unwrap(),
        "iam-res://iam-serv/p1?a=1"
    );
    let apis = auth_res_serv::get_apis_json()?["apis"].as_array().unwrap().clone();
    let url = apis.iter().filter(|a| a["uri"].as_str().unwrap() == "iam-serv/p1?a=1").collect::<Vec<_>>();
    assert!(url.len() == 1);

    assert_eq!(
        auth_res_serv::get_res_json()?["children"]["iam-res"]["children"]["iam-serv"]["children"]["p1"]["children"]["?"]["children"]["a=5"]["children"]["$"]["children"]["get"]
            ["leaf_info"]["uri"]
            .as_str()
            .unwrap(),
        "iam-res://iam-serv/p1?a=5"
    );
    let url = apis.iter().filter(|a| a["uri"].as_str().unwrap() == "iam-serv/p1?a=5").collect::<Vec<_>>();
    assert!(url.len() == 1);
    let url = apis.iter().filter(|a| a["uri"].as_str().unwrap() == "iam-serv/p1?a=6" && !a["need_crypto_req"].as_bool().unwrap()).collect::<Vec<_>>();
    assert!(url.len() == 1);

    let url = apis
        .iter()
        .filter(|a| {
            a["uri"].as_str().unwrap() == "iam-serv/p2?a=1"
                && a["action"].as_str().unwrap() == "get"
                && a["need_crypto_req"].as_bool().unwrap()
                && a["need_crypto_resp"].as_bool().unwrap()
                && a["need_double_auth"].as_bool().unwrap()
                && !a["need_login"].as_bool().unwrap()
        })
        .collect::<Vec<_>>();
    assert!(url.len() == 1);

    let url = apis
        .iter()
        .filter(|a| {
            a["uri"].as_str().unwrap() == "iam-serv/p2?a=1"
                && a["action"].as_str().unwrap() == "*"
                && a["need_crypto_req"].as_bool().unwrap()
                && !a["need_crypto_resp"].as_bool().unwrap()
                && a["need_double_auth"].as_bool().unwrap()
                && !a["need_login"].as_bool().unwrap()
        })
        .collect::<Vec<_>>();
    assert!(url.len() == 1);

    cache_client.hdel(&config.cache_key_res_info, "iam-res://iam-serv/p1?a=1##get").await?;
    cache_client
        .hset(
            &config.cache_key_res_info,
            "iam-res://iam-serv/p1?a=6##get",
            r###"{"auth":{"accounts":"#acc6#"},"need_crypto_req":true,"need_crypto_resp":false,"need_double_auth":false,"need_login":false}"###,
        )
        .await?;
    cache_client
        .hset(
            &config.cache_key_res_info,
            "iam-res://iam-serv/p1?a=7##get",
            r###"{"auth":{"accounts":"#acc7#"},"need_crypto_req":false,"need_crypto_resp":false,"need_double_auth":false,"need_login":false}"###,
        )
        .await?;
    cache_client.set(&format!("{}iam-res://iam-serv/p1?a=1##get", config.cache_key_res_changed_info), "").await?;
    cache_client.set(&format!("{}iam-res://iam-serv/p1?a=6##get", config.cache_key_res_changed_info), "").await?;
    cache_client.set(&format!("{}iam-res://iam-serv/p1?a=7##get", config.cache_key_res_changed_info), "").await?;

    sleep(Duration::from_secs(2)).await;

    assert!(auth_res_serv::get_res_json()?["children"]["iam-res"]["children"]["iam-serv"]["children"]["p1"]["children"]["?"]["children"].get("a=1").is_none());
    let apis = auth_res_serv::get_apis_json()?["apis"].as_array().unwrap().clone();
    let url = apis.iter().filter(|a| a["uri"].as_str().unwrap() == "iam-serv/p1?a=1").collect::<Vec<_>>();
    assert!(url.is_empty());
    assert_eq!(
        auth_res_serv::get_res_json()?["children"]["iam-res"]["children"]["iam-serv"]["children"]["p1"]["children"]["?"]["children"]["a=6"]["children"]["$"]["children"]["get"]
            ["leaf_info"]["uri"]
            .as_str()
            .unwrap(),
        "iam-res://iam-serv/p1?a=6"
    );
    let url = apis.iter().filter(|a| a["uri"].as_str().unwrap() == "iam-serv/p1?a=6" && a["need_crypto_req"].as_bool().unwrap()).collect::<Vec<_>>();
    assert!(url.len() == 1);

    assert_eq!(
        auth_res_serv::get_res_json()?["children"]["iam-res"]["children"]["iam-serv"]["children"]["p1"]["children"]["?"]["children"]["a=7"]["children"]["$"]["children"]["get"]
            ["leaf_info"]["uri"]
            .as_str()
            .unwrap(),
        "iam-res://iam-serv/p1?a=7"
    );
    let url = apis
        .iter()
        .filter(|a| {
            a["uri"].as_str().unwrap() == "iam-serv/p1?a=7"
                && !a["need_crypto_req"].as_bool().unwrap()
                && !a["need_crypto_resp"].as_bool().unwrap()
                && !a["need_double_auth"].as_bool().unwrap()
                && !a["need_login"].as_bool().unwrap()
        })
        .collect::<Vec<_>>();
    assert!(url.len() == 1);

    let res_json = auth_res_serv::get_res_json()?;
    let st = res_json["children"]["iam-res"]["children"]["iam-serv"]["children"]["p2"]["children"]["?"]["children"]["a=2"]["children"]["$"]["children"]["get"]["leaf_info"]["auth"]
        ["st"]
        .as_i64()
        .unwrap();
    let et = res_json["children"]["iam-res"]["children"]["iam-serv"]["children"]["p2"]["children"]["?"]["children"]["a=2"]["children"]["$"]["children"]["get"]["leaf_info"]["auth"]
        ["et"]
        .as_i64()
        .unwrap();
    let now = Utc::now().timestamp();
    assert!(st < now && now < et && st < et);

    Ok(())
}
