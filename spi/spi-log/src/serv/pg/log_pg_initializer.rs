use std::collections::HashMap;

use bios_basic::spi::{dto::spi_bs_dto::SpiBsCertResp, spi_funs::SpiBsInst, spi_initializer};
use tardis::{
    basic::{dto::TardisContext, error::TardisError, result::TardisResult},
    db::reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
    TardisFuns,
};

pub async fn init(bs_cert: &SpiBsCertResp, ctx: &TardisContext, mgr: bool) -> TardisResult<SpiBsInst> {
    let ext = TardisFuns::json.str_to_json(&bs_cert.ext)?;
    let client = TardisRelDBClient::init(
        &bs_cert.conn_uri,
        ext.get("max_connections").unwrap().as_u64().unwrap() as u32,
        ext.get("min_connections").unwrap().as_u64().unwrap() as u32,
        None,
        None,
    )
    .await?;
    let mut ext = HashMap::new();
    let schema_name = if bs_cert.private {
        "".to_string()
    } else if mgr {
        spi_initializer::common_pg::create_schema(&client, ctx).await?
    } else if spi_initializer::common_pg::check_schema_exit(&client, ctx).await? {
        spi_initializer::common_pg::get_schema_name_from_context(ctx)
    } else {
        return Err(TardisError::bad_request("The requested schema does not exist", ""));
    };
    spi_initializer::common_pg::set_schema_name_to_ext(&schema_name, &mut ext);
    Ok(SpiBsInst { client: Box::new(client), ext })
}

pub async fn init_table_and_conn(
    bs_inst: (&TardisRelDBClient, &HashMap<String, String>, String),
    tag: &str,
    ctx: &TardisContext,
    mgr: bool,
) -> TardisResult<TardisRelDBlConnection> {
    let mut conn = bs_inst.0.conn();
    let mut schema_name = "".to_string();
    if let Some(_schema_name) = spi_initializer::common_pg::get_schema_name_from_ext(bs_inst.1) {
        schema_name = _schema_name;
        spi_initializer::common_pg::set_schema_to_session(&schema_name, &mut conn).await?;
    }
    if spi_initializer::common_pg::check_table_exit(&format!("starsys_log_{}", tag), &conn, ctx).await? {
        return Ok(conn);
    } else if !mgr {
        return Err(TardisError::bad_request("The requested tag does not exist", ""));
    }
    conn.execute_one(
        &format!(
            r#"CREATE TABLE {}.starsys_log_{}
(
    ts timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    key character varying NOT NULL,
    op character varying NOT NULL,
    content text NOT NULL,
    rel_key character varying NOT NULL
)"#,
            schema_name, tag
        ),
        vec![],
    )
    .await?;
    conn.execute_one(
        &format!("CREATE INDEX idx_{}_{}_ts ON {}.starsys_log_{} USING btree(ts)", schema_name, tag, schema_name, tag),
        vec![],
    )
    .await?;
    conn.execute_one(
        &format!("CREATE INDEX idx_{}_{}_key ON {}.starsys_log_{} USING btree(key)", schema_name, tag, schema_name, tag),
        vec![],
    )
    .await?;
    conn.execute_one(
        &format!("CREATE INDEX idx_{}_{}_op ON {}.starsys_log_{} USING btree(op)", schema_name, tag, schema_name, tag),
        vec![],
    )
    .await?;
    conn.execute_one(
        &format!(
            "CREATE INDEX idx_{}_{}_rel_key ON {}.starsys_log_{} USING btree(rel_key)",
            schema_name, tag, schema_name, tag
        ),
        vec![],
    )
    .await?;
    Ok(conn)
}
