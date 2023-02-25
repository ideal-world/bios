use std::time::Duration;

use bios_basic::test::test_http_client::TestHttpClient;
use bios_spi_stats::dto::stats_record_dto::{StatsDimRecordAddReq, StatsFactRecordLoadReq, StatsFactRecordsLoadReq};
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
        client
            .put_resp::<StatsDimRecordAddReq, Void>(
                "/ci/record/dim/xx/xxx",
                &StatsDimRecordAddReq {
                    show_name: "错误记录".to_string(),
                    parent_key: None,
                },
            )
            .await
            .code,
        "409-spi-stats-dim_record-add"
    );

    // stable_ds = false error
    assert_eq!(
        client
            .put_resp::<StatsDimRecordAddReq, Void>(
                "/ci/record/dim/account/xxx",
                &StatsDimRecordAddReq {
                    show_name: "错误记录".to_string(),
                    parent_key: None,
                },
            )
            .await
            .code,
        "400-spi-stats-dim_record-add"
    );

    // hierarchy is empty error
    assert_eq!(
        client
            .put_resp::<StatsDimRecordAddReq, Void>(
                "/ci/record/dim/tag/t1",
                &StatsDimRecordAddReq {
                    show_name: "标签1".to_string(),
                    parent_key: Some("t2".to_string()),
                },
            )
            .await
            .code,
        "400-spi-stats-dim_record-add"
    );

    let _: Void = client
        .put(
            "/ci/record/dim/address/cn",
            &StatsDimRecordAddReq {
                show_name: "中国".to_string(),
                parent_key: None,
            },
        )
        .await;

    // parent dimension not exist error
    assert_eq!(
        client
            .put_resp::<StatsDimRecordAddReq, Void>(
                "/ci/record/dim/address/zhejiang",
                &StatsDimRecordAddReq {
                    show_name: "浙江".to_string(),
                    parent_key: Some("xxx".to_string()),
                },
            )
            .await
            .code,
        "404-spi-stats-dim_record-add"
    );

    let _: Void = client
        .put(
            "/ci/record/dim/address/zhejiang",
            &StatsDimRecordAddReq {
                show_name: "浙江".to_string(),
                parent_key: Some("cn".to_string()),
            },
        )
        .await;

    let _: Void = client
        .put(
            "/ci/record/dim/address/hangzhou",
            &StatsDimRecordAddReq {
                show_name: "杭州".to_string(),
                parent_key: Some("zhejiang".to_string()),
            },
        )
        .await;

    let _: Void = client
        .put(
            "/ci/record/dim/address/taizhou",
            &StatsDimRecordAddReq {
                show_name: "台州".to_string(),
                parent_key: Some("zhejiang".to_string()),
            },
        )
        .await;

    // dimension too deep error
    assert_eq!(
        client
            .put_resp::<StatsDimRecordAddReq, Void>(
                "/ci/record/dim/address/linhai",
                &StatsDimRecordAddReq {
                    show_name: "临海".to_string(),
                    parent_key: Some("taizhou".to_string()),
                },
            )
            .await
            .code,
        "409-spi-stats-dim_record-add"
    );

    let _: Void = client
        .put(
            "/ci/record/dim/tag/t1",
            &StatsDimRecordAddReq {
                show_name: "标签1".to_string(),
                parent_key: None,
            },
        )
        .await;

    // key exist error
    assert_eq!(
        client
            .put_resp::<StatsDimRecordAddReq, Void>(
                "/ci/record/dim/tag/t1",
                &StatsDimRecordAddReq {
                    show_name: "标签1".to_string(),
                    parent_key: None,
                },
            )
            .await
            .code,
        "409-spi-stats-dim_record-add"
    );

    let _: Void = client
        .put(
            "/ci/record/dim/tag/t2",
            &StatsDimRecordAddReq {
                show_name: "标签2".to_string(),
                parent_key: None,
            },
        )
        .await;

    let _: Void = client
        .put(
            "/ci/record/dim/req_status/open",
            &StatsDimRecordAddReq {
                show_name: "打开".to_string(),
                parent_key: None,
            },
        )
        .await;
    let _: Void = client
        .put(
            "/ci/record/dim/req_status/progress",
            &StatsDimRecordAddReq {
                show_name: "进行中".to_string(),
                parent_key: None,
            },
        )
        .await;
    let _: Void = client
        .put(
            "/ci/record/dim/req_status/close",
            &StatsDimRecordAddReq {
                show_name: "关闭".to_string(),
                parent_key: None,
            },
        )
        .await;

    let _: Void = client
        .put(
            "/ci/record/dim/req_priority/1",
            &StatsDimRecordAddReq {
                show_name: "紧急".to_string(),
                parent_key: None,
            },
        )
        .await;
    let _: Void = client
        .put(
            "/ci/record/dim/req_priority/2",
            &StatsDimRecordAddReq {
                show_name: "重要".to_string(),
                parent_key: None,
            },
        )
        .await;
    let _: Void = client
        .put(
            "/ci/record/dim/req_priority/3",
            &StatsDimRecordAddReq {
                show_name: "一般".to_string(),
                parent_key: None,
            },
        )
        .await;
    let _: Void = client
        .put(
            "/ci/record/dim/req_priority/4",
            &StatsDimRecordAddReq {
                show_name: "一般2_to_del".to_string(),
                parent_key: None,
            },
        )
        .await;
    sleep(Duration::from_millis(1000)).await;
    client.delete("/ci/record/dim/req_priority/4").await;

    let list: TardisPage<Value> = client.get("/ci/record/dim/req_priority?page_number=1&page_size=10").await;
    assert_eq!(list.total_size, 4);
    let list: TardisPage<Value> = client.get("/ci/record/dim/req_priority?show_name=一般&key=3&page_number=1&page_size=10").await;
    assert_eq!(list.total_size, 1);
    assert_eq!(list.records[0].get("key").unwrap().as_str().unwrap(), "3");
    assert_eq!(list.records[0].get("show_name").unwrap().as_str().unwrap(), "一般");
    assert!(!list.records[0].get("ct").unwrap().as_str().unwrap().is_empty());
    assert!(list.records[0].get("et").is_none());
    let list: TardisPage<Value> = client.get("/ci/record/dim/req_priority?&key=4&page_number=1&page_size=10").await;
    assert_eq!(list.total_size, 1);
    assert_eq!(list.records[0].get("key").unwrap().as_str().unwrap(), "4");
    assert_eq!(list.records[0].get("show_name").unwrap().as_str().unwrap(), "一般2_to_del");
    assert!(!list.records[0].get("ct").unwrap().as_str().unwrap().is_empty());
    assert!(!list.records[0].get("et").unwrap().as_str().unwrap().is_empty());
    let list: TardisPage<Value> = client.get("/ci/record/dim/address?key=taizhou&page_number=1&page_size=10").await;
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
                    "priority":"1",
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
                    "priority":"2",
                    "tag":["t1"],
                    "creator":"acc002",
                    "act_hours": 15,
                    "plan_hours": 10
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
                    "priority": "1",
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

    // dimension not exist error
    assert_eq!(
        client
            .put_resp::<Vec<StatsFactRecordsLoadReq>, Void>(
                "/ci/record/fact/req/batch/load",
                &vec![StatsFactRecordsLoadReq {
                    key: "rec3".to_string(),
                    own_paths: "t1/a1".to_string(),
                    ct: Utc::now(),
                    data: json!({
                        "source":"xxxx",
                        "status": "open",
                        "priority":"1",
                        "tag":["t1","t2"],
                        "creator":"acc001",
                        "act_hours": 40,
                        "plan_hours": 45
                    })
                }],
            )
            .await
            .code,
        "404-spi-stats-fact_record-load_set"
    );

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
                        "priority":"1",
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
                        "priority":"2",
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

    // ============================ clean ============================

    assert_eq!(client.delete_resp("/ci/record/fact/req/batch/clean").await.code, "200");

    Ok(())
}
