use std::time::Duration;

use bios_auth::{auth_config::AuthConfig, auth_constants::DOMAIN_CODE, auth_initializer, serv::auth_res_serv};
use tardis::{basic::result::TardisResult, tokio::time::sleep, TardisFuns};

pub async fn test_init() -> TardisResult<()> {
    let config = TardisFuns::cs_config::<AuthConfig>(DOMAIN_CODE);
    let cache_client = TardisFuns::cache_by_module_or_default(DOMAIN_CODE);
    cache_client.hset(&config.cache_key_res_info, "iam-res://iam-serv/p1?a=1##get", "{\"accounts\":\"#acc1#\"}").await?;
    cache_client.hset(&config.cache_key_res_info, "iam-res://iam-serv/p1?a=2##get", "{\"accounts\":\"#acc2#\"}").await?;
    cache_client.hset(&config.cache_key_res_info, "iam-res://iam-serv/p1?a=3##get", "{\"accounts\":\"#acc3#\"}").await?;
    cache_client.hset(&config.cache_key_res_info, "iam-res://iam-serv/p1?a=4##get", "{\"accounts\":\"#acc4#\"}").await?;
    cache_client.hset(&config.cache_key_res_info, "iam-res://iam-serv/p1?a=5##get", "{\"accounts\":\"#acc5#\"}").await?;

    auth_initializer::init_data().await?;

    assert_eq!(
        auth_res_serv::get_res_json()?["children"]["iam-res"]["children"]["iam-serv"]["children"]["p1"]["children"]["?"]["children"]["a=1"]["children"]["$"]["children"]["get"]
            ["leaf_info"]["uri"]
            .as_str()
            .unwrap(),
        "iam-res://iam-serv/p1?a=1"
    );
    assert_eq!(
        auth_res_serv::get_res_json()?["children"]["iam-res"]["children"]["iam-serv"]["children"]["p1"]["children"]["?"]["children"]["a=5"]["children"]["$"]["children"]["get"]
            ["leaf_info"]["uri"]
            .as_str()
            .unwrap(),
        "iam-res://iam-serv/p1?a=5"
    );

    cache_client.hdel(&config.cache_key_res_info, "iam-res://iam-serv/p1?a=1##get").await?;
    cache_client.hset(&config.cache_key_res_info, "iam-res://iam-serv/p1?a=6##get", "{\"accounts\":\"#acc6#\"}").await?;
    cache_client.hset(&config.cache_key_res_info, "iam-res://iam-serv/p1?a=7##get", "{\"accounts\":\"#acc7#\"}").await?;
    cache_client.set(&format!("{}:xx", config.cache_key_res_changed_info), "iam-res://iam-serv/p1?a=1##get").await?;
    cache_client.set(&format!("{}:yy", config.cache_key_res_changed_info), "iam-res://iam-serv/p1?a=6##get").await?;
    cache_client.set(&format!("{}:zz", config.cache_key_res_changed_info), "iam-res://iam-serv/p1?a=7##get").await?;

    sleep(Duration::from_secs(2)).await;

    println!("==========={}", auth_res_serv::get_res_json()?.to_string());

    assert!(auth_res_serv::get_res_json()?["children"]["iam-res"]["children"]["iam-serv"]["children"]["p1"]["children"]["?"]["children"].get("a=1").is_none());
    assert_eq!(
        auth_res_serv::get_res_json()?["children"]["iam-res"]["children"]["iam-serv"]["children"]["p1"]["children"]["?"]["children"]["a=6"]["children"]["$"]["children"]["get"]
            ["leaf_info"]["uri"]
            .as_str()
            .unwrap(),
        "iam-res://iam-serv/p1?a=6"
    );
    assert_eq!(
        auth_res_serv::get_res_json()?["children"]["iam-res"]["children"]["iam-serv"]["children"]["p1"]["children"]["?"]["children"]["a=7"]["children"]["$"]["children"]["get"]
            ["leaf_info"]["uri"]
            .as_str()
            .unwrap(),
        "iam-res://iam-serv/p1?a=7"
    );

    Ok(())
}
