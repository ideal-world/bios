use std::time::Duration;

use bios_basic::test::test_http_client::TestHttpClient;
use bios_spi_reldb::dto::reldb_exec_dto::{ReldbDdlReq, ReldbDmlReq, ReldbDmlResp, ReldbDqlReq, ReldbTxResp};
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::serde_json::{json, Value};
use tardis::tokio::time::sleep;
use tardis::web::web_resp::{TardisResp, Void};

pub async fn test(client: &mut TestHttpClient) -> TardisResult<()> {
    client.set_auth(&TardisContext {
        own_paths: "t1/a1".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "app001".to_string(),
        ..Default::default()
    })?;

    test_basic(client).await?;
    test_tx_normal(client).await?;
    test_tx_rollback(client).await?;
    test_tx_error(client).await?;
    test_tx_auto_commit(client).await?;
    test_tx_auto_rollback(client).await?;

    Ok(())
}

pub async fn test_basic(client: &mut TestHttpClient) -> TardisResult<()> {
    info!("【test_basic】");

    let _: Void = client
        .post(
            "/ci/exec/ddl",
            &ReldbDdlReq {
                sql: "create table test_table (id int primary key, name varchar)".to_string(),
                params: json!([]),
            },
        )
        .await;

    let dml_resp: ReldbDmlResp = client
        .post(
            "/ci/exec/dml",
            &ReldbDmlReq {
                sql: "insert into test_table (id,name) values ($1,$2)".to_string(),
                params: json!([10, "大大"]),
            },
        )
        .await;

    assert_eq!(dml_resp.affected_rows, 1);

    let dql_resp: Value = client
        .put(
            "/ci/exec/dql",
            &ReldbDqlReq {
                sql: "select * from test_table where id = $1".to_string(),
                params: json!([10]),
            },
        )
        .await;

    assert_eq!(dql_resp.to_string(), r#"[{"id":10,"name":"大大"}]"#);

    Ok(())
}

pub async fn test_tx_normal(client: &mut TestHttpClient) -> TardisResult<()> {
    info!("【test_tx_normal】");

    let _: Void = client
        .post(
            "/ci/exec/ddl",
            &ReldbDdlReq {
                sql: "create table test_tx_normal (id int primary key, name varchar)".to_string(),
                params: json!([]),
            },
        )
        .await;
    let tx_resp: ReldbTxResp = client.get("/ci/exec/tx?auto_commit=false").await;
    let tx_id = tx_resp.tx_id;
    let _: ReldbDmlResp = client
        .post(
            &format!("/ci/exec/dml?tx_id={}", tx_id),
            &ReldbDmlReq {
                sql: "insert into test_tx_normal (id,name) values ($1,$2)".to_string(),
                params: json!([11, "事务测试1"]),
            },
        )
        .await;
    let _: ReldbDmlResp = client
        .post(
            &format!("/ci/exec/dml?tx_id={}", tx_id),
            &ReldbDmlReq {
                sql: "insert into test_tx_normal (id,name) values ($1,$2)".to_string(),
                params: json!([12, "事务测试2"]),
            },
        )
        .await;
    let _: Void = client.put(&format!("/ci/exec/tx?tx_id={}", tx_id), &Void {}).await;

    let dql_resp: Value = client
        .put(
            "/ci/exec/dql",
            &ReldbDqlReq {
                sql: "select * from test_tx_normal".to_string(),
                params: json!([]),
            },
        )
        .await;

    assert_eq!(dql_resp.to_string(), r#"[{"id":11,"name":"事务测试1"},{"id":12,"name":"事务测试2"}]"#);

    Ok(())
}

pub async fn test_tx_rollback(client: &mut TestHttpClient) -> TardisResult<()> {
    info!("【test_tx_rollback】");

    let _: Void = client
        .post(
            "/ci/exec/ddl",
            &ReldbDdlReq {
                sql: "create table test_tx_rollback (id int primary key, name varchar)".to_string(),
                params: json!([]),
            },
        )
        .await;
    let _: ReldbDmlResp = client
        .post(
            "/ci/exec/dml",
            &ReldbDmlReq {
                sql: "insert into test_tx_rollback (id,name) values ($1,$2)".to_string(),
                params: json!([20, "事务测试1"]),
            },
        )
        .await;
    let tx_resp: ReldbTxResp = client.get("/ci/exec/tx?auto_commit=false").await;
    let tx_id = tx_resp.tx_id;
    let _: ReldbDmlResp = client
        .post(
            &format!("/ci/exec/dml?tx_id={}", tx_id),
            &ReldbDmlReq {
                sql: "insert into test_tx_rollback (id,name) values ($1,$2)".to_string(),
                params: json!([21, "事务测试3"]),
            },
        )
        .await;
    let _: ReldbDmlResp = client
        .post(
            &format!("/ci/exec/dml?tx_id={}", tx_id),
            &ReldbDmlReq {
                sql: "insert into test_tx_rollback (id,name) values ($1,$2)".to_string(),
                params: json!([22, "事务测试4"]),
            },
        )
        .await;

    client.delete(&format!("/ci/exec/tx?tx_id={}", tx_id)).await;
    let _: ReldbDmlResp = client
        .post(
            "/ci/exec/dml",
            &ReldbDmlReq {
                sql: "insert into test_tx_rollback (id,name) values ($1,$2)".to_string(),
                params: json!([21, "事务测试2"]),
            },
        )
        .await;
    let dql_resp: Value = client
        .put(
            "/ci/exec/dql",
            &ReldbDqlReq {
                sql: "select * from test_tx_rollback".to_string(),
                params: json!([]),
            },
        )
        .await;

    assert_eq!(dql_resp.to_string(), r#"[{"id":20,"name":"事务测试1"},{"id":21,"name":"事务测试2"}]"#);

    Ok(())
}

pub async fn test_tx_error(client: &mut TestHttpClient) -> TardisResult<()> {
    info!("【test_tx_error】");

    let _: Void = client
        .post(
            "/ci/exec/ddl",
            &ReldbDdlReq {
                sql: "create table test_tx_error (id int primary key, name varchar)".to_string(),
                params: json!([]),
            },
        )
        .await;
    let _: ReldbDmlResp = client
        .post(
            "/ci/exec/dml",
            &ReldbDmlReq {
                sql: "insert into test_tx_error (id,name) values ($1,$2)".to_string(),
                params: json!([20, "事务测试1"]),
            },
        )
        .await;
    let tx_resp: ReldbTxResp = client.get("/ci/exec/tx?auto_commit=false").await;
    let tx_id = tx_resp.tx_id;
    let _: ReldbDmlResp = client
        .post(
            &format!("/ci/exec/dml?tx_id={}", tx_id),
            &ReldbDmlReq {
                sql: "insert into test_tx_error (id,name) values ($1,$2)".to_string(),
                params: json!([21, "事务测试3"]),
            },
        )
        .await;
    let dml_resp: TardisResp<ReldbDmlResp> = client
        .post_resp(
            &format!("/ci/exec/dml?tx_id={}", tx_id),
            &ReldbDmlReq {
                sql: "insert into test_tx_error (id,name) values ($1,$2)".to_string(),
                params: json!([21, "事务测试4"]),
            },
        )
        .await;
    assert_eq!(dml_resp.code, "-1");
    let _: ReldbDmlResp = client
        .post(
            "/ci/exec/dml",
            &ReldbDmlReq {
                sql: "insert into test_tx_error (id,name) values ($1,$2)".to_string(),
                params: json!([21, "事务测试2"]),
            },
        )
        .await;

    let dql_resp: Value = client
        .put(
            "/ci/exec/dql",
            &ReldbDqlReq {
                sql: "select * from test_tx_error".to_string(),
                params: json!([]),
            },
        )
        .await;

    assert_eq!(dql_resp.to_string(), r#"[{"id":20,"name":"事务测试1"},{"id":21,"name":"事务测试2"}]"#);

    Ok(())
}

pub async fn test_tx_auto_commit(client: &mut TestHttpClient) -> TardisResult<()> {
    info!("【test_tx_auto_commit】");

    let _: Void = client
        .post(
            "/ci/exec/ddl",
            &ReldbDdlReq {
                sql: "create table test_tx_auto_commit (id int primary key, name varchar)".to_string(),
                params: json!([]),
            },
        )
        .await;
    let tx_resp: ReldbTxResp = client.get("/ci/exec/tx?auto_commit=true&exp_sec=1").await;
    let tx_id = tx_resp.tx_id;
    let _: ReldbDmlResp = client
        .post(
            &format!("/ci/exec/dml?tx_id={}", tx_id),
            &ReldbDmlReq {
                sql: "insert into test_tx_auto_commit (id,name) values ($1,$2)".to_string(),
                params: json!([1, "事务测试"]),
            },
        )
        .await;

    let dql_resp: Value = client
        .put(
            "/ci/exec/dql",
            &ReldbDqlReq {
                sql: "select * from test_tx_auto_commit".to_string(),
                params: json!([]),
            },
        )
        .await;

    assert_eq!(dql_resp.to_string(), r#"[]"#);

    sleep(Duration::from_secs(5)).await;

    let dql_resp: Value = client
        .put(
            "/ci/exec/dql",
            &ReldbDqlReq {
                sql: "select * from test_tx_auto_commit".to_string(),
                params: json!([]),
            },
        )
        .await;

    assert_eq!(dql_resp.to_string(), r#"[{"id":1,"name":"事务测试"}]"#);

    Ok(())
}

pub async fn test_tx_auto_rollback(client: &mut TestHttpClient) -> TardisResult<()> {
    info!("【test_tx_auto_rollback】");

    let _: Void = client
        .post(
            "/ci/exec/ddl",
            &ReldbDdlReq {
                sql: "create table test_tx_auto_rollback (id int primary key, name varchar)".to_string(),
                params: json!([]),
            },
        )
        .await;
    let tx_resp: ReldbTxResp = client.get("/ci/exec/tx?auto_commit=false&exp_sec=1").await;
    let tx_id = tx_resp.tx_id;
    let _: ReldbDmlResp = client
        .post(
            &format!("/ci/exec/dml?tx_id={}", tx_id),
            &ReldbDmlReq {
                sql: "insert into test_tx_auto_rollback (id,name) values ($1,$2)".to_string(),
                params: json!([1, "事务测试"]),
            },
        )
        .await;

    let dql_resp: Value = client
        .put(
            &format!("/ci/exec/dql?tx_id={}", tx_id),
            &ReldbDqlReq {
                sql: "select * from test_tx_auto_rollback".to_string(),
                params: json!([]),
            },
        )
        .await;

    assert_eq!(dql_resp.to_string(), r#"[{"id":1,"name":"事务测试"}]"#);

    let dql_resp: Value = client
        .put(
            "/ci/exec/dql",
            &ReldbDqlReq {
                sql: "select * from test_tx_auto_rollback".to_string(),
                params: json!([]),
            },
        )
        .await;

    assert_eq!(dql_resp.to_string(), r#"[]"#);

    sleep(Duration::from_secs(5)).await;

    let dql_resp: TardisResp<Value> = client
        .put_resp(
            &format!("/ci/exec/dql?tx_id={}", tx_id),
            &ReldbDqlReq {
                sql: "select * from test_tx_auto_rollback".to_string(),
                params: json!([]),
            },
        )
        .await;

    assert_eq!(dql_resp.code, "400");

    let dql_resp: Value = client
        .put(
            "/ci/exec/dql",
            &ReldbDqlReq {
                sql: "select * from test_tx_auto_rollback".to_string(),
                params: json!([]),
            },
        )
        .await;

    assert_eq!(dql_resp.to_string(), r#"[]"#);

    Ok(())
}
