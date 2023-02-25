use bios_basic::test::test_http_client::TestHttpClient;
use bios_spi_stats::dto::stats_conf_dto::{
    StatsConfDimAddReq, StatsConfDimInfoResp, StatsConfDimModifyReq, StatsConfFactAddReq, StatsConfFactColAddReq, StatsConfFactColInfoResp, StatsConfFactColModifyReq,
    StatsConfFactInfoResp, StatsConfFactModifyReq,
};
use bios_spi_stats::stats_enumeration::{StatsDataTypeKind, StatsFactColKind};
use tardis::basic::result::TardisResult;
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
            .put_resp::<StatsConfDimAddReq, Void>(
                "/ci/conf/dim",
                &StatsConfDimAddReq {
                    key: "Account".to_string(),
                    show_name: "账号".to_string(),
                    stable_ds: false,
                    data_type: StatsDataTypeKind::String,
                    hierarchy: None,
                    remark: Some("通用账号维度".to_string()),
                },
            )
            .await
            .code,
        "400"
    );

    let _: Void = client
        .put(
            "/ci/conf/dim",
            &StatsConfDimAddReq {
                key: "account".to_string(),
                show_name: "账号".to_string(),
                stable_ds: false,
                data_type: StatsDataTypeKind::String,
                hierarchy: None,
                remark: Some("通用账号维度".to_string()),
            },
        )
        .await;

    // key exist error
    assert_eq!(
        client
            .put_resp::<StatsConfDimAddReq, Void>(
                "/ci/conf/dim",
                &StatsConfDimAddReq {
                    key: "account".to_string(),
                    show_name: "账号".to_string(),
                    stable_ds: false,
                    data_type: StatsDataTypeKind::String,
                    hierarchy: None,
                    remark: Some("通用账号维度".to_string()),
                },
            )
            .await
            .code,
        "409-spi-stats-dim_conf-add"
    );

    let _: Void = client
        .put(
            "/ci/conf/dim",
            &StatsConfDimAddReq {
                key: "tag".to_string(),
                show_name: "标签".to_string(),
                stable_ds: true,
                data_type: StatsDataTypeKind::String,
                hierarchy: None,
                remark: Some("通用标签".to_string()),
            },
        )
        .await;
    let _: Void = client
        .put(
            "/ci/conf/dim",
            &StatsConfDimAddReq {
                key: "address".to_string(),
                show_name: "地址".to_string(),
                stable_ds: true,
                data_type: StatsDataTypeKind::String,
                hierarchy: Some(vec!["国家".to_string(), "省".to_string(), "市".to_string()]),
                remark: Some("通用地址维度".to_string()),
            },
        )
        .await;
    let _: Void = client
        .put(
            "/ci/conf/dim",
            &StatsConfDimAddReq {
                key: "req_priority".to_string(),
                show_name: "需求优先级".to_string(),
                stable_ds: true,
                data_type: StatsDataTypeKind::String,
                hierarchy: None,
                remark: Some("需求优先级".to_string()),
            },
        )
        .await;
    let _: Void = client
        .put(
            "/ci/conf/dim",
            &StatsConfDimAddReq {
                key: "req_status".to_string(),
                show_name: "状态".to_string(),
                stable_ds: false,
                data_type: StatsDataTypeKind::Boolean,
                hierarchy: None,
                remark: Some("状态".to_string()),
            },
        )
        .await;
    let _: Void = client
        .patch(
            "/ci/conf/dim/req_status",
            &StatsConfDimModifyReq {
                show_name: Some("需求状态".to_string()),
                stable_ds: Some(true),
                data_type: Some(StatsDataTypeKind::String),
                hierarchy: None,
                remark: Some("需求状态".to_string()),
            },
        )
        .await;
    let _: Void = client
        .put(
            "/ci/conf/dim",
            &StatsConfDimAddReq {
                key: "to_be_del".to_string(),
                show_name: "删除测试".to_string(),
                stable_ds: false,
                data_type: StatsDataTypeKind::Boolean,
                hierarchy: None,
                remark: None,
            },
        )
        .await;
    let _: Void = client
        .put(
            "/ci/conf/dim",
            &StatsConfDimAddReq {
                key: "to_be_del2".to_string(),
                show_name: "删除测试2".to_string(),
                stable_ds: false,
                data_type: StatsDataTypeKind::Boolean,
                hierarchy: None,
                remark: None,
            },
        )
        .await;
    let list: TardisPage<StatsConfDimInfoResp> = client.get("/ci/conf/dim?page_number=1&page_size=10").await;
    assert_eq!(list.total_size, 7);
    let list: TardisPage<StatsConfDimInfoResp> = client.get("/ci/conf/dim?page_number=1&page_size=10&show_name=需求状态&key=req_status").await;
    assert_eq!(list.total_size, 1);
    assert_eq!(list.records[0].key, "req_status");
    assert_eq!(list.records[0].show_name, "需求状态");
    assert!(list.records[0].stable_ds);
    assert_eq!(list.records[0].data_type, StatsDataTypeKind::String);
    assert!(list.records[0].hierarchy.is_empty());
    assert_eq!(list.records[0].remark, Some("需求状态".to_string()));

    client.delete("/ci/conf/dim/to_be_del").await;
    let list: TardisPage<StatsConfDimInfoResp> = client.get("/ci/conf/dim?page_number=1&page_size=10").await;
    assert_eq!(list.total_size, 6);
    assert!(!list.records.iter().any(|d| d.online));

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
            .patch_resp::<StatsConfDimModifyReq, Void>(
                "/ci/conf/dim/req_status",
                &StatsConfDimModifyReq {
                    show_name: Some("需求状态".to_string()),
                    stable_ds: Some(true),
                    data_type: Some(StatsDataTypeKind::String),
                    hierarchy: None,
                    remark: Some("需求状态".to_string()),
                },
            )
            .await
            .code,
        "409-spi-stats-dim_conf-modify"
    );

    client.delete("/ci/conf/dim/to_be_del2").await;
    let list: TardisPage<StatsConfDimInfoResp> = client.get("/ci/conf/dim?page_number=1&page_size=10").await;
    assert_eq!(list.total_size, 5);
    assert_eq!(list.records.iter().filter(|d| d.online).count(), 5);

    Ok(())
}

pub async fn test_fact_conf(client: &mut TestHttpClient) -> TardisResult<()> {
    // key format error
    assert_eq!(
        client
            .put_resp::<StatsConfFactAddReq, Void>(
                "/ci/conf/fact",
                &StatsConfFactAddReq {
                    key: "Kb_doc".to_string(),
                    show_name: "知识库文档".to_string(),
                    query_limit: 1000,
                    remark: Some("知识库文档".to_string()),
                },
            )
            .await
            .code,
        "400"
    );

    let _: Void = client
        .put(
            "/ci/conf/fact",
            &StatsConfFactAddReq {
                key: "kb_doc".to_string(),
                show_name: "知识库文档".to_string(),
                query_limit: 1000,
                remark: Some("知识库文档".to_string()),
            },
        )
        .await;

    // key exist error
    assert_eq!(
        client
            .put_resp::<StatsConfFactAddReq, Void>(
                "/ci/conf/fact",
                &StatsConfFactAddReq {
                    key: "kb_doc".to_string(),
                    show_name: "知识库文档".to_string(),
                    query_limit: 1000,
                    remark: Some("知识库文档".to_string()),
                },
            )
            .await
            .code,
        "409-spi-stats-fact_conf-add"
    );

    let _: Void = client
        .put(
            "/ci/conf/fact",
            &StatsConfFactAddReq {
                key: "req".to_string(),
                show_name: "需求1".to_string(),
                query_limit: 1000,
                remark: Some("需求1".to_string()),
            },
        )
        .await;
    let _: Void = client
        .patch(
            "/ci/conf/fact/req",
            &StatsConfFactModifyReq {
                show_name: Some("需求".to_string()),
                query_limit: Some(2000),
                remark: Some("需求说明".to_string()),
            },
        )
        .await;
    let _: Void = client
        .put(
            "/ci/conf/fact",
            &StatsConfFactAddReq {
                key: "to_be_del".to_string(),
                show_name: "删除测试".to_string(),
                query_limit: 1000,
                remark: None,
            },
        )
        .await;
    let _: Void = client
        .put(
            "/ci/conf/fact",
            &StatsConfFactAddReq {
                key: "to_be_del2".to_string(),
                show_name: "删除测试2".to_string(),
                query_limit: 1000,
                remark: None,
            },
        )
        .await;
    let list: TardisPage<StatsConfFactInfoResp> = client.get("/ci/conf/fact?page_number=1&page_size=10").await;
    assert_eq!(list.total_size, 4);
    let list: TardisPage<StatsConfFactInfoResp> = client.get("/ci/conf/fact?page_number=1&page_size=10&show_name=需求&key=req").await;
    assert_eq!(list.total_size, 1);
    assert_eq!(list.records[0].key, "req");
    assert_eq!(list.records[0].show_name, "需求");
    assert_eq!(list.records[0].query_limit, 2000);
    assert_eq!(list.records[0].remark, Some("需求说明".to_string()));

    client.delete("/ci/conf/fact/to_be_del").await;
    let list: TardisPage<StatsConfFactInfoResp> = client.get("/ci/conf/fact?page_number=1&page_size=10").await;
    assert_eq!(list.total_size, 3);
    assert!(!list.records.iter().any(|d| d.online));

    // fact column not exist error
    assert_eq!(
        client.put_resp::<Void, Void>("/ci/conf/fact/kb_doc/online", &Void {}).await.code,
        "404-spi-stats-fact_conf-create_inst"
    );

    test_fact_col_conf(client).await?;

    // online
    assert_eq!(
        client.put_resp::<Void, Void>("/ci/conf/fact/kb_doc/online", &Void {}).await.code,
        "404-spi-stats-fact_conf-create_inst"
    );
    let _: Void = client.put("/ci/conf/fact/req/online", &Void {}).await;

    // can't modify fact after online error
    assert_eq!(
        client
            .patch_resp::<StatsConfFactModifyReq, Void>(
                "/ci/conf/fact/req",
                &StatsConfFactModifyReq {
                    show_name: Some("需求".to_string()),
                    query_limit: Some(2000),
                    remark: Some("需求说明".to_string()),
                }
            )
            .await
            .code,
        "409-spi-stats-fact_conf-modify"
    );

    // can't modify fact column after online error
    assert_eq!(
        client
            .patch_resp::<StatsConfFactColModifyReq, Void>(
                "/ci/conf/fact/req/col/source",
                &StatsConfFactColModifyReq {
                    show_name: Some("来源".to_string()),
                    remark: Some("需求来源说明".to_string()),
                    kind: Some(StatsFactColKind::Dimension),
                    dim_rel_conf_dim_key: Some("address".to_string()),
                    dim_multi_values: Some(false),
                    mes_data_type: None,
                    mes_frequency: None,
                    mes_act_by_dim_conf_keys: None,
                    rel_conf_fact_and_col_key: None,
                },
            )
            .await
            .code,
        "409-spi-stats-fact_col_conf-modify"
    );
    // can't delete fact column after online error
    assert_eq!(client.delete_resp("/ci/conf/fact/req/col/source").await.code, "409-spi-stats-fact_col_conf-delete");

    client.delete("/ci/conf/fact/to_be_del2").await;
    let list: TardisPage<StatsConfFactInfoResp> = client.get("/ci/conf/fact?page_number=1&page_size=10").await;
    assert_eq!(list.total_size, 2);
    assert_eq!(list.records.iter().filter(|d| d.online).count(), 1);

    Ok(())
}

pub async fn test_fact_col_conf(client: &mut TestHttpClient) -> TardisResult<()> {
    // key format error
    assert_eq!(
        client
            .put_resp::<StatsConfFactColAddReq, Void>(
                "/ci/conf/fact/req/col",
                &StatsConfFactColAddReq {
                    key: "Status".to_string(),
                    show_name: "状态".to_string(),
                    remark: Some("状态说明".to_string()),
                    kind: StatsFactColKind::Dimension,
                    dim_rel_conf_dim_key: Some("ssss".to_string()),
                    dim_multi_values: Some(false),
                    mes_data_type: None,
                    mes_frequency: None,
                    mes_act_by_dim_conf_keys: None,
                    rel_conf_fact_and_col_key: None
                },
            )
            .await
            .code,
        "400"
    );

    // dimension not online/exist error
    assert_eq!(
        client
            .put_resp::<StatsConfFactColAddReq, Void>(
                "/ci/conf/fact/req/col",
                &StatsConfFactColAddReq {
                    key: "status".to_string(),
                    show_name: "状态".to_string(),
                    remark: Some("状态说明".to_string()),
                    kind: StatsFactColKind::Dimension,
                    dim_rel_conf_dim_key: Some("ssss".to_string()),
                    dim_multi_values: Some(false),
                    mes_data_type: None,
                    mes_frequency: None,
                    mes_act_by_dim_conf_keys: None,
                    rel_conf_fact_and_col_key: None
                },
            )
            .await
            .code,
        "409-spi-stats-fact_col_conf-add"
    );

    let _: Void = client
        .put(
            "/ci/conf/fact/req/col",
            &StatsConfFactColAddReq {
                key: "status".to_string(),
                show_name: "状态".to_string(),
                remark: Some("状态说明".to_string()),
                kind: StatsFactColKind::Dimension,
                dim_rel_conf_dim_key: Some("req_status".to_string()),
                dim_multi_values: Some(false),
                mes_data_type: None,
                mes_frequency: None,
                mes_act_by_dim_conf_keys: None,
                rel_conf_fact_and_col_key: None,
            },
        )
        .await;

    // key exist error
    assert_eq!(
        client
            .put_resp::<StatsConfFactColAddReq, Void>(
                "/ci/conf/fact/req/col",
                &StatsConfFactColAddReq {
                    key: "status".to_string(),
                    show_name: "状态".to_string(),
                    remark: Some("状态说明".to_string()),
                    kind: StatsFactColKind::Dimension,
                    dim_rel_conf_dim_key: Some("req_status".to_string()),
                    dim_multi_values: Some(false),
                    mes_data_type: None,
                    mes_frequency: None,
                    mes_act_by_dim_conf_keys: None,
                    rel_conf_fact_and_col_key: None
                },
            )
            .await
            .code,
        "409-spi-stats-fact_col_conf-add"
    );

    let _: Void = client
        .put(
            "/ci/conf/fact/req/col",
            &StatsConfFactColAddReq {
                key: "priority".to_string(),
                show_name: "优先级".to_string(),
                remark: Some("优先级说明".to_string()),
                kind: StatsFactColKind::Dimension,
                dim_rel_conf_dim_key: Some("req_priority".to_string()),
                dim_multi_values: Some(false),
                mes_data_type: None,
                mes_frequency: None,
                mes_act_by_dim_conf_keys: None,
                rel_conf_fact_and_col_key: None,
            },
        )
        .await;
    let _: Void = client
        .put(
            "/ci/conf/fact/req/col",
            &StatsConfFactColAddReq {
                key: "tag".to_string(),
                show_name: "标签".to_string(),
                remark: Some("标签说明".to_string()),
                kind: StatsFactColKind::Dimension,
                dim_rel_conf_dim_key: Some("tag".to_string()),
                dim_multi_values: Some(true),
                mes_data_type: None,
                mes_frequency: None,
                mes_act_by_dim_conf_keys: None,
                rel_conf_fact_and_col_key: None,
            },
        )
        .await;
    let _: Void = client
        .put(
            "/ci/conf/fact/req/col",
            &StatsConfFactColAddReq {
                key: "creator".to_string(),
                show_name: "创建人".to_string(),
                remark: Some("创建人说明".to_string()),
                kind: StatsFactColKind::Dimension,
                dim_rel_conf_dim_key: Some("account".to_string()),
                dim_multi_values: Some(false),
                mes_data_type: None,
                mes_frequency: None,
                mes_act_by_dim_conf_keys: None,
                rel_conf_fact_and_col_key: None,
            },
        )
        .await;
    let _: Void = client
        .put(
            "/ci/conf/fact/req/col",
            &StatsConfFactColAddReq {
                key: "source".to_string(),
                show_name: "来源_to_be_modify".to_string(),
                remark: Some("需求来源说明1".to_string()),
                kind: StatsFactColKind::Dimension,
                dim_rel_conf_dim_key: Some("req_status".to_string()),
                dim_multi_values: Some(true),
                mes_data_type: None,
                mes_frequency: None,
                mes_act_by_dim_conf_keys: None,
                rel_conf_fact_and_col_key: None,
            },
        )
        .await;
    let _: Void = client
        .patch(
            "/ci/conf/fact/req/col/source",
            &StatsConfFactColModifyReq {
                show_name: Some("来源".to_string()),
                remark: Some("需求来源说明".to_string()),
                kind: Some(StatsFactColKind::Dimension),
                dim_rel_conf_dim_key: Some("address".to_string()),
                dim_multi_values: Some(false),
                mes_data_type: None,
                mes_frequency: None,
                mes_act_by_dim_conf_keys: None,
                rel_conf_fact_and_col_key: None,
            },
        )
        .await;
    let _: Void = client
        .put(
            "/ci/conf/fact/req/col",
            &StatsConfFactColAddReq {
                key: "act_hours".to_string(),
                show_name: "实例工时".to_string(),
                remark: None,
                kind: StatsFactColKind::Measure,
                dim_rel_conf_dim_key: None,
                dim_multi_values: None,
                mes_data_type: Some(StatsDataTypeKind::Number),
                mes_frequency: Some("RT".to_string()),
                mes_act_by_dim_conf_keys: Some(vec!["account".to_string()]),
                rel_conf_fact_and_col_key: None,
            },
        )
        .await;
    let _: Void = client
        .put(
            "/ci/conf/fact/req/col",
            &StatsConfFactColAddReq {
                key: "plan_hours".to_string(),
                show_name: "计划工时_to_be_modify".to_string(),
                remark: None,
                kind: StatsFactColKind::Measure,
                dim_rel_conf_dim_key: None,
                dim_multi_values: None,
                mes_data_type: Some(StatsDataTypeKind::Boolean),
                mes_frequency: Some("RT".to_string()),
                mes_act_by_dim_conf_keys: None,
                rel_conf_fact_and_col_key: None,
            },
        )
        .await;
    let _: Void = client
        .patch(
            "/ci/conf/fact/req/col/plan_hours",
            &StatsConfFactColModifyReq {
                show_name: Some("计划工时".to_string()),
                remark: Some("计划工时说明".to_string()),
                kind: Some(StatsFactColKind::Measure),
                dim_rel_conf_dim_key: None,
                dim_multi_values: None,
                mes_data_type: Some(StatsDataTypeKind::Number),
                mes_frequency: Some("1H".to_string()),
                mes_act_by_dim_conf_keys: Some(vec!["account".to_string()]),
                rel_conf_fact_and_col_key: None,
            },
        )
        .await;
    let _: Void = client
        .put(
            "/ci/conf/fact/req/col",
            &StatsConfFactColAddReq {
                key: "to_be_del".to_string(),
                show_name: "删除测试".to_string(),
                remark: None,
                kind: StatsFactColKind::Measure,
                dim_rel_conf_dim_key: None,
                dim_multi_values: None,
                mes_data_type: Some(StatsDataTypeKind::Number),
                mes_frequency: Some("RT".to_string()),
                mes_act_by_dim_conf_keys: Some(vec!["account".to_string()]),
                rel_conf_fact_and_col_key: None,
            },
        )
        .await;

    let list: TardisPage<StatsConfFactColInfoResp> = client.get("/ci/conf/fact/req/col?page_number=1&page_size=10").await;
    assert_eq!(list.total_size, 8);
    let list: TardisPage<StatsConfFactColInfoResp> = client.get("/ci/conf/fact/req/col?page_number=1&page_size=10&show_name=工时").await;
    assert_eq!(list.total_size, 2);
    let list: TardisPage<StatsConfFactColInfoResp> = client.get("/ci/conf/fact/req/col?page_number=1&page_size=10&key=source").await;
    assert_eq!(list.total_size, 1);
    assert_eq!(list.records[0].key, "source");
    assert_eq!(list.records[0].show_name, "来源");
    assert_eq!(list.records[0].remark, Some("需求来源说明".to_string()));
    assert_eq!(list.records[0].kind, StatsFactColKind::Dimension);
    assert_eq!(list.records[0].dim_rel_conf_dim_key, Some("address".to_string()));
    assert_eq!(list.records[0].dim_multi_values, Some(false));
    assert_eq!(list.records[0].mes_data_type, None);
    assert_eq!(list.records[0].mes_frequency, None);
    assert_eq!(list.records[0].mes_act_by_dim_conf_keys, None);
    assert_eq!(list.records[0].rel_conf_fact_and_col_key, None);
    let list: TardisPage<StatsConfFactColInfoResp> = client.get("/ci/conf/fact/req/col?page_number=1&page_size=10&show_name=工时&key=plan_hours").await;
    assert_eq!(list.total_size, 1);
    assert_eq!(list.records[0].key, "plan_hours");
    assert_eq!(list.records[0].show_name, "计划工时");
    assert_eq!(list.records[0].remark, Some("计划工时说明".to_string()));
    assert_eq!(list.records[0].kind, StatsFactColKind::Measure);
    assert_eq!(list.records[0].dim_rel_conf_dim_key, None);
    assert_eq!(list.records[0].dim_multi_values, None);
    assert_eq!(list.records[0].mes_data_type, Some(StatsDataTypeKind::Number));
    assert_eq!(list.records[0].mes_frequency, Some("1H".to_string()));
    assert_eq!(list.records[0].mes_act_by_dim_conf_keys, Some(vec!["account".to_string()]));
    assert_eq!(list.records[0].rel_conf_fact_and_col_key, None);

    client.delete("/ci/conf/fact/req/col/to_be_del").await;
    let list: TardisPage<StatsConfFactColInfoResp> = client.get("/ci/conf/fact/req/col?page_number=1&page_size=10").await;
    assert_eq!(list.total_size, 7);

    Ok(())
}
