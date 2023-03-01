use bios_basic::test::test_http_client::TestHttpClient;
use bios_spi_stats::dto::stats_query_dto::StatsQueryMetricsResp;
use tardis::basic::result::TardisResult;
use tardis::serde_json::{json, Value};
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

    test_metric_query(client).await?;

    Ok(())
}

pub async fn test_metric_query(client: &mut TestHttpClient) -> TardisResult<()> {
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
    assert_eq!(resp.group.as_object().unwrap()["_"]["act_hours_sum"], 100);
    assert_eq!(resp.group.as_object().unwrap()["_"]["plan_hours_sum"], 200);
    assert_eq!(resp.group.as_object().unwrap()["hangzhou"]["act_hours_sum"], 80);
    assert_eq!(resp.group.as_object().unwrap()["hangzhou"]["plan_hours_sum"], 160);

    Ok(())
}
