use std::time::Duration;

use bios_basic::test::test_http_client::TestHttpClient;
use bios_spi_stats::dto::stats_query_dto::StatsQueryMetricsResp;
use tardis::basic::result::TardisResult;
use tardis::chrono::Utc;
use tardis::serde_json::{json, Value};
use tardis::tokio::time::sleep;
use tardis::web::web_resp::Void;

pub async fn test(client: &mut TestHttpClient) -> TardisResult<()> {
    let data = vec![
        // own_paths not illegal
        ("r001", "t1/a2", "zhejiang", "open", 1, vec!["t1", "t2"], "acc001", 10, 20, "2023-01-01T12:00:00.000Z"),
        ("r002", "t1/a1", "zhejiang", "open", 1, vec!["t1", "t2"], "acc001", 10, 20, "2023-01-01T12:00:00.000Z"),
        ("r003", "t1/a1", "hangzhou", "open", 1, vec!["t1", "t2"], "acc001", 10, 20, "2023-01-01T12:00:00.000Z"),
        ("r004", "t1/a1", "taizhou", "open", 1, vec!["t1", "t2"], "acc001", 10, 20, "2023-01-01T12:00:00.000Z"),
        ("r005", "t1/a1", "hangzhou", "open", 1, vec![], "acc001", 10, 20, "2023-01-01T12:00:00.000Z"),
        ("r006", "t1/a1", "hangzhou", "open", 1, vec!["t1"], "acc001", 10, 20, "2023-01-01T12:00:00.000Z"),
        ("r007", "t1/a1", "hangzhou", "open", 1, vec!["t2"], "acc001", 10, 20, "2023-01-01T12:00:00.000Z"),
        ("r008", "t1/a1", "hangzhou", "open", 2, vec!["t2"], "acc001", 10, 20, "2023-01-01T12:00:00.000Z"),
        ("r009", "t1/a1", "hangzhou", "open", 3, vec!["t2"], "acc001", 10, 20, "2023-01-01T12:00:00.000Z"),
        ("r010", "t1/a1", "hangzhou", "progress", 1, vec!["t1"], "acc001", 10, 20, "2023-01-02T12:00:00.000Z"),
        ("r011", "t1/a1", "hangzhou", "close", 1, vec!["t2"], "acc001", 10, 20, "2023-01-03T12:00:00.000Z"),
    ];

    let data = data
        .into_iter()
        .map(|(key, own_paths, source, status, priority, tag, creator, act_hours, plan_hours, ct)| {
            json!({
                "key":key,
                "own_paths":own_paths,
                "ct":ct,
                "data": json!({
                "source":source,
                "status": status,
                "priority":priority,
                "tag":tag,
                "creator":creator,
                "act_hours": act_hours,
                "plan_hours": plan_hours
                })
            })
        })
        .collect::<Vec<Value>>();

    let _: Void = client.put("/ci/record/fact/req/batch/load", &data).await;

    test_metric_query_check(client).await?;
    test_metric_query(client).await?;

    Ok(())
}

pub async fn test_metric_query_check(client: &mut TestHttpClient) -> TardisResult<()> {
    // fact not exist error
    assert_eq!(
        client
            .put_resp::<Value, StatsQueryMetricsResp>(
                "/ci/metric",
                &json!({
                    "from":"xx",
                    "select":[{"code":"act_hours","fun":"sum"},{"code":"plan_hours","fun":"sum"}],
                    "group":[{"code":"source"}],
                    "start_time":"2023-01-01T12:00:00.000Z",
                    "end_time":"2023-02-01T12:00:00.000Z"
                }),
            )
            .await
            .code,
        "404-spi-stats-metric-query"
    );

    // dimension in group not exist error
    assert_eq!(
        client
            .put_resp::<Value, StatsQueryMetricsResp>(
                "/ci/metric",
                &json!({
                    "from":"req",
                    "select":[{"code":"act_hours","fun":"sum"},{"code":"plan_hours","fun":"sum"}],
                    "group":[{"code":"xx"}],
                    "start_time":"2023-01-01T12:00:00.000Z",
                    "end_time":"2023-02-01T12:00:00.000Z"
                }),
            )
            .await
            .code,
        "404-spi-stats-metric-query"
    );

    // fact column in select not exist error
    assert_eq!(
        client
            .put_resp::<Value, StatsQueryMetricsResp>(
                "/ci/metric",
                &json!({
                    "from":"req",
                    "select":[{"code":"xxx","fun":"sum"},{"code":"plan_hours","fun":"sum"}],
                    "group":[{"code":"source"}],
                    "start_time":"2023-01-01T12:00:00.000Z",
                    "end_time":"2023-02-01T12:00:00.000Z"
                }),
            )
            .await
            .code,
        "404-spi-stats-metric-query"
    );

    // order info not exist in select error
    assert_eq!(
        client
            .put_resp::<Value, StatsQueryMetricsResp>(
                "/ci/metric",
                &json!({
                    "from":"req",
                    "select":[{"code":"act_hours","fun":"sum"},{"code":"plan_hours","fun":"sum"}],
                    "group":[{"code":"ct","time_window":"date"},{"code":"status"}],
                    "start_time":"2023-01-01T12:00:00.000Z",
                    "end_time":"2023-02-01T12:00:00.000Z",
                    "order": [{"code":"act_hours", "fun":"max","asc": false}]
                }),
            )
            .await
            .code,
        "404-spi-stats-metric-query"
    );

    // having info not exist in select error
    assert_eq!(
        client
            .put_resp::<Value, StatsQueryMetricsResp>(
                "/ci/metric",
                &json!({
                    "from":"req",
                    "select":[{"code":"act_hours","fun":"sum"},{"code":"plan_hours","fun":"sum"}],
                    "group":[{"code":"ct","time_window":"date"},{"code":"status"}],
                    "start_time":"2023-01-01T12:00:00.000Z",
                    "end_time":"2023-02-01T12:00:00.000Z",
                    "having": [{"code":"act_hours", "fun":"max","op": "=","value":10}]
                }),
            )
            .await
            .code,
        "404-spi-stats-metric-query"
    );

    // select function not exist error
    assert_eq!(
        client
            .put_resp::<Value, StatsQueryMetricsResp>(
                "/ci/metric",
                &json!({
                    "from":"req",
                    "select":[{"code":"act_hours","fun":"xxx"},{"code":"plan_hours","fun":"sum"}],
                    "group":[{"code":"source"}],
                    "start_time":"2023-01-01T12:00:00.000Z",
                    "end_time":"2023-02-01T12:00:00.000Z"
                }),
            )
            .await
            .code,
        "400"
    );

    // group function not exist error
    assert_eq!(
        client
            .put_resp::<Value, StatsQueryMetricsResp>(
                "/ci/metric",
                &json!({
                    "from":"req",
                    "select":[{"code":"act_hours","fun":"sum"},{"code":"plan_hours","fun":"sum"}],
                    "group":[{"code":"source","time_window":"xxx"}],
                    "start_time":"2023-01-01T12:00:00.000Z",
                    "end_time":"2023-02-01T12:00:00.000Z"
                }),
            )
            .await
            .code,
        "400"
    );

    // group function not legal (time window function only for date/datetime type) error
    assert_eq!(
        client
            .put_resp::<Value, StatsQueryMetricsResp>(
                "/ci/metric",
                &json!({
                    "from":"req",
                    "select":[{"code":"act_hours","fun":"sum"},{"code":"plan_hours","fun":"sum"}],
                    "group":[{"code":"source","time_window":"day"}],
                    "start_time":"2023-01-01T12:00:00.000Z",
                    "end_time":"2023-02-01T12:00:00.000Z"
                }),
            )
            .await
            .code,
        "404-spi-stats-metric-query"
    );
    Ok(())
}

pub async fn test_metric_query(client: &mut TestHttpClient) -> TardisResult<()> {
    // test without dimension
    let resp: StatsQueryMetricsResp = client
        .put(
            "/ci/metric",
            &json!({
                "from":"req",
                "select":[{"code":"act_hours","fun":"sum"},{"code":"plan_hours","fun":"sum"}],
                "group":[],
                "start_time":"2023-01-01T12:00:00.000Z",
                "end_time":"2023-02-01T12:00:00.000Z"
            }),
        )
        .await;
    assert_eq!(resp.from, "req");
    assert_eq!(resp.show_names.len(), 2);
    assert_eq!(resp.group.as_object().unwrap().len(), 2);
    assert_eq!(resp.group.as_object().unwrap()["act_hours__sum"], 100);
    assert_eq!(resp.group.as_object().unwrap()["plan_hours__sum"], 200);

    // test simple one dimension
    let resp: StatsQueryMetricsResp = client
        .put(
            "/ci/metric",
            &json!({
                "from":"req",
                "select":[{"code":"act_hours","fun":"sum"},{"code":"plan_hours","fun":"sum"}],
                "group":[{"code":"source"}],
                "start_time":"2023-01-01T12:00:00.000Z",
                "end_time":"2023-02-01T12:00:00.000Z"
            }),
        )
        .await;
    assert_eq!(resp.from, "req");
    assert_eq!(resp.show_names.len(), 3);
    assert_eq!(resp.group.as_object().unwrap().len(), 4);
    assert_eq!(resp.group.as_object().unwrap()[""]["act_hours__sum"], 100);
    assert_eq!(resp.group.as_object().unwrap()[""]["plan_hours__sum"], 200);
    assert_eq!(resp.group.as_object().unwrap()["hangzhou"]["act_hours__sum"], 80);
    assert_eq!(resp.group.as_object().unwrap()["hangzhou"]["plan_hours__sum"], 160);

    // test simple one dimension with key count
    let resp: StatsQueryMetricsResp = client
        .put(
            "/ci/metric",
            &json!({
                "from":"req",
                "select":[{"code":"key","fun":"count"},{"code":"plan_hours","fun":"sum"}],
                "group":[{"code":"source"}],
                "start_time":"2023-01-01T12:00:00.000Z",
                "end_time":"2023-02-01T12:00:00.000Z"
            }),
        )
        .await;
    assert_eq!(resp.from, "req");
    assert_eq!(resp.show_names.len(), 3);
    assert_eq!(resp.group.as_object().unwrap().len(), 4);
    assert_eq!(resp.group.as_object().unwrap()[""]["key__count"], 10);
    assert_eq!(resp.group.as_object().unwrap()[""]["plan_hours__sum"], 200);
    assert_eq!(resp.group.as_object().unwrap()["hangzhou"]["key__count"], 8);
    assert_eq!(resp.group.as_object().unwrap()["hangzhou"]["plan_hours__sum"], 160);

    // test simple two dimensions
    let resp: StatsQueryMetricsResp = client
        .put(
            "/ci/metric",
            &json!({
                "from":"req",
                "select":[{"code":"act_hours","fun":"sum"},{"code":"plan_hours","fun":"max"}],
                "group":[{"code":"source"},{"code":"status"}],
                "start_time":"2023-01-01T12:00:00.000Z",
                "end_time":"2023-02-01T12:00:00.000Z"
            }),
        )
        .await;
    assert_eq!(resp.from, "req");
    assert_eq!(resp.show_names.len(), 4);
    assert_eq!(resp.show_names["act_hours__sum"].as_str(), "实例工时");
    assert_eq!(resp.show_names["plan_hours__max"].as_str(), "计划工时");
    assert_eq!(resp.show_names["status__"].as_str(), "状态");
    assert_eq!(resp.show_names["source__"].as_str(), "来源");
    assert_eq!(resp.group.as_object().unwrap().len(), 4);
    assert_eq!(resp.group.as_object().unwrap()["hangzhou"][""]["act_hours__sum"], 80);
    assert_eq!(resp.group.as_object().unwrap()["hangzhou"][""]["plan_hours__max"], 20);
    assert_eq!(resp.group.as_object().unwrap()["hangzhou"]["close"]["act_hours__sum"], 10);
    assert_eq!(resp.group.as_object().unwrap()["hangzhou"]["close"]["plan_hours__max"], 20);

    // TODO test tree dimensions with multiple values
    // let resp: StatsQueryMetricsResp = client
    //     .put(
    //         "/ci/metric",
    //         &json!({
    //             "from":"req",
    //             "select":[{"code":"act_hours","fun":"avg"},{"code":"plan_hours","fun":"avg"}],
    //             "group":[{"code":"source"},{"code":"status"},{"code":"tag"}],
    //             "start_time":"2023-01-01T12:00:00.000Z",
    //             "end_time":"2023-02-01T12:00:00.000Z"
    //         }),
    //     )
    //     .await;

    // test two dimensions with time window
    let resp: StatsQueryMetricsResp = client
        .put(
            "/ci/metric",
            &json!({
                "from":"req",
                "select":[{"code":"act_hours","fun":"avg"},{"code":"plan_hours","fun":"avg"}],
                "group":[{"code":"ct","time_window":"day"},{"code":"status"}],
                "start_time":"2023-01-01T12:00:00.000Z",
                "end_time":"2023-02-01T12:00:00.000Z"
            }),
        )
        .await;
    assert_eq!(resp.from, "req");
    assert_eq!(resp.show_names.len(), 4);
    assert_eq!(resp.show_names["ct__day"].as_str(), "创建时间");
    assert_eq!(resp.group.as_object().unwrap().len(), 4);
    assert_eq!(resp.group.as_object().unwrap()[""][""]["act_hours__avg"], 10.0);
    assert_eq!(resp.group.as_object().unwrap()[""][""]["plan_hours__avg"], 20.0);
    assert_eq!(resp.group.as_object().unwrap()["1"][""]["act_hours__avg"], 10.0);
    assert_eq!(resp.group.as_object().unwrap()["1"][""]["plan_hours__avg"], 20.0);
    assert_eq!(resp.group.as_object().unwrap()["1"]["open"]["act_hours__avg"], 10.0);
    assert_eq!(resp.group.as_object().unwrap()["1"]["open"]["plan_hours__avg"], 20.0);

    // test two dimensions with limit
    let resp: StatsQueryMetricsResp = client
        .put(
            "/ci/metric",
            &json!({
                "from":"req",
                "select":[{"code":"act_hours","fun":"sum"},{"code":"plan_hours","fun":"sum"}],
                "group":[{"code":"ct","time_window":"date"},{"code":"status"}],
                "start_time":"2023-01-01T12:00:00.000Z",
                "end_time":"2023-02-01T12:00:00.000Z",
                "limit":2
            }),
        )
        .await;
    assert_eq!(resp.from, "req");
    assert_eq!(resp.show_names["ct__date"].as_str(), "创建时间");
    assert_eq!(resp.group.as_object().unwrap().len(), 1);
    assert_eq!(resp.group.as_object().unwrap()["2023-01-01"][""]["act_hours__sum"], 80);
    assert_eq!(resp.group.as_object().unwrap()["2023-01-01"]["open"]["act_hours__sum"], 80);

    // test two dimensions with order
    let resp: StatsQueryMetricsResp = client
        .put(
            "/ci/metric",
            &json!({
                "from":"req",
                "select":[{"code":"act_hours","fun":"sum"},{"code":"plan_hours","fun":"sum"}],
                "group":[{"code":"ct","time_window":"date"},{"code":"status"}],
                "start_time":"2023-01-01T12:00:00.000Z",
                "end_time":"2023-02-01T12:00:00.000Z",
                "order": [{"code":"act_hours","fun":"sum","asc": false}]
            }),
        )
        .await;
    assert_eq!(resp.from, "req");
    assert_eq!(resp.show_names.len(), 4);
    assert_eq!(resp.show_names["ct__date"].as_str(), "创建时间");
    assert_eq!(resp.group.as_object().unwrap().len(), 4);
    assert_eq!(resp.group.as_object().unwrap()[""][""]["act_hours__sum"], 100);
    assert_eq!(resp.group.as_object().unwrap()[""][""]["plan_hours__sum"], 200);
    assert_eq!(resp.group.as_object().unwrap()["2023-01-03"]["close"]["act_hours__sum"], 10);

    // test tree dimensions with having
    let resp: StatsQueryMetricsResp = client
        .put(
            "/ci/metric",
            &json!({
                "from":"req",
                "select":[{"code":"act_hours","fun":"sum"},{"code":"plan_hours","fun":"sum"}],
                "group":[{"code":"ct","time_window":"date"},{"code":"status"}],
                "start_time":"2023-01-01T12:00:00.000Z",
                "end_time":"2023-02-01T12:00:00.000Z",
                "having": [{"code":"act_hours","fun": "sum", "op":">", "value":30}]
            }),
        )
        .await;
    assert_eq!(resp.from, "req");
    assert_eq!(resp.group.as_object().unwrap().len(), 2);
    assert_eq!(resp.group.as_object().unwrap()[""][""]["act_hours__sum"], 100);
    assert_eq!(resp.group.as_object().unwrap()[""][""]["plan_hours__sum"], 200);
    assert_eq!(resp.group.as_object().unwrap()["2023-01-01"]["open"]["act_hours__sum"], 80);
    assert_eq!(resp.group.as_object().unwrap()["2023-01-01"]["open"]["plan_hours__sum"], 160);

    // test two dimensions with all
    let resp: StatsQueryMetricsResp = client
        .put(
            "/ci/metric",
            &json!({
                "from":"req",
                "select":[{"code":"act_hours","fun":"sum"},{"code":"plan_hours","fun":"sum"}],
                "group":[{"code":"ct","time_window":"date"},{"code":"status"}],
                "start_time":"2023-01-01T12:00:00.000Z",
                "end_time":"2023-02-01T12:00:00.000Z",
                "order": [{"code":"act_hours","fun":"sum","asc": false}],
                "having": [{"code":"act_hours","fun": "sum", "op":">", "value":30}],
                "limit":2
            }),
        )
        .await;
    assert_eq!(resp.from, "req");
    assert_eq!(resp.show_names["ct__date"].as_str(), "创建时间");
    assert_eq!(resp.group.as_object().unwrap().len(), 2);
    assert_eq!(resp.group.as_object().unwrap()[""][""]["act_hours__sum"], 100);
    assert_eq!(resp.group.as_object().unwrap()[""][""]["plan_hours__sum"], 200);
    assert_eq!(resp.group.as_object().unwrap()["2023-01-01"]["open"]["act_hours__sum"], 80);
    assert_eq!(resp.group.as_object().unwrap()["2023-01-01"]["open"]["plan_hours__sum"], 160);

    // test where
    let resp: StatsQueryMetricsResp = client
        .put(
            "/ci/metric",
            &json!({
                "from":"req",
                "select":[{"code":"act_hours","fun":"sum"},{"code":"plan_hours","fun":"sum"}],
                "group":[{"code":"ct","time_window":"date"},{"code":"status"}],
                "where":[
                    [{"code":"act_hours", "op":">", "value":10},{"code":"ct", "op":"!=", "value":1, "time_window":"day"}],
                    [{"code":"status", "op":"=", "value":"open"}]
                    ],
                "start_time":"2023-01-01T12:00:00.000Z",
                "end_time":"2023-02-01T12:00:00.000Z"
            }),
        )
        .await;
    assert_eq!(resp.from, "req");
    assert_eq!(resp.show_names["ct__date"].as_str(), "创建时间");
    assert_eq!(resp.group.as_object().unwrap().len(), 2);
    assert_eq!(resp.group.as_object().unwrap()[""][""]["act_hours__sum"], 80);
    assert_eq!(resp.group.as_object().unwrap()[""][""]["plan_hours__sum"], 160);
    assert_eq!(resp.group.as_object().unwrap()["2023-01-01"]["open"]["act_hours__sum"], 80);
    assert_eq!(resp.group.as_object().unwrap()["2023-01-01"]["open"]["plan_hours__sum"], 160);

    // test with delete record
    let resp: StatsQueryMetricsResp = client
        .put(
            "/ci/metric",
            &json!({
                "from":"req",
                "select":[{"code":"act_hours","fun":"sum"},{"code":"plan_hours","fun":"sum"}],
                "group":[{"code":"source"}],
                "start_time":"2023-01-01T12:00:00.000Z",
                "end_time": Utc::now().to_rfc3339()
            }),
        )
        .await;
    assert_eq!(resp.from, "req");
    assert_eq!(resp.group.as_object().unwrap().len(), 4);
    assert_eq!(resp.group.as_object().unwrap()["hangzhou"]["act_hours__sum"], 80);
    assert_eq!(resp.group.as_object().unwrap()["hangzhou"]["plan_hours__sum"], 160);

    assert_eq!(client.delete_resp("/ci/record/fact/req/r011").await.code, "200");
    sleep(Duration::from_millis(100)).await;
    let resp: StatsQueryMetricsResp = client
        .put(
            "/ci/metric",
            &json!({
                "from":"req",
                "select":[{"code":"act_hours","fun":"sum"},{"code":"plan_hours","fun":"sum"}],
                "group":[{"code":"source"}],
                "start_time":"2023-01-01T12:00:00.000Z",
                "end_time": Utc::now().to_rfc3339()
            }),
        )
        .await;
    assert_eq!(resp.from, "req");
    assert_eq!(resp.group.as_object().unwrap().len(), 4);
    assert_eq!(resp.group.as_object().unwrap()["hangzhou"]["act_hours__sum"], 70);
    assert_eq!(resp.group.as_object().unwrap()["hangzhou"]["plan_hours__sum"], 140);

    let resp: StatsQueryMetricsResp = client
        .put(
            "/ci/metric",
            &json!({
                "from":"req",
                "select":[{"code":"act_hours","fun":"sum"},{"code":"plan_hours","fun":"sum"}],
                "group":[{"code":"source"}],
                "start_time":"2023-01-01T12:00:00.000Z",
                "end_time":"2023-02-01T12:00:00.000Z"
            }),
        )
        .await;
    assert_eq!(resp.from, "req");
    assert_eq!(resp.group.as_object().unwrap().len(), 4);
    assert_eq!(resp.group.as_object().unwrap()["hangzhou"]["act_hours__sum"], 80);
    assert_eq!(resp.group.as_object().unwrap()["hangzhou"]["plan_hours__sum"], 160);

    Ok(())
}
