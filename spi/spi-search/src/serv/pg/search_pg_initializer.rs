use std::collections::HashMap;

use bios_basic::spi::{dto::spi_bs_dto::SpiBsCertResp, spi_funs::SpiBsInst, spi_initializer};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    db::reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
    TardisFuns,
};

pub async fn init(bs_cert: &SpiBsCertResp, ctx: &TardisContext) -> TardisResult<SpiBsInst> {
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
    } else {
        spi_initializer::common_pg::init_pg_schema(&client, ctx).await?
    };
    spi_initializer::common_pg::set_pg_schema_to_ext(&schema_name, &mut ext);
    Ok(SpiBsInst { client: Box::new(client), ext })
}

pub async fn init_table_and_conn(bs_inst: (&TardisRelDBClient, &HashMap<String, String>, String), tag: &str, ctx: &TardisContext) -> TardisResult<TardisRelDBlConnection> {
    let conn = bs_inst.0.conn();
    let mut schema_name = "".to_string();
    if let Some(_schema_name) = spi_initializer::common_pg::get_pg_schema_from_ext(bs_inst.1) {
        schema_name = _schema_name;
        spi_initializer::common_pg::set_pg_schema_to_session(&schema_name, &conn).await?;
    }
    if spi_initializer::common_pg::check_table_exit(&format!("starsys_search_{}", tag), &conn, ctx).await? {
        return Ok(conn);
    }
    conn.execute_one(
        &format!(
            r#"CREATE TABLE {}.starsys_search_{}
(
    key character varying NOT NULL PRIMARY KEY,
    title character varying NOT NULL,
    title_tsv tsvector,
    content_tsv tsvector,
    owner character varying NOT NULL,
    own_paths character varying NOT NULL,
    create_time timestamp without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    update_time timestamp without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ext jsonb NOT NULL,
    visit_keys character varying[]
)"#,
            schema_name, tag
        ),
        vec![],
    )
    .await?;
    conn.execute_one(&format!("CREATE INDEX idx_key ON {}.starsys_search_{} USING btree(key)", schema_name, tag), vec![]).await?;
    conn.execute_one(
        &format!("CREATE INDEX idx_title_tsv ON {}.starsys_search_{} USING gin(title_tsv)", schema_name, tag),
        vec![],
    )
    .await?;
    conn.execute_one(
        &format!("CREATE INDEX idx_content_tsv ON {}.starsys_search_{} USING gin(content_tsv)", schema_name, tag),
        vec![],
    )
    .await?;
    conn.execute_one(&format!("CREATE INDEX idx_owner ON {}.starsys_search_{} USING btree(owner)", schema_name, tag), vec![]).await?;
    conn.execute_one(
        &format!("CREATE INDEX idx_own_paths ON {}.starsys_search_{} USING btree(own_paths)", schema_name, tag),
        vec![],
    )
    .await?;
    conn.execute_one(
        &format!("CREATE INDEX idx_create_time ON {}.starsys_search_{} USING btree(create_time)", schema_name, tag),
        vec![],
    )
    .await?;
    conn.execute_one(
        &format!("CREATE INDEX idx_update_time ON {}.starsys_search_{} USING btree(update_time)", schema_name, tag),
        vec![],
    )
    .await?;
    conn.execute_one(&format!("CREATE INDEX idx_ext ON {}.starsys_search_{} USING gin(ext)", schema_name, tag), vec![]).await?;
    conn.execute_one(
        &format!("CREATE INDEX idx_visit_keys ON {}.starsys_search_{} USING btree(visit_keys)", schema_name, tag),
        vec![],
    )
    .await?;
    Ok(conn)
}
