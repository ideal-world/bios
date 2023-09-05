use std::time::Duration;

use bios_basic::test::test_http_client::TestHttpClient;
use bios_spi_stats::dto::stats_record_dto::{StatsDimRecordDeleteReq, StatsFactRecordLoadReq, StatsFactRecordsLoadReq};
use tardis::basic::result::TardisResult;
use tardis::chrono::Utc;
use tardis::serde_json::{json, Value};
use tardis::tokio::time::sleep;
use tardis::web::web_resp::{TardisPage, Void};

pub async fn test(client: &mut TestHttpClient) -> TardisResult<()> {
    test_dim_record(client).await?;
    test_fact_record(client).await?;
    Ok(())
}

pub async fn test_dim_record(client: &mut TestHttpClient) -> TardisResult<()> {
    // dimension not exist error
    assert_eq!(
        client.put_resp::<Value, Void>("/ci/record/dim/xx", &json!({ "key":"xxx","show_name":"错误记录" })).await.code,
        "409-spi-stats-dim_record-add"
    );

    // stable_ds = false error
    assert_eq!(
        client.put_resp::<Value, Void>("/ci/record/dim/account", &json!({ "key":"xxx","show_name":"错误记录" })).await.code,
        "400-spi-stats-dim_record-add"
    );

    // hierarchy is empty error
    assert_eq!(
        client.put_resp::<Value, Void>("/ci/record/dim/tag", &json!({ "key":"t1","show_name":"标签1","parent_key":"t2" })).await.code,
        "400-spi-stats-dim_record-add"
    );

    let _: Void = client.put("/ci/record/dim/address", &json!({ "key":"cn","show_name":"中国" })).await;

    // parent dimension not exist error
    assert_eq!(
        client.put_resp::<Value, Void>("/ci/record/dim/address", &json!({ "key":"zhejiang","show_name":"浙江","parent_key":"xxx" })).await.code,
        "404-spi-stats-dim_record-add"
    );

    let _: Void = client.put("/ci/record/dim/address", &json!({ "key":"zhejiang","show_name":"浙江","parent_key":"cn" })).await;

    let _: Void = client.put("/ci/record/dim/address", &json!({ "key":"hangzhou","show_name":"杭州","parent_key":"zhejiang" })).await;

    let _: Void = client.put("/ci/record/dim/address", &json!({ "key":"taizhou","show_name":"台州","parent_key":"zhejiang" })).await;

    // dimension too deep error
    assert_eq!(
        client.put_resp::<Value, Void>("/ci/record/dim/address", &json!({ "key":"linhai","show_name":"临海","parent_key":"taizhou" })).await.code,
        "409-spi-stats-dim_record-add"
    );

    let _: Void = client.put("/ci/record/dim/tag", &json!({ "key":"t1","show_name":"标签1" })).await;

    // key exist error
    assert_eq!(
        client.put_resp::<Value, Void>("/ci/record/dim/tag", &json!({ "key":"t1","show_name":"标签1" })).await.code,
        "409-spi-stats-dim_record-add"
    );

    let _: Void = client.put("/ci/record/dim/tag", &json!({ "key":"t2","show_name":"标签2" })).await;

    let _: Void = client.put("/ci/record/dim/req_status", &json!({ "key":"open","show_name":"打开" })).await;
    let _: Void = client.put("/ci/record/dim/req_status", &json!({ "key":"progress","show_name":"进行中" })).await;
    let _: Void = client.put("/ci/record/dim/req_status", &json!({ "key":"close","show_name":"关闭" })).await;

    let _: Void = client.put("/ci/record/dim/req_priority", &json!({ "key":1,"show_name":"紧急" })).await;
    let _: Void = client.put("/ci/record/dim/req_priority", &json!({ "key":2,"show_name":"重要" })).await;
    let _: Void = client.put("/ci/record/dim/req_priority", &json!({ "key":3,"show_name":"一般" })).await;
    let _: Void = client.put("/ci/record/dim/req_priority", &json!({ "key":4,"show_name":"一般2_to_del" })).await;
    sleep(Duration::from_millis(1000)).await;
    let _: Void = client.put("/ci/record/dim/req_priority/remove", &json!({ "key":4})).await;

    let list: TardisPage<Value> = client.get("/ci/record/dim/req_priority?page_number=1&page_size=10").await;
    assert_eq!(list.total_size, 4);
    let list: TardisPage<Value> = client.get("/ci/record/dim/req_priority?show_name=一般&page_number=1&page_size=10").await;
    assert_eq!(list.total_size, 2);
    assert_eq!(list.records[0].get("key").unwrap().as_i64().unwrap(), 3);
    assert_eq!(list.records[0].get("show_name").unwrap().as_str().unwrap(), "一般");
    assert!(!list.records[0].get("ct").unwrap().as_str().unwrap().is_empty());
    assert!(list.records[0].get("et").unwrap().is_null());
    let list: TardisPage<Value> = client.get("/ci/record/dim/req_priority?&show_name=一般2_to_del&page_number=1&page_size=10").await;
    assert_eq!(list.total_size, 1);
    assert_eq!(list.records[0].get("key").unwrap().as_i64().unwrap(), 4);
    assert_eq!(list.records[0].get("show_name").unwrap().as_str().unwrap(), "一般2_to_del");
    assert!(!list.records[0].get("ct").unwrap().as_str().unwrap().is_empty());
    assert!(!list.records[0].get("et").unwrap().as_str().unwrap().is_empty());
    let list: TardisPage<Value> = client.get("/ci/record/dim/address?show_name=台州&page_number=1&page_size=10").await;
    assert_eq!(list.total_size, 1);
    assert_eq!(list.records[0].get("key").unwrap().as_str().unwrap(), "taizhou");
    assert_eq!(list.records[0].get("show_name").unwrap().as_str().unwrap(), "台州");
    assert_eq!(list.records[0].get("key0").unwrap().as_str().unwrap(), "cn");
    assert_eq!(list.records[0].get("key1").unwrap().as_str().unwrap(), "zhejiang");
    assert_eq!(list.records[0].get("key2").unwrap().as_str().unwrap(), "taizhou");
    assert_eq!(list.records[0].get("hierarchy").unwrap().as_i64().unwrap(), 2);
    assert!(!list.records[0].get("ct").unwrap().as_str().unwrap().is_empty());

    Ok(())
}

pub async fn test_fact_record(client: &mut TestHttpClient) -> TardisResult<()> {
    // ============================ load ============================
    // fact not exist error
    assert_eq!(
        client
            .put_resp::<StatsFactRecordLoadReq, Void>(
                "/ci/record/fact/xx/xxx",
                &StatsFactRecordLoadReq {
                    own_paths: "t1/a1".to_string(),
                    ct: Utc::now(),
                    data: json!({})
                },
            )
            .await
            .code,
        "409-spi-stats-fact_record-load"
    );

    // fact column not exist error
    assert_eq!(
        client
            .put_resp::<StatsFactRecordLoadReq, Void>(
                "/ci/record/fact/req/rec1",
                &StatsFactRecordLoadReq {
                    own_paths: "t1/a1".to_string(),
                    ct: Utc::now(),
                    data: json!({
                        "xx": 1
                    })
                },
            )
            .await
            .code,
        "404-spi-stats-fact_record-load"
    );

    // dimension not exist error
    assert_eq!(
        client
            .put_resp::<StatsFactRecordLoadReq, Void>(
                "/ci/record/fact/req/rec1",
                &StatsFactRecordLoadReq {
                    own_paths: "t1/a1".to_string(),
                    ct: Utc::now(),
                    data: json!({
                        "status": "openxxx"
                    })
                },
            )
            .await
            .code,
        "404-spi-stats-fact_record-load"
    );

    // parent record not exist error
    assert_eq!(
        client
            .put_resp::<StatsFactRecordLoadReq, Void>(
                "/ci/record/fact/req/rec1",
                &StatsFactRecordLoadReq {
                    own_paths: "t1/a1".to_string(),
                    ct: Utc::now(),
                    data: json!({
                        "status": "open"
                    })
                },
            )
            .await
            .code,
        "404-spi-stats-fact_record-load"
    );

    let _: Void = client
        .put(
            "/ci/record/fact/req/rec1",
            &StatsFactRecordLoadReq {
                own_paths: "t1/a1".to_string(),
                ct: Utc::now(),
                data: json!({
                    "source":"zhejiang",
                    "status": "open",
                    "priority":1,
                    "tag":["t1","t2"],
                    "creator":"acc001",
                    "act_hours": 40,
                    "plan_hours": 45
                }),
            },
        )
        .await;
    let _: Void = client
        .put(
            "/ci/record/fact/req/rec2",
            &StatsFactRecordLoadReq {
                own_paths: "t1/a1".to_string(),
                ct: Utc::now(),
                data: json!({
                    "source":"hangzhou",
                    "status": "open",
                    "priority":2,
                    "tag":["t1"],
                    "creator":"acc002",
                    "act_hours": 15,
                    "plan_hours": 10
                }),
            },
        )
        .await;
    // std::io::stdin().read_line(&mut Default::default()).unwrap();
    let latest_req_2: Value = client.get("/ci/record/fact/req/latest/rec2").await;
    assert!(!latest_req_2.is_null());
    let latest_reqs: Vec<Value> = client.get("/ci/record/fact/req/latest/?record_keys=rec1,rec2").await;
    assert!(latest_reqs.len() == 2);
    sleep(Duration::from_millis(1000)).await;
    let _: Void = client
        .put(
            "/ci/record/fact/req/rec2",
            &StatsFactRecordLoadReq {
                own_paths: "t1/a1".to_string(),
                ct: Utc::now(),
                data: json!({
                    "status": "progress",
                }),
            },
        )
        .await;
    sleep(Duration::from_millis(1000)).await;
    let _: Void = client
        .put(
            "/ci/record/fact/req/rec2",
            &StatsFactRecordLoadReq {
                own_paths: "t1/a1".to_string(),
                ct: Utc::now(),
                data: json!({
                    "priority": 1,
                }),
            },
        )
        .await;

    // ============================ load set ============================

    // fact not exist error
    assert_eq!(
        client
            .put_resp::<Vec<StatsFactRecordsLoadReq>, Void>(
                "/ci/record/fact/xx/batch/load",
                &vec![StatsFactRecordsLoadReq {
                    key: "rec3".to_string(),
                    own_paths: "t1/a1".to_string(),
                    ct: Utc::now(),
                    data: json!({})
                }],
            )
            .await
            .code,
        "409-spi-stats-fact_record-load_set"
    );

    // missing columns error
    assert_eq!(
        client
            .put_resp::<Vec<StatsFactRecordsLoadReq>, Void>(
                "/ci/record/fact/req/batch/load",
                &vec![StatsFactRecordsLoadReq {
                    key: "rec3".to_string(),
                    own_paths: "t1/a1".to_string(),
                    ct: Utc::now(),
                    data: json!({})
                }],
            )
            .await
            .code,
        "400-spi-stats-fact_record-load_set"
    );

    // // dimension not exist error
    // assert_eq!(
    //     client
    //         .put_resp::<Vec<StatsFactRecordsLoadReq>, Void>(
    //             "/ci/record/fact/req/batch/load",
    //             &vec![StatsFactRecordsLoadReq {
    //                 key: "rec3".to_string(),
    //                 own_paths: "t1/a1".to_string(),
    //                 ct: Utc::now(),
    //                 data: json!({
    //                     "source":"xxxx",
    //                     "status": "open",
    //                     "priority":1,
    //                     "tag":["t1","t2"],
    //                     "creator":"acc001",
    //                     "act_hours": 40,
    //                     "plan_hours": 45
    //                 })
    //             }],
    //         )
    //         .await
    //         .code,
    //     "404-spi-stats-fact_record-load_set"
    // );

    let _: Void = client
        .put(
            "/ci/record/fact/req/batch/load",
            &vec![
                StatsFactRecordsLoadReq {
                    key: "rec3".to_string(),
                    own_paths: "t1/a1".to_string(),
                    ct: Utc::now(),
                    data: json!({
                        "source":"zhejiang",
                        "status": "open",
                        "priority":1,
                        "tag":["t1","t2"],
                        "creator":"acc001",
                        "act_hours": 40,
                        "plan_hours": 45
                    }),
                },
                StatsFactRecordsLoadReq {
                    key: "rec4".to_string(),
                    own_paths: "t1/a2".to_string(),
                    ct: Utc::now(),
                    data: json!({
                        "source":"zhejiang",
                        "status": "open",
                        "priority":2,
                        "tag":["t1","t2"],
                        "creator":"acc001",
                        "act_hours": 40,
                        "plan_hours": 45
                    }),
                },
            ],
        )
        .await;

    // ============================ delete ============================

    assert_eq!(client.delete_resp("/ci/record/fact/req/rec1").await.code, "200");

    assert_eq!(
        client.put_resp::<Vec<String>, Void>("/ci/record/fact/req/batch/remove", &vec!["rec3".to_string()]).await.code,
        "200"
    );

    let _ = &&&assert_eq!(
        client
            .put_resp::<StatsDimRecordDeleteReq, Void>(
                "/ci/record/fact/req/dim/req_status/batch/remove",
                &StatsDimRecordDeleteReq {
                    key: tardis::serde_json::Value::String("open".to_string())
                }
            )
            .await
            .code,
        "200"
    );

    // ============================ clean ============================

    assert_eq!(client.delete_resp("/ci/record/fact/req/batch/clean").await.code, "200");

    Ok(())
}
