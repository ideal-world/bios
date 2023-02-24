use bios_basic::spi::{spi_funs::SpiBsInstExtractor, spi_initializer::common_pg};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    chrono::{DateTime, NaiveDate, Utc},
    db::{
        reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
        sea_orm::{QueryResult, Value},
    },
    TardisFunsInst,
};

use crate::{
    dto::stats_record_dto::{StatsDimRecordAddReq, StatsFactRecordLoadReq, StatsFactRecordsLoadReq},
    stats_enumeration::{StatsDataTypeKind, StatsFactColKind},
};

use super::{stats_pg_conf_dim_serv, stats_pg_conf_fact_col_serv, stats_pg_conf_fact_serv};

pub(crate) async fn fact_load_record(fact_key: &str, record_key: &str, add_req: StatsFactRecordLoadReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, schema_name) = common_pg::init_conn(bs_inst).await?;
    conn.begin().await?;
    if !stats_pg_conf_fact_serv::online(fact_key, &conn, ctx).await? {
        return Err(funs.err().conflict("fact_record", "load", "The fact config not online.", "409-spi-stats-fact-conf-not-online"));
    }

    let fact_col_conf_set = stats_pg_conf_fact_col_serv::find_by_fact_key(fact_key, &conn, ctx).await?;

    let mut fields = vec!["key".to_string(), "own_paths".to_string(), "st".to_string()];
    let mut values = vec![Value::from(record_key), Value::from(add_req.own_paths), Value::from(add_req.ct)];
    let req_data = add_req.data.as_object().unwrap();

    for (req_fact_col_key, req_fact_col_value) in req_data {
        let fact_col_conf = fact_col_conf_set.iter().find(|c| &c.key == req_fact_col_key).ok_or(funs.err().not_found(
            "fact_record",
            "load",
            &format!("The fact column config [{req_fact_col_key}] not exists."),
            "404-spi-stats-fact-col-conf-not-exist",
        ))?;

        if fact_col_conf.kind == StatsFactColKind::Dimension
            && stats_pg_conf_dim_serv::get(fact_col_conf.dim_rel_conf_dim_key.as_ref().unwrap(), &conn, ctx).await?.unwrap().stable_ds
        {
            let dim_record_id = dim_get_inst_record_id(
                fact_col_conf.dim_rel_conf_dim_key.as_ref().unwrap(),
                req_fact_col_value.as_str().unwrap(),
                &conn,
                &schema_name,
            )
            .await?
            .ok_or(funs.err().not_found(
                "fact_record",
                "load",
                &format!(
                    "The parent dimension instance record [{}] not exists.",
                    &fact_col_conf.dim_rel_conf_dim_key.as_ref().unwrap()
                ),
                "404-spi-stats-dim-inst-record-not-exist",
            ))?;
            // Replace dimension instance record id to dimension instance record key
            fields.push(req_fact_col_key.to_string());
            values.push(Value::from(dim_record_id));
        } else if fact_col_conf.kind == StatsFactColKind::Dimension {
            fields.push(req_fact_col_key.to_string());
            if fact_col_conf.dim_multi_values.unwrap_or(false) {
                values.push(Value::from(
                    req_fact_col_value.as_array().unwrap().iter().map(|v| v.as_str().unwrap().to_string()).collect::<Vec<String>>(),
                ));
            } else {
                values.push(Value::from(req_fact_col_value.as_str().unwrap()));
            }
        } else {
            fields.push(req_fact_col_key.to_string());
            match fact_col_conf.mes_data_type {
                Some(StatsDataTypeKind::Number) => values.push(Value::from(req_fact_col_value.as_i64().unwrap())),
                Some(StatsDataTypeKind::Boolean) => values.push(Value::from(req_fact_col_value.as_bool().unwrap())),
                Some(StatsDataTypeKind::DateTime) => values.push(Value::from(req_fact_col_value.as_str().unwrap())),
                Some(StatsDataTypeKind::Date) => values.push(Value::from(req_fact_col_value.as_str().unwrap())),
                _ => values.push(Value::from(req_fact_col_value.as_str().unwrap())),
            }
        }
        // TODO check data type
    }
    if fact_col_conf_set.len() != req_data.len() {
        let latest_data = fact_get_latest_record_raw(&fact_key, &record_key, &conn, &schema_name).await?.ok_or(funs.err().not_found(
            "fact_record",
            "load",
            &format!("The fact latest instance record [{}][{}] not exists.", &fact_key, &record_key),
            "404-spi-stats-fact-inst-record-not-exist",
        ))?;
        for fact_col_conf in fact_col_conf_set {
            if !req_data.contains_key(&fact_col_conf.key) {
                fields.push(fact_col_conf.key.to_string());
                match fact_col_conf.mes_data_type {
                    Some(StatsDataTypeKind::Number) => values.push(Value::from(latest_data.try_get::<i32>("", &fact_col_conf.key).unwrap())),
                    Some(StatsDataTypeKind::Boolean) => values.push(Value::from(latest_data.try_get::<bool>("", &fact_col_conf.key).unwrap())),
                    Some(StatsDataTypeKind::DateTime) => values.push(Value::from(latest_data.try_get::<DateTime<Utc>>("", &fact_col_conf.key).unwrap())),
                    Some(StatsDataTypeKind::Date) => values.push(Value::from(latest_data.try_get::<NaiveDate>("", &fact_col_conf.key).unwrap())),
                    _ => values.push(Value::from(latest_data.try_get::<String>("", &fact_col_conf.key).unwrap())),
                }
            }
        }
    }

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

pub(crate) async fn fact_load_records(fact_key: &str, add_req_set: Vec<StatsFactRecordsLoadReq>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, schema_name) = common_pg::init_conn(bs_inst).await?;
    conn.begin().await?;
    if !stats_pg_conf_fact_serv::online(fact_key, &conn, ctx).await? {
        return Err(funs.err().conflict("fact_record", "load_et", "The fact config not online.", "409-spi-stats-fact-conf-not-online"));
    }

    let fact_col_conf_set = stats_pg_conf_fact_col_serv::find_by_fact_key(fact_key, &conn, ctx).await?;

    let mut has_fields_init = false;
    let mut fields = vec!["key".to_string(), "own_paths".to_string(), "st".to_string()];
    let mut value_sets = vec![];

    for add_req in add_req_set {
        let req_data = add_req.data.as_object().unwrap();
        let mut values = vec![Value::from(&add_req.key), Value::from(add_req.own_paths), Value::from(add_req.ct)];

        for fact_col_conf in &fact_col_conf_set {
            let req_fact_col_value = req_data.get(&fact_col_conf.key).ok_or(funs.err().bad_request(
                "fact_record",
                "load_set",
                &format!(
                    "The fact instance record [{}][{}] is missing a required column [{}].",
                    fact_key, add_req.key, fact_col_conf.key
                ),
                "400-spi-stats-fact-inst-record-missing-column",
            ))?;

            if !has_fields_init {
                fields.push(fact_col_conf.key.to_string());
            }

            if fact_col_conf.kind == StatsFactColKind::Dimension
                && stats_pg_conf_dim_serv::get(fact_col_conf.dim_rel_conf_dim_key.as_ref().unwrap(), &conn, ctx).await?.unwrap().stable_ds
            {
                let dim_record_id = dim_get_inst_record_id(
                    fact_col_conf.dim_rel_conf_dim_key.as_ref().unwrap(),
                    req_fact_col_value.as_str().unwrap(),
                    &conn,
                    &schema_name,
                )
                .await?
                .ok_or(funs.err().not_found(
                    "fact_record",
                    "load_set",
                    &format!(
                        "The parent dimension instance record [{}] not exists.",
                        &fact_col_conf.dim_rel_conf_dim_key.as_ref().unwrap()
                    ),
                    "404-spi-stats-dim-inst-record-not-exist",
                ))?;
                // Replace dimension instance record id to dimension instance record key
                if !has_fields_init {
                    fields.push(fact_col_conf.key.to_string());
                }
                values.push(Value::from(dim_record_id));
            } else if fact_col_conf.kind == StatsFactColKind::Dimension {
                if !has_fields_init {
                    fields.push(fact_col_conf.key.to_string());
                }
                if fact_col_conf.dim_multi_values.unwrap_or(false) {
                    values.push(Value::from(
                        req_fact_col_value.as_array().unwrap().iter().map(|v| v.as_str().unwrap().to_string()).collect::<Vec<String>>(),
                    ));
                } else {
                    values.push(Value::from(req_fact_col_value.as_str().unwrap()));
                }
            } else {
                if !has_fields_init {
                    fields.push(fact_col_conf.key.to_string());
                }
                match fact_col_conf.mes_data_type {
                    Some(StatsDataTypeKind::Number) => values.push(Value::from(req_fact_col_value.as_i64().unwrap())),
                    Some(StatsDataTypeKind::Boolean) => values.push(Value::from(req_fact_col_value.as_bool().unwrap())),
                    Some(StatsDataTypeKind::DateTime) => values.push(Value::from(req_fact_col_value.as_str().unwrap())),
                    Some(StatsDataTypeKind::Date) => values.push(Value::from(req_fact_col_value.as_str().unwrap())),
                    _ => values.push(Value::from(req_fact_col_value.as_str().unwrap())),
                }
            }
            // TODO check data type
        }
        value_sets.push(values);
        has_fields_init = true;
    }

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
    if !stats_pg_conf_fact_serv::online(fact_key, &conn, ctx).await? {
        return Err(funs.err().conflict("fact_record", "delete", "The fact config not online.", "409-spi-stats-fact-conf-not-online"));
    }

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
    if !stats_pg_conf_fact_serv::online(fact_key, &conn, ctx).await? {
        return Err(funs.err().conflict("fact_record", "delete_set", "The fact config not online.", "409-spi-stats-fact-conf-not-online"));
    }

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

pub(crate) async fn fact_clean_records(fact_key: &str, before_ct: Option<DateTime<Utc>>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, schema_name) = common_pg::init_conn(bs_inst).await?;
    conn.begin().await?;
    if !stats_pg_conf_fact_serv::online(fact_key, &conn, ctx).await? {
        return Err(funs.err().conflict("fact_record", "clean", "The fact config not online.", "409-spi-stats-fact-conf-not-online"));
    }

    let table_name = format!("{schema_name}.starsys_stats_inst_fact_{fact_key}");
    if let Some(before_ct) = before_ct {
        conn.execute_one(&format!("DELETE FROM {table_name} WHERE ct <= $1"), vec![Value::from(before_ct)]).await?;
    } else {
        conn.execute_one(&format!("DELETE FROM {table_name}"), vec![]).await?;
    }
    conn.commit().await?;
    Ok(())
}

pub(crate) async fn dim_add_record(dim_conf_key: String, record_key: String, add_req: StatsDimRecordAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, schema_name) = common_pg::init_conn(bs_inst).await?;
    conn.begin().await?;
    if !stats_pg_conf_dim_serv::online(&dim_conf_key, &conn, ctx).await? {
        return Err(funs.err().conflict("dim_record", "add", "The dim config not online.", "409-spi-stats-dim-conf-not-online"));
    }
    let dim_conf = stats_pg_conf_dim_serv::get(&dim_conf_key, &conn, ctx).await?.unwrap();
    if dim_conf.hierarchy.is_empty() && add_req.parent_key.is_some() {
        return Err(funs.err().bad_request(
            "dim_record",
            "add",
            &format!("The dimension config [{}] not allow hierarchy.", &dim_conf_key),
            "400-spi-stats-dim-conf-not-hierarchy",
        ));
    }

    let table_name = format!("{schema_name}.starsys_stats_inst_dim_{}", dim_conf.key);
    if conn.query_one(&format!("SELECT 1 FROM {table_name} WHERE key = $1"), vec![Value::from(&record_key)]).await?.is_some() {
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
        let parent_record = conn.query_one(&format!("SELECT 1 FROM {table_name} WHERE key = $1"), vec![Value::from(&parent_key)]).await?.ok_or(funs.err().not_found(
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

pub(crate) async fn dim_delete_record(dim_conf_key: String, record_key: String, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, schema_name) = common_pg::init_conn(bs_inst).await?;
    conn.begin().await?;
    if !stats_pg_conf_dim_serv::online(&dim_conf_key, &conn, ctx).await? {
        return Err(funs.err().conflict("dim_record", "delete", "The dim config not online.", "409-spi-stats-dim-conf-not-online"));
    }
    let dim_conf = stats_pg_conf_dim_serv::get(&dim_conf_key, &conn, ctx).await?.unwrap();

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

async fn fact_get_latest_record_raw(fact_key: &str, record_key: &str, conn: &TardisRelDBlConnection, schema_name: &str) -> TardisResult<Option<QueryResult>> {
    let table_name = format!("{schema_name}.starsys_stats_inst_fact_{fact_key}");
    let result = conn.query_one(&format!("SELECT * FROM　{table_name} WHERE key = $1"), vec![Value::from(record_key)]).await?;
    Ok(result)
}

async fn dim_get_inst_record_id(dim_conf_key: &str, record_key: &str, conn: &TardisRelDBlConnection, schema_name: &str) -> TardisResult<Option<i32>> {
    let table_name = format!("{schema_name}.starsys_stats_inst_dim_{dim_conf_key}");
    let id = conn.query_one(&format!("SELECT id　FROM {table_name} WHERE key = $1"), vec![Value::from(record_key)]).await?.map(|r| r.try_get("", "id").unwrap());
    Ok(id)
}
