use bios_basic::test::test_http_client::TestHttpClient;
use bios_spi_search::dto::search_item_dto::SearchItemSearchResp;
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
                "tag":"feed",
                "kind": "req",
                "key": "001",
                "title": "搜索",
                "content": "在xxx。",
                "owner":"account001",
                "own_paths":"t001",
                "create_time":"2022-11-26T23:23:55.000Z",
                "update_time": "2022-11-27T01:20:20.000Z",
                "ext":{"version":"1.1","xxx":0}
            }),
        )
        .await;
    sleep(std::time::Duration::from_secs(1)).await;
    let _: Void = client
        .put(
            "/ci/item",
            &json!({
                "tag":"feed",
                "kind": "req",
                "key": "002",
                "title": "新增全局账号逻辑",
                "content": "账号登录 登录名：默认提示：用户名/手机号/邮箱，输入类型不限，最多输入30个字 密码：默认提示：密码，输入类型不限，最多输入30个字； 登录：1、点击判断用户名和密码是否已填写，如果没有则在每个必填项下提示：****不能为空；2、判断校验是否正确，没有则提示：用户名或密码不正确；3、没有选择租户，则按登录平台逻辑进行处理；则继续判断该账号是否全局账号，非全局账号则提示：请选择租户登录；是全局账号并校验成功则登录平台；4、选择租户后，密码校验成功后登录对应的租户",
                "owner":"account002",
                "own_paths":"t001/a001",
                "create_time":"2022-09-26T23:23:56.000Z",
                "update_time": "2022-09-27T01:20:20.000Z",
                "ext":{"start_time":"2022-10-25T14:23:20.000Z","end_time":"2022-10-30T14:23:20.000Z","rel_accounts":["acc01","acc03"],"version":"1.3"},
                "visit_keys":{"apps":["003"],"tenants":["001"],"roles":["sys"]}
            }),
        )
        .await;
    sleep(std::time::Duration::from_secs(1)).await;
    let _: Void = client
        .put(
            "/ci/item",
            &json!({
                "tag":"feed",
                "kind": "task",
                "key": "003",
                "title": "新增知识管理优化",
                "content": "整个知识库优化 1、支持库支持压缩包、文件的上传/下载 点击单个文件下，右侧显示按钮，如下图，上传文件：点击可选择文件进行上传；新建文档/表格/文件夹/图集：点击在该文件夹下新建内容",
                "owner":"account002",
                "own_paths":"t001/a002",
                "create_time":"2022-09-26T23:23:59.000Z",
                "update_time": "2022-09-27T01:20:20.000Z",
                "ext":{"start_time":"2022-09-25T14:23:20.000Z","end_time":"2022-09-30T14:23:20.000Z","rel_accounts":["acc03","acc04"],"version":"1.3","int":1,"bool":false,"float":1.1},
                "visit_keys":{"apps":["003"],"tenants":["001"],"roles":["sys","admin"]}
            }),
        )
        .await;
    sleep(std::time::Duration::from_secs(2)).await;
    let _: Void = client
        .put(
            "/ci/item/feed/001",
            &json!({
                "title": "全局#号搜索",
                "content": "在任意信息流（FEED，包含需求、任务、缺陷、文档等）中输入#号时出现一个跟随光标的快捷搜索小窗口，可以输入编号或内容模糊匹配对应的数据，如果存在，则可以选中对应的数据并显示在文本中。",
                "ext":{"start_time":"2022-11-25T14:23:20.000Z","end_time":"2022-11-30T14:23:20.000Z","rel_accounts":["acc01","acc02"],"version":"1.x"}
            }),
        )
        .await;
    sleep(std::time::Duration::from_secs(2)).await;

    // Search without conditions
    let search_result: TardisResp<TardisPage<SearchItemSearchResp>> = client
        .put_resp(
            "/ci/item/search",
            &json!({
                "tag":"feed2",
                "ctx":{},
                "query":{},
                "page":{"number":1,"size":10,"fetch_total":true}
            }),
        )
        .await;
    assert_ne!(search_result.code, "200".to_string());

    let search_result: TardisPage<SearchItemSearchResp> = client
        .put(
            "/ci/item/search",
            &json!({
                "tag":"feed",
                "ctx":{},
                "query":{},
                "page":{"number":1,"size":10,"fetch_total":true}
            }),
        )
        .await;
    assert_eq!(search_result.total_size, 3);
    assert_eq!(search_result.records[0].key, "001");
    assert_eq!(search_result.records[0].ext.get("xxx").unwrap().as_i64().unwrap(), 0);
    assert_eq!(search_result.records[0].ext.get("version").unwrap().as_str().unwrap(), "1.x");

    // Basic search
    let search_result: TardisPage<SearchItemSearchResp> = client
        .put(
            "/ci/item/search",
            &json!({
                "tag":"feed",
                "ctx":{
                    "apps":["003"]
                },
                "query":{
                    "q": "新增"
                },
                "page":{"number":1,"size":10,"fetch_total":true}
            }),
        )
        .await;
    assert_eq!(search_result.total_size, 2);
    assert_eq!(search_result.records[0].key, "002");
    assert_eq!(search_result.records[1].key, "003");
    let search_result: TardisPage<SearchItemSearchResp> = client
        .put(
            "/ci/item/search",
            &json!({
                "tag":"feed",
                "ctx":{
                    "apps":["003"]
                },
                "query":{
                    "q": "类型 & 上传"
                },
                "page":{"number":1,"size":10,"fetch_total":true}
            }),
        )
        .await;
    assert_eq!(search_result.total_size, 0);
    let search_result: TardisPage<SearchItemSearchResp> = client
        .put(
            "/ci/item/search",
            &json!({
                "tag":"feed",
                "ctx":{
                    "apps":["003"]
                },
                "query":{
                    "q": "类型 | 上传"
                },
                "page":{"number":1,"size":10,"fetch_total":true}
            }),
        )
        .await;
    assert_eq!(search_result.total_size, 0);

    let search_result: TardisPage<SearchItemSearchResp> = client
        .put(
            "/ci/item/search",
            &json!({
                "tag":"feed",
                "ctx":{
                    "apps":["003"]
                },
                "query":{
                    "q": "类型 ｜ 上传",
                    "q_scope": "title_content",
                },
                "page":{"number":1,"size":10,"fetch_total":true}
            }),
        )
        .await;
    assert_eq!(search_result.total_size, 2);
    assert_eq!(search_result.records[0].key, "002");
    assert_eq!(search_result.records[1].key, "003");

    //  Search with ext
    let search_result: TardisPage<SearchItemSearchResp> = client
        .put(
            "/ci/item/search",
            &json!({
                "tag":"feed",
                "ctx":{
                    "apps":["003"]
                },
                "query":{
                    "ext": [{
                        "field":"version",
                        "op":"=",
                        "value":"1.3"
                    }]
                },
                "page":{"number":1,"size":10,"fetch_total":true}
            }),
        )
        .await;
    assert_eq!(search_result.total_size, 2);
    assert_eq!(search_result.records[0].key, "002");
    assert_eq!(search_result.records[1].key, "003");
    let search_result: TardisPage<SearchItemSearchResp> = client
        .put(
            "/ci/item/search",
            &json!({
                "tag":"feed",
                "ctx":{
                    "apps":["003"]
                },
                "query":{
                    "ext": [{
                        "field":"rel_accounts",
                        "op":"in",
                        "value":["acc01"]
                    }]
                },
                "page":{"number":1,"size":10,"fetch_total":true}
            }),
        )
        .await;
    assert_eq!(search_result.total_size, 2);
    assert_eq!(search_result.records[0].key, "001");
    assert_eq!(search_result.records[1].key, "002");
    let search_result: TardisPage<SearchItemSearchResp> = client
        .put(
            "/ci/item/search",
            &json!({
                "tag":"feed",
                "ctx":{
                    "apps":["003"]
                },
                "query":{
                    "ext": [{
                        "field":"rel_accounts",
                        "op":"in",
                        "value":["acc01","acc02"]
                    }]
                },
                "page":{"number":1,"size":10,"fetch_total":true}
            }),
        )
        .await;
    assert_eq!(search_result.total_size, 2);
    assert_eq!(search_result.records[0].key, "001");
    assert_eq!(search_result.records[1].key, "002");
    let search_result: TardisPage<SearchItemSearchResp> = client
        .put(
            "/ci/item/search",
            &json!({
                "tag":"feed",
                "ctx":{
                    "apps":["003"]
                },
                "query":{
                    "ext": [{
                        "field":"end_time",
                        "op":"<=",
                        "value":"2022-10-30T14:23:20.000Z"
                    }]
                },
                "page":{"number":1,"size":10,"fetch_total":true}
            }),
        )
        .await;
    assert_eq!(search_result.total_size, 2);
    assert_eq!(search_result.records[0].key, "002");
    assert_eq!(search_result.records[1].key, "003");
    // search ext with some types
    let search_result: TardisPage<SearchItemSearchResp> = client
        .put(
            "/ci/item/search",
            &json!({
                "tag":"feed",
                "ctx":{
                    "apps":["003"]
                },
                "query":{
                    "ext": [{
                        "field":"int",
                        "op":"=",
                        "value":1
                    },{
                        "field":"bool",
                        "op":"=",
                        "value":false
                    },{
                        "field":"float",
                        "op":"=",
                        "value":1.1
                    }]
                },
                "page":{"number":1,"size":10,"fetch_total":true}
            }),
        )
        .await;
    assert_eq!(search_result.total_size, 1);
    assert_eq!(search_result.records[0].key, "003");

    let search_result: TardisPage<SearchItemSearchResp> = client
        .put(
            "/ci/item/search",
            &json!({
                "tag":"feed",
                "ctx":{
                    "apps":["003"]
                },
                "query":{
                    "kinds": ["req","task"]
                },
                "page":{"number":1,"size":10,"fetch_total":true}
            }),
        )
        .await;
    assert_eq!(search_result.total_size, 3);
    let search_result: TardisPage<SearchItemSearchResp> = client
        .put(
            "/ci/item/search",
            &json!({
                "tag":"feed",
                "ctx":{
                    "apps":["003"]
                },
                "query":{
                    "kinds": ["task"]
                },
                "page":{"number":1,"size":10,"fetch_total":true}
            }),
        )
        .await;
    assert_eq!(search_result.total_size, 1);

    //  Search with auth
    let search_result: TardisPage<SearchItemSearchResp> = client
        .put(
            "/ci/item/search",
            &json!({
                "tag":"feed",
                "ctx":{
                    "apps":["003"]
                },
                "query":{
                },
                "page":{"number":1,"size":10,"fetch_total":true}
            }),
        )
        .await;
    assert_eq!(search_result.total_size, 3);

    //  Search with sort
    let search_result: TardisPage<SearchItemSearchResp> = client
        .put(
            "/ci/item/search",
            &json!({
                "tag":"feed",
                "ctx":{
                    "apps":["003"]
                },
                "query":{
                    "ext": [{
                        "field":"end_time",
                        "op":"<=",
                        "value":"2022-10-30T14:23:20.000Z"
                    }]
                },
                "sort":[{
                    "field":"end_time",
                    "order":"asc"
                }],
                "page":{"number":1,"size":10,"fetch_total":true}
            }),
        )
        .await;
    assert_eq!(search_result.total_size, 2);
    assert_eq!(search_result.records[0].key, "003");
    assert_eq!(search_result.records[1].key, "002");
    let search_result: TardisPage<SearchItemSearchResp> = client
        .put(
            "/ci/item/search",
            &json!({
                "tag":"feed",
                "ctx":{
                    "apps":["003"]
                },
                "query":{
                    "ext": [{
                        "field":"end_time",
                        "op":"<=",
                        "value":"2022-10-30T14:23:20.000Z"
                    }]
                },
                "sort":[{
                    "field":"end_time",
                    "order":"desc"
                }],
                "page":{"number":1,"size":10,"fetch_total":true}
            }),
        )
        .await;
    assert_eq!(search_result.total_size, 2);
    assert_eq!(search_result.records[0].key, "002");
    assert_eq!(search_result.records[1].key, "003");
    // with out total
    let search_result: TardisPage<SearchItemSearchResp> = client
        .put(
            "/ci/item/search",
            &json!({
                "tag":"feed",
                "ctx":{
                    "apps":["003"]
                },
                "query":{
                    "ext": [{
                        "field":"end_time",
                        "op":"<=",
                        "value":"2022-10-30T14:23:20.000Z"
                    }]
                },
                "sort":[{
                    "field":"end_time",
                    "order":"desc"
                }],
                "page":{"number":1,"size":10,"fetch_total":false}
            }),
        )
        .await;
    assert_eq!(search_result.total_size, 0);
    assert_eq!(search_result.records[0].key, "002");
    assert_eq!(search_result.records[1].key, "003");

    //  Search!
    let search_result: TardisPage<SearchItemSearchResp> = client
        .put(
            "/ci/item/search",
            &json!({
                "tag":"feed",
                "ctx":{
                    "apps":["003"],
                    "tenants":["001"],
                    "roles":["root","sys"]
                },
                "query":{
                    "q": "新增",
                    "q_scope": "title_content",
                    "own_paths":["t001"],
                    "ext": [{
                        "field":"end_time",
                        "op":"<=",
                        "value":"2022-10-30T14:23:20.000Z"
                    }]
                },
                "sort":[{
                    "field":"end_time",
                    "order":"asc"
                },{
                    "field":"rank_title",
                    "order":"desc"
                },{
                    "field":"rank_content",
                    "order":"desc"
                }],
                "page":{"number":2,"size":1,"fetch_total":true}
            }),
        )
        .await;
    assert_eq!(search_result.total_size, 2);
    assert_eq!(search_result.records[0].key, "002");

    // Delete
    let search_result: TardisPage<SearchItemSearchResp> = client
        .put(
            "/ci/item/search",
            &json!({
                "tag":"feed",
                "ctx":{},
                "query":{
                    "keys": ["001","0xxx"]
                },
                "page":{"number":1,"size":10,"fetch_total":true}
            }),
        )
        .await;
    assert_eq!(search_result.total_size, 1);
    /// es not implemented
    // client.delete(&format!("/ci/item/{}/{}", "feed", "001")).await;
    let search_result: TardisPage<SearchItemSearchResp> = client
        .put(
            "/ci/item/search",
            &json!({
                "tag":"feed",
                "ctx":{},
                "query":{
                    "keys": ["001"]
                },
                "page":{"number":1,"size":10,"fetch_total":true}
            }),
        )
        .await;
    assert_eq!(search_result.total_size, 0);

    Ok(())
}
