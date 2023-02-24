use bios_basic::spi::{spi_funs::SpiBsInstExtractor, spi_initializer::common_pg};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    chrono::{DateTime, Utc},
    db::{
        reldb_client::TardisRelDBClient,
        sea_orm::{QueryResult, Value},
    },
    TardisFunsInst,
};

use crate::dto::{stats_conf_dto::StatsConfDimInfoResp, stats_record_dto::StatsDimRecordAddReq};

pub(crate) async fn fact_load_record(fact_key: &str, fields: Vec<String>, values: Vec<Value>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, schema_name) = common_pg::init_conn(bs_inst).await?;
    conn.begin().await?;
    let table_name = format!("{schema_name}.starsys_stats_inst_fact_{fact_key}");
    conn.execute_one(
        &format!(
            r#"INSERT INTO {table_name}
({})
VALUES
({})
"#,
            fields.join(","),
            fields.iter().enumerate().map(|(i, _)| format!("${}", i + 1)).collect::<Vec<String>>().join(",")
        ),
        values,
    )
    .await?;
    conn.commit().await?;
    Ok(())
}

pub(crate) async fn fact_load_records(fact_key: &str, fields: Vec<String>, value_sets: Vec<Vec<Value>>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, schema_name) = common_pg::init_conn(bs_inst).await?;
    conn.begin().await?;
    let table_name = format!("{schema_name}.starsys_stats_inst_fact_{fact_key}");
    let columns = fields.join(",");
    let column_placeholders = fields.iter().enumerate().map(|(i, _)| format!("${}", i + 1)).collect::<Vec<String>>().join(",");
    for values in value_sets {
        conn.execute_one(
            &format!(
                r#"INSERT INTO {table_name}
    ({columns})
    VALUES
    ({column_placeholders})
    "#,
            ),
            values,
        )
        .await?;
    }
    conn.commit().await?;
    Ok(())
}

pub(crate) async fn fact_delete_record(fact_key: &str, record_key: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, schema_name) = common_pg::init_conn(bs_inst).await?;
    conn.begin().await?;
    let table_name = format!("{schema_name}.starsys_stats_inst_fact_{fact_key}_del");
    conn.execute_one(
        &format!(
            r#"INSERT INTO {table_name}
(key)
VALUES
($1)
"#,
        ),
        vec![Value::from(record_key)],
    )
    .await?;
    conn.commit().await?;
    Ok(())
}

pub(crate) async fn fact_delete_records(fact_key: &str, delete_keys: &[String], funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, schema_name) = common_pg::init_conn(bs_inst).await?;
    conn.begin().await?;
    let table_name = format!("{schema_name}.starsys_stats_inst_fact_{fact_key}_del");
    for delete_key in delete_keys {
        conn.execute_one(
            &format!(
                r#"INSERT INTO {table_name}
    (key)
    VALUES
    ($1)
    "#,
            ),
            vec![Value::from(delete_key)],
        )
        .await?;
    }
    conn.commit().await?;
    Ok(())
}

pub(crate) async fn fact_get_latest_record_raw(fact_key: &str, record_key: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<QueryResult>> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (conn, schema_name) = common_pg::init_conn(bs_inst).await?;
    let table_name = format!("{schema_name}.starsys_stats_inst_fact_{fact_key}");
    let result = conn.query_one(&format!("SELECT * {table_name} WHERE key = $1"), vec![Value::from(record_key)]).await?;
    Ok(result)
}

pub(crate) async fn fact_clean_records(fact_key: &str, before_ct: Option<DateTime<Utc>>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, schema_name) = common_pg::init_conn(bs_inst).await?;
    conn.begin().await?;
    let table_name = format!("{schema_name}.starsys_stats_inst_fact_{fact_key}");
    if let Some(before_ct) = before_ct {
        conn.execute_one(&format!("DELETE FROM {table_name} WHERE ct <= $1"), vec![Value::from(before_ct)]).await?;
    } else {
        conn.execute_one(&format!("DELETE FROM {table_name}"), vec![]).await?;
    }
    conn.commit().await?;
    Ok(())
}

pub(crate) async fn dim_add_record(
    record_key: String,
    add_req: StatsDimRecordAddReq,
    dim_conf: &StatsConfDimInfoResp,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, schema_name) = common_pg::init_conn(bs_inst).await?;
    conn.begin().await?;
    let table_name = format!("{schema_name}.starsys_stats_inst_dim_{}", dim_conf.key);
    if conn.query_one(&format!("SELECT 1 {table_name} WHERE key = $1"), vec![Value::from(&record_key)]).await?.is_some() {
        return Err(funs.err().conflict(
            "dim_record",
            "add",
            "The dimension instance record already exists, please delete it and then add it.",
            "409-spi-stats-dim-inst-record-exist",
        ));
    }
    let mut sql_fields = vec![];
    let mut params = vec![Value::from(&record_key), Value::from(add_req.show_name.clone()), Value::from(add_req.ct)];

    if let Some(parent_key) = add_req.parent_key {
        let parent_record = conn.query_one(&format!("SELECT 1 {table_name} WHERE key = $1"), vec![Value::from(&parent_key)]).await?.ok_or(funs.err().not_found(
            "dim_record",
            "add",
            &format!("The parent dimension instance record [{parent_key}] not exists."),
            "404-spi-stats-dim-inst-record-not-exist",
        ))?;
        let parent_hierarchy: i16 = parent_record.try_get("", "hierarchy").unwrap();
        if (parent_hierarchy + 1) as usize > dim_conf.hierarchy.len() {
            return Err(funs.err().conflict(
                "dim_record",
                "add",
                "The dimension instance record hierarchy is too deep.",
                "409-spi-stats-dim-inst-record-hierarchy-too-deep",
            ));
        }
        params.push(Value::from(parent_hierarchy + 1));
        sql_fields.push("hierarchy".to_string());
        params.push(Value::from(&record_key));
        sql_fields.push(format!("key{}", parent_hierarchy + 1));
        for i in 0..parent_hierarchy {
            params.push(Value::from(parent_record.try_get::<String>("", &format!("key{i}")).unwrap()));
            sql_fields.push(format!("key{i}"));
        }
    } else if dim_conf.hierarchy.len() > 1 {
        params.push(Value::from(0));
        sql_fields.push("hierarchy".to_string());
        params.push(Value::from(&record_key));
        sql_fields.push("key0".to_string());
    }
    conn.execute_one(
        &format!(
            r#"INSERT INTO {table_name}
(key, show_name, st {})
VALUES
($1, $2, $3 {})
"#,
            if sql_fields.is_empty() { "".to_string() } else { format!(",{}", sql_fields.join(",")) },
            if sql_fields.is_empty() {
                "".to_string()
            } else {
                format!(",{}", sql_fields.iter().enumerate().map(|(i, _)| format!("${}", i + 4)).collect::<Vec<String>>().join(","))
            }
        ),
        params,
    )
    .await?;
    conn.commit().await?;
    Ok(())
}

pub(crate) async fn dim_delete_record(record_key: String, dim_conf: &StatsConfDimInfoResp, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, schema_name) = common_pg::init_conn(bs_inst).await?;
    conn.begin().await?;
    let table_name = format!("{schema_name}.starsys_stats_inst_dim_{}", dim_conf.key);
    conn.execute_one(
        &format!(
            r#"UPDATE {table_name}
SET et = now()
WHERE key = $1
"#,
        ),
        vec![Value::from(&record_key)],
    )
    .await?;
    conn.commit().await?;
    Ok(())
}

pub(crate) async fn dim_get_inst_record_id(dim_conf_key: &str, record_key: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<i32>> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (conn, schema_name) = common_pg::init_conn(bs_inst).await?;
    let table_name = format!("{schema_name}.starsys_stats_inst_dim_{dim_conf_key}");
    let id = conn.query_one(&format!("SELECT id {table_name} WHERE key = $1"), vec![Value::from(record_key)]).await?.map(|r| r.try_get("", "id").unwrap());
    Ok(id)
}
