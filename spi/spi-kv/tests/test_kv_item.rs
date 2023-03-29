use std::time::Duration;

use bios_basic::test::test_http_client::TestHttpClient;
use bios_spi_kv::dto::kv_item_dto::{KvItemDetailResp, KvItemSummaryResp, KvNameFindResp, KvTagFindResp};
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::serde_json::json;
use tardis::tokio::time::sleep;
use tardis::web::web_resp::{TardisPage, TardisResp, Void};

pub async fn test(client: &mut TestHttpClient) -> TardisResult<()> {
    client.set_auth(&TardisContext {
        own_paths: "t1/app001".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "app001".to_string(),
        ..Default::default()
    })?;

    let _: Void = client
        .put(
            "/ci/item",
            &json!({
                "key":"db:url",
                "value": "postgres://xxxx",
                "info":"xx系统的数据库地址",
            }),
        )
        .await;

    let result: KvItemDetailResp = client.get("/ci/item/?key=db:url").await;
    assert_eq!(result.key, "db:url");
    assert_eq!(result.value, "postgres://xxxx");
    assert_eq!(result.info, "xx系统的数据库地址");

    let _: Void = client
        .put(
            "/ci/item",
            &json!({
                "key":"db_info:001",
                "value": {
                    "url": "postgres://xxxx001",
                    "username":"pg",
                    "password":"11111"

                },
                "info":"xx系统的数据库信息",
            }),
        )
        .await;

    let _: Void = client
        .put(
            "/ci/item",
            &json!({
                "key":"db_info:002",
                "value": {
                    "url": "postgres://xxxx001",
                    "username":"pg",
                    "password":"11111"

                },
                "info":"xx系统的数据库信息",
            }),
        )
        .await;
    sleep(Duration::from_millis(100)).await;

    let _: Void = client
        .put(
            "/ci/item",
            &json!({
                "key":" db_info:002 ",
                "value": {
                    "url": "postgres://xxxx002",
                    "username":"pg",
                    "password":"2222"

                },
                "info":"002系统的数据库信息",
            }),
        )
        .await;

    let result: KvItemDetailResp = client.get("/ci/item/?key=db_info:002").await;
    assert_eq!(result.key, "db_info:002");
    assert_eq!(result.info, "002系统的数据库信息");
    assert_eq!(result.value.get("url").unwrap().as_str().unwrap(), "postgres://xxxx002");
    assert!(result.create_time < result.update_time);

    let result: KvItemDetailResp = client.get("/ci/item/?key=db_info:002&extract=url").await;
    assert_eq!(result.key, "db_info:002");
    assert_eq!(result.info, "002系统的数据库信息");
    assert_eq!(result.value.as_str().unwrap(), "postgres://xxxx002");

    let result: Vec<KvItemSummaryResp> = client.get("/ci/items/?keys=db_info:002&keys=db_info:001").await;
    assert_eq!(result.len(), 2);
    assert_eq!(result[1].key, "db_info:002");
    assert_eq!(result[1].info, "002系统的数据库信息");
    assert_eq!(result[1].value.get("url").unwrap().as_str().unwrap(), "postgres://xxxx002");

    let result: Vec<KvItemSummaryResp> = client.get("/ci/items/?keys=db_info:002&keys=db_info:001&&extract=url").await;
    assert_eq!(result.len(), 2);
    assert_eq!(result[1].key, "db_info:002");
    assert_eq!(result[1].info, "002系统的数据库信息");
    assert_eq!(result[1].value.as_str().unwrap(), "postgres://xxxx002");

    let result: TardisPage<KvItemSummaryResp> = client.get("/ci/item/match?key_prefix=db_info&page_number=1&page_size=10").await;
    assert_eq!(result.total_size, 2);
    assert_eq!(result.records[1].key, "db_info:002");
    assert_eq!(result.records[1].info, "002系统的数据库信息");
    assert_eq!(result.records[1].value.get("url").unwrap().as_str().unwrap(), "postgres://xxxx002");

    let result: TardisPage<KvItemSummaryResp> = client.get("/ci/item/match?key_prefix=db_info&extract=url&page_number=1&page_size=10").await;
    assert_eq!(result.total_size, 2);
    assert_eq!(result.records[1].key, "db_info:002");
    assert_eq!(result.records[1].info, "002系统的数据库信息");
    assert_eq!(result.records[1].value.as_str().unwrap(), "postgres://xxxx002");

    client.delete("/ci/item?key=db_info:001").await;

    let result: TardisResp<KvItemDetailResp> = client.get_resp("/ci/item/?key=db_info:001").await;
    assert!(result.data.is_none());

    // key-Name

    let _: Void = client
        .put(
            "/ci/scene/key-name",
            &json!({
                "key":"account001",
                "name": "星航"
            }),
        )
        .await;

    let result: Vec<KvNameFindResp> = client.get("/ci/scene/key-names?keys=account001").await;
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].key, "account001");
    assert_eq!(result[0].name, "星航");

    let _: Void = client
        .put(
            "/ci/scene/key-name",
            &json!({
                "key":"account001",
                "name": "星航大大"
            }),
        )
        .await;

    let result: Vec<KvNameFindResp> = client.get("/ci/scene/key-names?keys=account001").await;
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].key, "account001");
    assert_eq!(result[0].name, "星航大大");

    // tag

    let _: Void = client
        .put(
            "/ci/scene/tag",
            &json!({
                "key":"feed:priority",
                "items":[
                    {
                        "code":"01",
                        "label":"紧急",
                        "color":"",
                        "icon":""
                    },
                    {
                        "code":"02",
                        "label":"高",
                        "color":"",
                        "icon":""
                    },
                    {
                        "code":"03",
                        "label":"低",
                        "color":"",
                        "icon":""
                    }
                ]
            }),
        )
        .await;

    let _: Void = client
        .put(
            "/ci/scene/tag",
            &json!({
                "key":"feed:kind",
                "items":[
                    {
                        "code":"req",
                        "label":"需求",
                        "color":"",
                        "icon":""
                    },
                    {
                        "code":"task",
                        "label":"任务",
                        "color":"",
                        "icon":""
                    },
                    {
                        "code":"bug",
                        "label":"缺陷",
                        "color":"",
                        "icon":""
                    }
                ]
            }),
        )
        .await;

    let result: TardisPage<KvTagFindResp> = client.get("/ci/scene/tags?key_prefix=feed:&page_number=1&page_size=10").await;
    assert_eq!(result.total_size, 2);
    assert_eq!(result.records[1].key, "feed:kind");
    assert_eq!(result.records[1].items[1].code, "task");
    Ok(())
}
