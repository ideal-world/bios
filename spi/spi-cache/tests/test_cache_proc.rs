use std::collections::HashMap;

use bios_basic::test::test_http_client::TestHttpClient;
use bios_spi_cache::dto::cache_proc_dto::{ExpReq, KIncrReq, KReq, KbReq, KbvReq, KfIncrReq, KfReq, KfvReq, KvReq, KvWithExReq};
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::web::web_resp::{TardisResp, Void};

pub async fn test(client: &mut TestHttpClient) -> TardisResult<()> {
    client.set_auth(&TardisContext {
        own_paths: "t1/a1".to_string(),
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
                key: "k".to_string(),
                value: "v".to_string(),
            },
        )
        .await;

    let _: Void = client
        .post(
            "/ci/proc/set_ex",
            &KvWithExReq {
                key: "k_ex".to_string(),
                value: "值_ex".to_string(),
                exp_sec: 1,
            },
        )
        .await;

    let result: Option<String> = client.put("/ci/proc/get", &KReq { key: "k_ex".to_string() }).await;
    assert_eq!(result, Some("值_ex".to_string()));

    let result: bool = client
        .put(
            "/ci/proc/set_nx",
            &KvReq {
                key: "k_ex".to_string(),
                value: "值_ex".to_string(),
            },
        )
        .await;
    assert!(!result);
    tardis::tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    let result: TardisResp<Option<String>> = client.put_resp("/ci/proc/get", &KReq { key: "k_ex".to_string() }).await;
    assert_eq!(result.data, None);

    let result: bool = client
        .put(
            "/ci/proc/set_nx",
            &KvReq {
                key: "k_ex".to_string(),
                value: "值_ex".to_string(),
            },
        )
        .await;
    assert!(result);

    let result: TardisResp<Option<String>> = client
        .put_resp(
            "/ci/proc/getset",
            &KvReq {
                key: "k_getset".to_string(),
                value: "v_getset".to_string(),
            },
        )
        .await;
    assert_eq!(result.data, None);
    let result: Option<String> = client
        .put(
            "/ci/proc/getset",
            &KvReq {
                key: "k_getset".to_string(),
                value: "v_getset2".to_string(),
            },
        )
        .await;
    assert_eq!(result, Some("v_getset".to_string()));

    let result: i64 = client
        .post(
            "/ci/proc/incr",
            &KIncrReq {
                key: "k_incr".to_string(),
                delta: 1,
            },
        )
        .await;
    assert_eq!(result, 1);
    let result: i64 = client
        .post(
            "/ci/proc/incr",
            &KIncrReq {
                key: "k_incr".to_string(),
                delta: 1,
            },
        )
        .await;
    assert_eq!(result, 2);
    let result: i64 = client
        .post(
            "/ci/proc/incr",
            &KIncrReq {
                key: "k_incr".to_string(),
                delta: -3,
            },
        )
        .await;
    assert_eq!(result, -1);

    let result: bool = client.put("/ci/proc/exists", &KReq { key: "k".to_string() }).await;
    assert!(result);
    let _: Void = client.put("/ci/proc/del", &KReq { key: "k".to_string() }).await;
    let result: bool = client.put("/ci/proc/exists", &KReq { key: "k".to_string() }).await;
    assert!(!result);

    let _: Void = client
        .post(
            "/ci/proc/expire",
            &ExpReq {
                key: "k_incr".to_string(),
                exp_sec: 1,
            },
        )
        .await;
    let result: u64 = client.put("/ci/proc/ttl", &KReq { key: "k_incr".to_string() }).await;
    assert!(result > 0);
    tardis::tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    let result: TardisResp<Option<String>> = client.put_resp("/ci/get", &KReq { key: "k_incr".to_string() }).await;
    assert_eq!(result.data, None);

    info!("【test_cache_list】");

    let _: Void = client
        .put(
            "/ci/proc/lpush",
            &KvReq {
                key: "k_list".to_string(),
                value: "v1".to_string(),
            },
        )
        .await;
    let _: Void = client
        .put(
            "/ci/proc/lpush",
            &KvReq {
                key: "k_list".to_string(),
                value: "v2".to_string(),
            },
        )
        .await;
    let result: Vec<String> = client.put("/ci/proc/lrangeall", &KReq { key: "k_list".to_string() }).await;
    assert_eq!(result, vec!["v2".to_string(), "v1".to_string()]);

    let result: u64 = client.put("/ci/proc/llen", &KReq { key: "k_list".to_string() }).await;
    assert_eq!(result, 2);

    info!("【test_cache_hash】");

    let _: Void = client
        .put(
            "/ci/proc/hset",
            &KfvReq {
                key: "k_set".to_string(),
                field: "f1".to_string(),
                value: "v1".to_string(),
            },
        )
        .await;
    let result: TardisResp<Option<String>> = client
        .put_resp(
            "/ci/proc/hget",
            &KfReq {
                key: "k_set".to_string(),
                field: "f2".to_string(),
            },
        )
        .await;
    assert_eq!(result.data, None);
    let _: Void = client
        .put(
            "/ci/proc/hset",
            &KfvReq {
                key: "k_set".to_string(),
                field: "f2".to_string(),
                value: "v2".to_string(),
            },
        )
        .await;
    let result: Option<String> = client
        .put(
            "/ci/proc/hget",
            &KfReq {
                key: "k_set".to_string(),
                field: "f2".to_string(),
            },
        )
        .await;
    assert_eq!(result, Some("v2".to_string()));

    let result: bool = client
        .put(
            "/ci/proc/hset_nx",
            &KfvReq {
                key: "k_set".to_string(),
                field: "f3".to_string(),
                value: "v3".to_string(),
            },
        )
        .await;
    assert!(result);
    let result: bool = client
        .put(
            "/ci/proc/hset_nx",
            &KfvReq {
                key: "k_set".to_string(),
                field: "f3".to_string(),
                value: "v3".to_string(),
            },
        )
        .await;
    assert!(!result);

    let _: Void = client
        .put(
            "/ci/proc/hdel",
            &KfReq {
                key: "k_set".to_string(),
                field: "f2".to_string(),
            },
        )
        .await;
    let result: bool = client
        .put(
            "/ci/proc/hexists",
            &KfReq {
                key: "k_set".to_string(),
                field: "f2".to_string(),
            },
        )
        .await;
    assert!(!result);

    let result: i64 = client
        .post(
            "/ci/proc/hincr",
            &KfIncrReq {
                key: "k_set".to_string(),
                field: "f4".to_string(),
                delta: 4,
            },
        )
        .await;
    assert_eq!(result, 4);
    let result: i64 = client
        .post(
            "/ci/proc/hincr",
            &KfIncrReq {
                key: "k_set".to_string(),
                field: "f4".to_string(),
                delta: 4,
            },
        )
        .await;
    assert_eq!(result, 8);

    let result: Vec<String> = client.put("/ci/proc/hkeys", &KReq { key: "k_set".to_string() }).await;
    assert_eq!(result, vec!["f1".to_string(), "f3".to_string(), "f4".to_string()]);

    let result: Vec<String> = client.put("/ci/proc/hvals", &KReq { key: "k_set".to_string() }).await;
    assert_eq!(result, vec!["v1".to_string(), "v3".to_string(), "8".to_string()]);

    let result: HashMap<String, String> = client.put("/ci/proc/hgetall", &KReq { key: "k_set".to_string() }).await;
    assert_eq!(result.len(), 3);

    let result: u64 = client.put("/ci/proc/hlen", &KReq { key: "k_set".to_string() }).await;
    assert_eq!(result, 3);

    info!("【test_cache_bitmap】");

    let result: bool = client
        .put(
            "/ci/proc/setbit",
            &KbvReq {
                key: "k_bitmap".to_string(),
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
                key: "k_bitmap".to_string(),
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
                key: "k_bitmap".to_string(),
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
                key: "k_bitmap".to_string(),
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
                key: "k_bitmap".to_string(),
                offset: 100,
            },
        )
        .await;
    assert!(result);
    let result: bool = client
        .put(
            "/ci/proc/getbit",
            &KbReq {
                key: "k_bitmap".to_string(),
                offset: 2,
            },
        )
        .await;
    assert!(!result);

    let result: u32 = client.put("/ci/proc/bitcount", &KReq { key: "k_bitmap".to_string() }).await;
    assert_eq!(result, 2);

    Ok(())
}
