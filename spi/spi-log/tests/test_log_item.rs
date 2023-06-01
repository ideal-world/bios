use bios_basic::test::test_http_client::TestHttpClient;
use bios_spi_log::dto::log_item_dto::LogItemFindResp;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::serde_json::json;
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
        .post(
            "/ci/item",
            &json!({
                "tag":"audit",
                "content": r#"账号[xxxx]登录系统"#,
                "op":"login"
            }),
        )
        .await;

    let _: Void = client
        .post(
            "/ci/item",
            &json!({
                "tag":"feed",
                "key": "001",
                "content": r#"{"content":"在任意信息流（FEED，包含需求、任务、缺陷、文档等）中输入#号时出现一个跟随光标的快捷搜索小窗口，可以输入编号或内容模糊匹配对应的数据，如果存在，则可以选中对应的数据并显示在文本中。","title":"全局#号搜索","kind":"req","assign_to":"account002"}"#,
                "op":"init",
                "ts":"2022-09-26T23:23:59.000Z",
                "rel_key":"app001"
            }),
        )
        .await;

    let _: Void = client
        .post(
            "/ci/item",
            &json!({
                "tag":"feed",
                "key": "001",
                "content": r#"{"assign_to":"account004"}"#,
                "op":"modify",
                "ts":"2022-09-27T23:23:59.000Z",
                "rel_key":"app001"
            }),
        )
        .await;

    let _: Void = client
        .post(
            "/ci/item",
            &json!({
                "tag":"feed",
                "key": "002",
                "content": r#"{"content":"账号登录 登录名：默认提示：用户名/手机号/邮箱，输入类型不限，最多输入30个字 密码：默认提示：密码，输入类型不限，最多输入30个字； 登录：1、点击判断用户名和密码是否已填写，如果没有则在每个必填项下提示...","title":"新增全局账号逻辑","kind":"req","assign_to":"account002"}"#,
                "op":"init",
                "ts":"2022-09-26T23:23:50.000Z",
                "rel_key":"app002"
            }),
        )
        .await;

    let _: Void = client
        .post(
            "/ci/item",
            &json!({
                "tag":"project",
                "kind":"req",
                "key": "001",
                "content": r#"{"content":"账号登录 登录名：默认提示：用户名/手机号/邮箱，输入类型不限，最多输入30个字 密码：默认提示：密码，输入类型不限，最多输入30个字； 登录：1、点击判断用户名和密码是否已填写，如果没有则在每个必填项下提示...","title":"新增全局账号逻辑","kind":"req","assign_to":"account002"}"#,
                "ext": {"name":"测试","status":1,"apps":["app01"],"assign_to":"account002"},
                "owner":"account002",
                "own_paths":"tenant001",
                "op":"init",
                "ts":"2022-09-26T23:23:50.000Z",
                "rel_key":"app002"
            }),
        )
        .await;

    let find_result: TardisResp<TardisPage<LogItemFindResp>> = client
        .put_resp(
            "/ci/item/find",
            &json!({
                "tag":"feed2",
                "page_number":1,
                "page_size":10
            }),
        )
        .await;
    assert!(find_result.code.starts_with("400"));

    let find_result: TardisPage<LogItemFindResp> = client
        .put(
            "/ci/item/find",
            &json!({
                "tag":"feed",
                "page_number":1,
                "page_size":10
            }),
        )
        .await;
    assert_eq!(find_result.total_size, 3);
    assert_eq!(find_result.records[0].key, "001");
    assert_eq!(find_result.records[0].op, "modify");

    let find_result: TardisPage<LogItemFindResp> = client
        .put(
            "/ci/item/find",
            &json!({
                "tag":"feed",
                "keys":["001"],
                "page_number":1,
                "page_size":10
            }),
        )
        .await;
    assert_eq!(find_result.total_size, 2);
    assert_eq!(find_result.records[0].key, "001");
    assert_eq!(find_result.records[0].op, "modify");

    let find_result: TardisPage<LogItemFindResp> = client
        .put(
            "/ci/item/find",
            &json!({
                "tag":"feed",
                "keys":["001"],
                "ops":["init"],
                "page_number":1,
                "page_size":10
            }),
        )
        .await;
    assert_eq!(find_result.total_size, 1);
    assert_eq!(find_result.records[0].key, "001");
    assert_eq!(find_result.records[0].op, "init");

    let find_result: TardisPage<LogItemFindResp> = client
        .put(
            "/ci/item/find",
            &json!({
                "tag":"feed",
                "rel_keys":["app001"],
                "page_number":1,
                "page_size":10
            }),
        )
        .await;
    assert_eq!(find_result.total_size, 2);
    assert_eq!(find_result.records[0].key, "001");
    assert_eq!(find_result.records[0].op, "modify");

    let find_result: TardisPage<LogItemFindResp> = client
        .put(
            "/ci/item/find",
            &json!({
                "tag":"feed",
                "ts_start":"2022-09-26T23:23:50.000Z",
                "ts_end":"2022-09-27T01:23:59.000Z",
                "page_number":1,
                "page_size":10
            }),
        )
        .await;
    assert_eq!(find_result.total_size, 2);
    assert_eq!(find_result.records[0].key, "001");
    assert_eq!(find_result.records[0].op, "init");
    assert_eq!(find_result.records[1].key, "002");

    let find_result: TardisPage<LogItemFindResp> = client
        .put(
            "/ci/item/find",
            &json!({
                "tag":"feed",
                "keys":["001"],
                "ops":["init","modify"],
                "page_number":1,
                "page_size":10
            }),
        )
        .await;
    assert_eq!(find_result.total_size, 2);
    assert_eq!(find_result.records[0].key, "001");
    assert_eq!(find_result.records[0].op, "modify");

    let find_result: TardisPage<LogItemFindResp> = client
        .put(
            "/ci/item/find",
            &json!({
                "tag":"project",
                "keys":["001"],
                "kinds":["req"],
                "owners":["account002"],
                "own_paths":"tenant001",
                "ext":[
                    {"field":"name","op":"like","value":"测试"},
                    {"field":"status","op":"=","value":1},
                    {"field":"apps","op":"in","value":["app01"]},
                    {"field":"assign_to","op":"=","value":"account002"}
                ],
                "ops":["init","modify"],
                "page_number":1,
                "page_size":10
            }),
        )
        .await;
    assert_eq!(find_result.total_size, 1);
    assert_eq!(find_result.records[0].key, "001");
    assert_eq!(find_result.records[0].op, "init");

    let find_result: TardisPage<LogItemFindResp> = client
        .put(
            "/ci/item/find",
            &json!({
                "tag":"project",
                "keys":["001"],
                "kinds":["req"],
                "owners":["account002"],
                "own_paths":"tenant001",
                "query":"测试",
                "ops":["init","modify"],
                "page_number":1,
                "page_size":10
            }),
        )
        .await;
    assert_eq!(find_result.total_size, 1);
    assert_eq!(find_result.records[0].key, "001");
    assert_eq!(find_result.records[0].op, "init");
    Ok(())
}
