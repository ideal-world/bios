use bios_basic::test::test_http_client::TestHttpClient;
use bios_spi_stats::dto::stats_conf_dto::{StatsConfDimAddReq, StatsConfDimInfoResp, StatsConfDimModifyReq};
use bios_spi_stats::stats_enumeration::StatsDataTypeKind;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::web::web_resp::{TardisPage, Void};

pub async fn test(client: &mut TestHttpClient) -> TardisResult<()> {
    client.set_auth(&TardisContext {
        own_paths: "t1/a1".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "app001".to_string(),
        ..Default::default()
    })?;
    test_dim_conf(client).await?;

    Ok(())
}

pub async fn test_dim_conf(client: &mut TestHttpClient) -> TardisResult<()> {
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
    let _: Void = client
        .put(
            "/ci/conf/dim",
            &StatsConfDimAddReq {
                key: "tag".to_string(),
                show_name: "标签".to_string(),
                stable_ds: false,
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

    client.delete("/ci/conf/dim/to_be_del2").await;
    let list: TardisPage<StatsConfDimInfoResp> = client.get("/ci/conf/dim?page_number=1&page_size=10").await;
    assert_eq!(list.total_size, 5);
    assert_eq!(list.records.iter().filter(|d| d.online).count(), 5);

    Ok(())
}
