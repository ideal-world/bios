use bios_basic::test::test_http_client::TestHttpClient;
use tardis::basic::result::TardisResult;
use tardis::serde_json::{json, Value};
use tardis::web::web_resp::{TardisPage, Void};

pub async fn test(client: &mut TestHttpClient) -> TardisResult<()> {
    test_dim_conf(client).await?;
    test_fact_conf(client).await?;
    Ok(())
}

pub async fn test_dim_conf(client: &mut TestHttpClient) -> TardisResult<()> {
    // key format error
    assert_eq!(
        client
            .put_resp::<Value, Void>(
                "/ci/conf/dim",
                &json!({
                    "key":"Account",
                    "show_name":"账号",
                    "stable_ds": false,
                    "data_type":"string",
                    "remark":"通用账号维度"
                }),
            )
            .await
            .code,
        "400"
    );

    let _: Void = client
        .put(
            "/ci/conf/dim",
            &json!({
                "key":"account",
                "show_name":"账号",
                "stable_ds": false,
                "data_type":"string",
                "remark":"通用账号维度"
            }),
        )
        .await;

    // key exist error
    assert_eq!(
        client
            .put_resp::<Value, Void>(
                "/ci/conf/dim",
                &json!({
                    "key":"account",
                    "show_name":"账号",
                    "stable_ds": false,
                    "data_type":"string",
                    "remark":"通用账号维度"
                }),
            )
            .await
            .code,
        "409-spi-stats-dim_conf-add"
    );

    let _: Void = client
        .put(
            "/ci/conf/dim",
            &json!({
                "key":"tag",
                "show_name":"标签",
                "stable_ds": true,
                "data_type":"string",
                "remark":"通用标签"
            }),
        )
        .await;
    let _: Void = client
        .put(
            "/ci/conf/dim",
            &json!({
                "key":"address",
                "show_name":"地址",
                "stable_ds": true,
                "data_type":"string",
                "hierarchy":["国家","省","市"],
                "remark":"通用地址维度"
            }),
        )
        .await;
    let _: Void = client
        .put(
            "/ci/conf/dim",
            &json!({
                "key":"req_priority",
                "show_name":"需求优先级",
                "stable_ds": true,
                "data_type":"int",
                "remark":"需求优先级"
            }),
        )
        .await;
    let _: Void = client
        .put(
            "/ci/conf/dim",
            &json!({
                "key":"req_status",
                "show_name":"状态",
                "stable_ds": false,
                "data_type":"bool",
                "remark":"状态",
                "dynamic_url": "dynamic/url"
            }),
        )
        .await;
    let _: Void = client
        .patch(
            "/ci/conf/dim/req_status",
            &json!({
                "key":"req_status",
                "show_name":"需求状态",
                "stable_ds": true,
                "data_type":"string",
                "remark":"需求状态",
                "dynamic_url": "dynamic/url/2"
            }),
        )
        .await;
    let _: Void = client
        .put(
            "/ci/conf/dim",
            &json!({
                "key":"to_be_del",
                "show_name":"删除测试",
                "stable_ds": false,
                "data_type":"bool"
            }),
        )
        .await;
    let _: Void = client
        .put(
            "/ci/conf/dim",
            &json!({
                "key":"to_be_del2",
                "show_name":"删除测试2",
                "stable_ds": false,
                "data_type":"bool"
            }),
        )
        .await;
    let list: TardisPage<Value> = client.get("/ci/conf/dim?page_number=1&page_size=10").await;
    assert_eq!(list.total_size, 7);
    let list: TardisPage<Value> = client.get("/ci/conf/dim?page_number=1&page_size=10&show_name=需求状态&key=req_status").await;
    assert_eq!(list.total_size, 1);
    assert_eq!(list.records[0]["key"].as_str().unwrap(), "req_status");
    assert_eq!(list.records[0]["show_name"].as_str().unwrap(), "需求状态");
    assert!(list.records[0]["stable_ds"].as_bool().unwrap());
    assert_eq!(list.records[0]["data_type"].as_str().unwrap(), "string");
    assert!(list.records[0]["hierarchy"].as_array().unwrap().is_empty());
    assert_eq!(list.records[0]["remark"].as_str().unwrap(), "需求状态");
    assert_eq!(list.records[0]["dynamic_url"].as_str().unwrap(), "dynamic/url/2");

    client.delete("/ci/conf/dim/to_be_del").await;
    let list: TardisPage<Value> = client.get("/ci/conf/dim?page_number=1&page_size=10").await;
    assert_eq!(list.total_size, 6);
    assert!(!list.records.iter().any(|d| d["online"].as_bool().unwrap()));

    // online
    let _: Void = client.put("/ci/conf/dim/account/online", &Void {}).await;
    let _: Void = client.put("/ci/conf/dim/tag/online", &Void {}).await;
    let _: Void = client.put("/ci/conf/dim/address/online", &Void {}).await;
    let _: Void = client.put("/ci/conf/dim/req_priority/online", &Void {}).await;
    let _: Void = client.put("/ci/conf/dim/req_status/online", &Void {}).await;
    let _: Void = client.put("/ci/conf/dim/to_be_del2/online", &Void {}).await;

    // can't modify after online error
    assert_eq!(
        client
            .patch_resp::<Value, Void>(
                "/ci/conf/dim/req_status",
                &json!({
                    "show_name":"需求状态",
                    "stable_ds": true,
                    "data_type":"string",
                    "remark":"需求状态"
                }),
            )
            .await
            .code,
        "409-spi-stats-dim_conf-modify"
    );

    client.delete("/ci/conf/dim/to_be_del2").await;
    let list: TardisPage<Value> = client.get("/ci/conf/dim?page_number=1&page_size=10").await;
    assert_eq!(list.total_size, 5);
    assert_eq!(list.records.iter().filter(|d| d.get("online").unwrap().as_bool().unwrap()).count(), 5);

    Ok(())
}

pub async fn test_fact_conf(client: &mut TestHttpClient) -> TardisResult<()> {
    // key format error
    assert_eq!(
        client
            .put_resp::<Value, Void>(
                "/ci/conf/fact",
                &json!({
                    "key":"Kb_doc",
                    "show_name":"知识库文档",
                    "query_limit": 1000,
                    "remark":"知识库文档"
                }),
            )
            .await
            .code,
        "400"
    );

    let _: Void = client
        .put(
            "/ci/conf/fact",
            &json!({
                "key":"kb_doc",
                "show_name":"知识库文档",
                "query_limit": 1000,
                "remark":"知识库文档",
                "is_online": false,
                "redirect_path": "path/to/some/where"
            }),
        )
        .await;

    // key exist error
    assert_eq!(
        client
            .put_resp::<Value, Void>(
                "/ci/conf/fact",
                &json!({
                    "key":"kb_doc",
                    "show_name":"知识库文档",
                    "query_limit": 1000,
                    "remark":"知识库文档"
                }),
            )
            .await
            .code,
        "409-spi-stats-fact_conf-add"
    );

    let _: Void = client
        .put(
            "/ci/conf/fact",
            &json!({
                "key":"req",
                "show_name":"需求1",
                "query_limit": 1000,
                "remark":"需求1"
            }),
        )
        .await;
    let _: Void = client
        .patch(
            "/ci/conf/fact/req",
            &json!({
                "show_name":"需求",
                "query_limit": 2000,
                "remark":"需求说明",
                "is_online": true,
                "redirect_path": "path/to/some/where"
            }),
        )
        .await;
    let _: Void = client
        .put(
            "/ci/conf/fact",
            &json!({
                "key":"to_be_del",
                "show_name":"删除测试",
                "query_limit": 1000
            }),
        )
        .await;
    let _: Void = client
        .put(
            "/ci/conf/fact",
            &json!({
                "key":"to_be_del2",
                "show_name":"删除测试2",
                "query_limit": 1000
            }),
        )
        .await;
    let list: TardisPage<Value> = client.get("/ci/conf/fact?page_number=1&page_size=10").await;
    assert_eq!(list.total_size, 4);
    let list: TardisPage<Value> = client.get("/ci/conf/fact?page_number=1&page_size=10&is_online=true").await;
    assert_eq!(list.total_size, 1);
    let list: TardisPage<Value> = client.get("/ci/conf/fact?page_number=1&page_size=10&show_name=需求&key=req").await;
    assert_eq!(list.total_size, 1);
    assert_eq!(list.records[0]["key"].as_str().unwrap(), "req");
    assert_eq!(list.records[0]["show_name"].as_str().unwrap(), "需求");
    assert!(list.records[0]["is_online"].as_bool().unwrap());
    assert_eq!(list.records[0]["redirect_path"].as_str().unwrap(), "path/to/some/where");
    assert_eq!(list.records[0]["query_limit"].as_i64().unwrap(), 2000);
    assert_eq!(list.records[0]["remark"].as_str().unwrap(), "需求说明");

    client.delete("/ci/conf/fact/to_be_del").await;
    let list: TardisPage<Value> = client.get("/ci/conf/fact?page_number=1&page_size=10").await;
    assert_eq!(list.total_size, 3);
    assert!(!list.records.iter().any(|d| d["online"].as_bool().unwrap()));

    // fact column not exist error
    assert_eq!(
        client.put_resp::<Void, Void>("/ci/conf/fact/kb_doc/online", &Void {}).await.code,
        "404-spi-stats-fact_col_conf-create_inst"
    );

    test_fact_col_conf(client).await?;

    // online
    assert_eq!(
        client.put_resp::<Void, Void>("/ci/conf/fact/kb_doc/online", &Void {}).await.code,
        "404-spi-stats-fact_col_conf-create_inst"
    );
    let _: Void = client.put("/ci/conf/fact/req/online", &Void {}).await;

    // can't modify fact after online error
    assert_eq!(
        client
            .patch_resp::<Value, Void>(
                "/ci/conf/fact/req",
                &json!({
                    "show_name":"需求",
                    "query_limit": 2000,
                    "remark": "需求说明"
                }),
            )
            .await
            .code,
        "409-spi-stats-fact_conf-modify"
    );

    // can't modify fact column after online error
    assert_eq!(
        client
            .patch_resp::<Value, Void>(
                "/ci/conf/fact/req/col/source",
                &json!({
                    "show_name":"来源",
                    "remark": "需求来源说明",
                    "kind": "dimension",
                    "dim_rel_conf_dim_key": "address",
                    "dim_multi_values": false
                }),
            )
            .await
            .code,
        "409-spi-stats-fact_col_conf-modify"
    );
    // can't delete fact column after online error
    assert_eq!(client.delete_resp("/ci/conf/fact/req/col/source").await.code, "409-spi-stats-fact_col_conf-delete");

    client.delete("/ci/conf/fact/to_be_del2").await;
    let list: TardisPage<Value> = client.get("/ci/conf/fact?page_number=1&page_size=10").await;
    assert_eq!(list.total_size, 2);
    assert_eq!(list.records.iter().filter(|d| d["online"].as_bool().unwrap()).count(), 1);
    Ok(())
}

pub async fn test_fact_col_conf(client: &mut TestHttpClient) -> TardisResult<()> {
    // key format error
    assert_eq!(
        client
            .put_resp::<Value, Void>(
                "/ci/conf/fact/req/col",
                &json!({
                    "key":"Status",
                    "show_name":"状态",
                    "remark": "状态说明",
                    "kind": "dimension",
                    "dim_rel_conf_dim_key": "ssss",
                    "dim_multi_values": false
                }),
            )
            .await
            .code,
        "400"
    );

    // dimension not online/exist error
    assert_eq!(
        client
            .put_resp::<Value, Void>(
                "/ci/conf/fact/req/col",
                &json!({
                    "key":"status",
                    "show_name":"状态",
                    "remark": "状态说明",
                    "kind": "dimension",
                    "dim_rel_conf_dim_key": "ssss",
                    "dim_multi_values": false
                }),
            )
            .await
            .code,
        "409-spi-stats-fact_col_conf-add"
    );

    let _: Void = client
        .put(
            "/ci/conf/fact/req/col",
            &json!({
                "key":"status",
                "show_name":"状态",
                "remark": "状态说明",
                "kind": "dimension",
                "dim_rel_conf_dim_key": "req_status",
                "dim_multi_values": false
            }),
        )
        .await;

    // key exist error
    assert_eq!(
        client
            .put_resp::<Value, Void>(
                "/ci/conf/fact/req/col",
                &json!({
                    "key":"status",
                    "show_name":"状态",
                    "remark": "状态说明",
                    "kind": "dimension",
                    "dim_rel_conf_dim_key": "req_status",
                    "dim_multi_values": false
                }),
            )
            .await
            .code,
        "409-spi-stats-fact_col_conf-add"
    );

    let _: Void = client
        .put(
            "/ci/conf/fact/req/col",
            &json!({
                "key":"priority",
                "show_name":"优先级",
                "remark": "优先级说明",
                "kind": "dimension",
                "dim_rel_conf_dim_key": "req_priority",
                "dim_multi_values": false
            }),
        )
        .await;
    let _: Void = client
        .put(
            "/ci/conf/fact/req/col",
            &json!({
                "key":"tag",
                "show_name":"标签",
                "remark": "标签说明",
                "kind": "dimension",
                "dim_rel_conf_dim_key": "tag",
                "dim_multi_values": true
            }),
        )
        .await;
    let _: Void = client
        .put(
            "/ci/conf/fact/req/col",
            &json!({
                "key":"creator",
                "show_name":"创建人",
                "remark": "创建人说明",
                "kind": "dimension",
                "dim_rel_conf_dim_key": "account",
                "dim_multi_values": false
            }),
        )
        .await;
    let _: Void = client
        .put(
            "/ci/conf/fact/req/col",
            &json!({
                "key":"source",
                "show_name":"来源_to_be_modify",
                "remark": "需求来源说明1",
                "kind": "dimension",
                "dim_rel_conf_dim_key": "req_status",
                "dim_multi_values": true
            }),
        )
        .await;
    let _: Void = client
        .patch(
            "/ci/conf/fact/req/col/source",
            &json!({
                "show_name":"来源",
                "remark": "需求来源说明",
                "kind": "dimension",
                "dim_rel_conf_dim_key": "address",
                "dim_multi_values": false
            }),
        )
        .await;
    let _: Void = client
        .put(
            "/ci/conf/fact/req/col",
            &json!({
                "key":"act_hours",
                "show_name":"实例工时",
                "kind": "measure",
                "mes_data_type": "int",
                "mes_frequency": "RT",
                "mes_act_by_dim_conf_keys": ["account"]
            }),
        )
        .await;
    let _: Void = client
        .put(
            "/ci/conf/fact/req/col",
            &json!({
                "key":"plan_hours",
                "show_name":"计划工时_to_be_modify",
                "kind": "measure",
                "mes_unit": "hour",
                "mes_data_type": "bool",
                "mes_frequency": "RT"
            }),
        )
        .await;
    let _: Void = client
        .patch(
            "/ci/conf/fact/req/col/plan_hours",
            &json!({
                "show_name":"计划工时",
                "remark":"计划工时说明",
                "kind": "measure",
                "mes_data_type": "int",
                "mes_frequency": "1H",
                "mes_act_by_dim_conf_keys": ["account"]
            }),
        )
        .await;
    let _: Void = client
        .put(
            "/ci/conf/fact/req/col",
            &json!({
                "key":"to_be_del",
                "show_name":"删除测试",
                "kind": "measure",
                "mes_data_type": "int",
                "mes_frequency": "RT",
                "mes_act_by_dim_conf_keys": ["account"]
            }),
        )
        .await;
    // cannot delete this dim because it is used by other fact
    let resp = client.delete_resp("/ci/conf/dim/address").await;
    assert!(resp.code.contains("409"));
    let list: TardisPage<Value> = client.get("/ci/conf/fact/req/col?page_number=1&page_size=10").await;
    assert_eq!(list.total_size, 8);
    // agg query with dim
    let list: TardisPage<Value> = client.get("/ci/conf/dim/address/col?page_number=1&page_size=10").await;
    assert_eq!(list.total_size, 1);
    assert_eq!(list.records[0]["key"].as_str().unwrap(), "source");
    assert_eq!(list.records[0]["show_name"].as_str().unwrap(), "来源");
    let list: TardisPage<Value> = client.get("/ci/conf/fact/req/col?page_number=1&page_size=10&show_name=工时").await;
    assert_eq!(list.total_size, 2);
    let list: TardisPage<Value> = client.get("/ci/conf/fact/req/col?page_number=1&page_size=10&key=source").await;
    assert_eq!(list.total_size, 1);
    assert_eq!(list.records[0]["key"].as_str().unwrap(), "source");
    assert_eq!(list.records[0]["show_name"].as_str().unwrap(), "来源");
    assert_eq!(list.records[0]["remark"].as_str().unwrap(), "需求来源说明");
    assert_eq!(list.records[0]["kind"].as_str().unwrap(), "dimension");
    assert_eq!(list.records[0]["dim_rel_conf_dim_key"].as_str().unwrap(), "address");
    assert!(!list.records[0]["dim_multi_values"].as_bool().unwrap());
    assert!(list.records[0]["mes_data_type"].is_null());
    assert!(list.records[0]["mes_frequency"].is_null());
    assert!(list.records[0]["mes_act_by_dim_conf_keys"].is_null());
    assert!(list.records[0]["rel_conf_fact_and_col_key"].is_null());
    let list: TardisPage<Value> = client.get("/ci/conf/fact/req/col?page_number=1&page_size=10&show_name=工时&key=plan_hours").await;
    assert_eq!(list.total_size, 1);
    assert_eq!(list.records[0]["key"].as_str().unwrap(), "plan_hours");
    assert_eq!(list.records[0]["show_name"].as_str().unwrap(), "计划工时");
    assert_eq!(list.records[0]["remark"].as_str().unwrap(), "计划工时说明");
    assert_eq!(list.records[0]["kind"].as_str().unwrap(), "measure");
    assert!(list.records[0]["dim_rel_conf_dim_key"].is_null());
    assert!(list.records[0]["dim_multi_values"].is_null());
    assert_eq!(list.records[0]["mes_data_type"].as_str().unwrap(), "int");
    assert_eq!(list.records[0]["mes_frequency"].as_str().unwrap(), "1H");
    assert_eq!(list.records[0]["mes_unit"].as_str().unwrap(), "hour");
    assert_eq!(list.records[0]["mes_act_by_dim_conf_keys"].as_array().unwrap().len(), 1);
    assert!(list.records[0]["rel_conf_fact_and_col_key"].is_null());

    client.delete("/ci/conf/fact/req/col/to_be_del").await;
    let list: TardisPage<Value> = client.get("/ci/conf/fact/req/col?page_number=1&page_size=10").await;
    assert_eq!(list.total_size, 7);

    /* enable to test delete fact
    // delete by kind
    client.delete("/ci/conf/fact/req/kind/dimension").await;
    let list: TardisPage<Value> = client.get("/ci/conf/fact/req/col?page_number=1&page_size=10").await;
    assert_eq!(list.total_size, 2);

    // delete fact will delete related measure and dimension
    client.delete("/ci/conf/fact/req").await;
    let list: TardisPage<Value> = client.get("/ci/conf/fact/req/col?page_number=1&page_size=10").await;
    assert_eq!(list.total_size, 0);
    */

    Ok(())
}
