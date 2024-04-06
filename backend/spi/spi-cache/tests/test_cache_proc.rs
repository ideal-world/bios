use std::collections::HashMap;

use bios_basic::test::test_http_client::TestHttpClient;
use bios_spi_cache::dto::cache_proc_dto::{ExpReq, KIncrReq, KReq, KbReq, KbvReq, KfIncrReq, KfReq, KfvReq, KvReq, KvWithExReq};
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::web::web_resp::{TardisResp, Void};

pub async fn test(client: &mut TestHttpClient) -> TardisResult<()> {
    client.set_auth(&TardisContext {
        own_paths: "t1/app001".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "app001".to_string(),
        ..Default::default()
    })?;

    info!("【test_cache_basic】");
    let _: Void = client
        .put(
            "/ci/proc/set",
            &KvReq {
                key: "k".into(),
                value: "v".to_string(),
            },
        )
        .await;

    let _: Void = client
        .post(
            "/ci/proc/set_ex",
            &KvWithExReq {
                key: "k_ex".into(),
                value: "值_ex".to_string(),
                exp_sec: 1,
            },
        )
        .await;

    let result: Option<String> = client.put("/ci/proc/get", &KReq { key: "k_ex".into() }).await;
    assert_eq!(result, Some("值_ex".to_string()));

    let result: bool = client
        .put(
            "/ci/proc/set_nx",
            &KvReq {
                key: "k_ex".into(),
                value: "值_ex".to_string(),
            },
        )
        .await;
    assert!(!result);
    tardis::tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    let result: TardisResp<Option<String>> = client.put_resp("/ci/proc/get", &KReq { key: "k_ex".into() }).await;
    assert_eq!(result.data, None);

    let result: bool = client
        .put(
            "/ci/proc/set_nx",
            &KvReq {
                key: "k_ex".into(),
                value: "值_ex".to_string(),
            },
        )
        .await;
    assert!(result);

    let result: TardisResp<Option<String>> = client
        .put_resp(
            "/ci/proc/getset",
            &KvReq {
                key: "k_getset".into(),
                value: "v_getset".to_string(),
            },
        )
        .await;
    assert_eq!(result.data, None);
    let result: Option<String> = client
        .put(
            "/ci/proc/getset",
            &KvReq {
                key: "k_getset".into(),
                value: "v_getset2".to_string(),
            },
        )
        .await;
    assert_eq!(result, Some("v_getset".to_string()));

    let result: i64 = client.post("/ci/proc/incr", &KIncrReq { key: "k_incr".into(), delta: 1 }).await;
    assert_eq!(result, 1);
    let result: i64 = client.post("/ci/proc/incr", &KIncrReq { key: "k_incr".into(), delta: 1 }).await;
    assert_eq!(result, 2);
    let result: i64 = client.post("/ci/proc/incr", &KIncrReq { key: "k_incr".into(), delta: -3 }).await;
    assert_eq!(result, -1);

    let result: bool = client.put("/ci/proc/exists", &KReq { key: "k".into() }).await;
    assert!(result);
    let _: Void = client.put("/ci/proc/del", &KReq { key: "k".into() }).await;
    let result: bool = client.put("/ci/proc/exists", &KReq { key: "k".into() }).await;
    assert!(!result);

    let _: Void = client.post("/ci/proc/expire", &ExpReq { key: "k_incr".into(), exp_sec: 1 }).await;
    let result: u64 = client.put("/ci/proc/ttl", &KReq { key: "k_incr".into() }).await;
    assert!(result > 0);
    tardis::tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    let result: TardisResp<Option<String>> = client.put_resp("/ci/get", &KReq { key: "k_incr".into() }).await;
    assert_eq!(result.data, None);

    info!("【test_cache_list】");

    let _: Void = client
        .put(
            "/ci/proc/lpush",
            &KvReq {
                key: "k_list".into(),
                value: "v1".to_string(),
            },
        )
        .await;
    let _: Void = client
        .put(
            "/ci/proc/lpush",
            &KvReq {
                key: "k_list".into(),
                value: "v2".to_string(),
            },
        )
        .await;
    let result: Vec<String> = client.put("/ci/proc/lrangeall", &KReq { key: "k_list".into() }).await;
    assert_eq!(result, vec!["v2".to_string(), "v1".to_string()]);

    let result: u64 = client.put("/ci/proc/llen", &KReq { key: "k_list".into() }).await;
    assert_eq!(result, 2);

    info!("【test_cache_hash】");

    let _: Void = client
        .put(
            "/ci/proc/hset",
            &KfvReq {
                key: "k_set".into(),
                field: "f1".into(),
                value: "v1".to_string(),
            },
        )
        .await;
    let result: TardisResp<Option<String>> = client
        .put_resp(
            "/ci/proc/hget",
            &KfReq {
                key: "k_set".into(),
                field: "f2".into(),
            },
        )
        .await;
    assert_eq!(result.data, None);
    let _: Void = client
        .put(
            "/ci/proc/hset",
            &KfvReq {
                key: "k_set".into(),
                field: "f2".into(),
                value: "v2".to_string(),
            },
        )
        .await;
    let result: Option<String> = client
        .put(
            "/ci/proc/hget",
            &KfReq {
                key: "k_set".into(),
                field: "f2".into(),
            },
        )
        .await;
    assert_eq!(result, Some("v2".to_string()));

    let result: bool = client
        .put(
            "/ci/proc/hset_nx",
            &KfvReq {
                key: "k_set".into(),
                field: "f3".into(),
                value: "v3".to_string(),
            },
        )
        .await;
    assert!(result);
    let result: bool = client
        .put(
            "/ci/proc/hset_nx",
            &KfvReq {
                key: "k_set".into(),
                field: "f3".into(),
                value: "v3".to_string(),
            },
        )
        .await;
    assert!(!result);

    let _: Void = client
        .put(
            "/ci/proc/hdel",
            &KfReq {
                key: "k_set".into(),
                field: "f2".into(),
            },
        )
        .await;
    let result: bool = client
        .put(
            "/ci/proc/hexists",
            &KfReq {
                key: "k_set".into(),
                field: "f2".into(),
            },
        )
        .await;
    assert!(!result);

    let result: i64 = client
        .post(
            "/ci/proc/hincr",
            &KfIncrReq {
                key: "k_set".into(),
                field: "f4".into(),
                delta: 4,
            },
        )
        .await;
    assert_eq!(result, 4);
    let result: i64 = client
        .post(
            "/ci/proc/hincr",
            &KfIncrReq {
                key: "k_set".into(),
                field: "f4".into(),
                delta: 4,
            },
        )
        .await;
    assert_eq!(result, 8);

    let result: Vec<String> = client.put("/ci/proc/hkeys", &KReq { key: "k_set".into() }).await;
    assert_eq!(result, vec!["f1".to_string(), "f3".to_string(), "f4".to_string()]);

    let result: Vec<String> = client.put("/ci/proc/hvals", &KReq { key: "k_set".into() }).await;
    assert_eq!(result, vec!["v1".to_string(), "v3".to_string(), "8".to_string()]);

    let result: HashMap<String, String> = client.put("/ci/proc/hgetall", &KReq { key: "k_set".into() }).await;
    assert_eq!(result.len(), 3);

    let result: u64 = client.put("/ci/proc/hlen", &KReq { key: "k_set".into() }).await;
    assert_eq!(result, 3);

    info!("【test_cache_bitmap】");

    let result: bool = client
        .put(
            "/ci/proc/setbit",
            &KbvReq {
                key: "k_bitmap".into(),
                offset: 1,
                value: true,
            },
        )
        .await;
    assert!(!result);
    let result: bool = client
        .put(
            "/ci/proc/setbit",
            &KbvReq {
                key: "k_bitmap".into(),
                offset: 100,
                value: true,
            },
        )
        .await;
    assert!(!result);
    let result: bool = client
        .put(
            "/ci/proc/setbit",
            &KbvReq {
                key: "k_bitmap".into(),
                offset: 100,
                value: false,
            },
        )
        .await;
    assert!(result);
    let result: bool = client
        .put(
            "/ci/proc/setbit",
            &KbvReq {
                key: "k_bitmap".into(),
                offset: 100,
                value: true,
            },
        )
        .await;
    assert!(!result);
    let result: bool = client
        .put(
            "/ci/proc/getbit",
            &KbReq {
                key: "k_bitmap".into(),
                offset: 100,
            },
        )
        .await;
    assert!(result);
    let result: bool = client
        .put(
            "/ci/proc/getbit",
            &KbReq {
                key: "k_bitmap".into(),
                offset: 2,
            },
        )
        .await;
    assert!(!result);

    let result: u32 = client.put("/ci/proc/bitcount", &KReq { key: "k_bitmap".into() }).await;
    assert_eq!(result, 2);

    Ok(())
}
