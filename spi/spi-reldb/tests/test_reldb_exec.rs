use bios_basic::test::test_http_client::TestHttpClient;
use bios_spi_reldb::dto::reldb_exec_dto::{ReldbDdlReq, ReldbDmlReq, ReldbDmlResp, ReldbDqlReq};
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::serde_json::{json, Value};
use tardis::web::web_resp::Void;

pub async fn test(client: &mut TestHttpClient) -> TardisResult<()> {
    client.set_auth(&TardisContext {
        own_paths: "t1/a1".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "app001".to_string(),
        ..Default::default()
    })?;

    let _: Void = client
        .post(
            "/ci/exec/ddl",
            &ReldbDdlReq {
                sql: "create table test_table (id int,name varchar)".to_string(),
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
                tx_id: None,
            },
        )
        .await;

    assert_eq!(dml_resp.affected_rows, 1);

    let dml_resp: Value = client
        .put(
            "/ci/exec/dql",
            &ReldbDqlReq {
                sql: "select * from test_table where id = $1".to_string(),
                params: json!([10]),
                tx_id: None,
            },
        )
        .await;

    assert_eq!(dml_resp.to_string(), r#"[{"id":10,"name":"大大"}]"#);
    Ok(())
}
