use bios_basic::spi::{
    spi_funs::SpiBsInstExtractor,
    spi_initializer::common_pg::{self, package_table_name},
};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    chrono::{DateTime, Utc},
    db::{
        reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
        sea_orm::{FromQueryResult, Value},
    },
    serde_json::{self},
    web::web_resp::TardisPage,
    TardisFunsInst,
};

use crate::{
    dto::stats_record_dto::{StatsDimRecordAddReq, StatsFactRecordLoadReq, StatsFactRecordsLoadReq},
    stats_enumeration::StatsFactColKind,
};

use super::{stats_pg_conf_dim_serv, stats_pg_conf_fact_col_serv, stats_pg_conf_fact_serv};

pub(crate) async fn fact_record_load(fact_conf_key: &str, fact_record_key: &str, add_req: StatsFactRecordLoadReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, _) = common_pg::init_conn(bs_inst).await?;
    conn.begin().await?;
    if !stats_pg_conf_fact_serv::online(fact_conf_key, &conn, ctx).await? {
        return Err(funs.err().conflict("fact_record", "load", "The fact config not online.", "409-spi-stats-fact-conf-not-online"));
    }

    let fact_col_conf_set = stats_pg_conf_fact_col_serv::find_by_fact_conf_key(fact_conf_key, &conn, ctx).await?;

    let mut fields = vec!["key".to_string(), "own_paths".to_string(), "ct".to_string()];
    let mut values = vec![Value::from(fact_record_key), Value::from(add_req.own_paths), Value::from(add_req.ct)];
    let req_data = add_req.data.as_object().unwrap();

    for (req_fact_col_key, req_fact_col_value) in req_data {
        let fact_col_conf = fact_col_conf_set.iter().find(|c| &c.key == req_fact_col_key).ok_or_else(|| {
            funs.err().not_found(
                "fact_record",
                "load",
                &format!("The fact column config [{req_fact_col_key}] not exists."),
                "404-spi-stats-fact-col-conf-not-exist",
            )
        })?;
        if fact_col_conf.kind == StatsFactColKind::Dimension {
            let dim_conf = stats_pg_conf_dim_serv::get(fact_col_conf.dim_rel_conf_dim_key.as_ref().unwrap(), &conn, ctx).await?.unwrap();
            // TODO check value enum when stable_ds =true
            fields.push(req_fact_col_key.to_string());
            if fact_col_conf.dim_multi_values.unwrap_or(false) {
                values.push(dim_conf.data_type.json_to_sea_orm_value_array(req_fact_col_value, false));
            } else {
                values.push(dim_conf.data_type.json_to_sea_orm_value(req_fact_col_value, false));
            }
        } else if fact_col_conf.kind == StatsFactColKind::Measure {
            fields.push(req_fact_col_key.to_string());
            values.push(fact_col_conf.mes_data_type.as_ref().unwrap().json_to_sea_orm_value(req_fact_col_value, false));
        } else {
            fields.push(req_fact_col_key.to_string());
            values.push(Value::from(req_fact_col_value.as_str().unwrap()));
        }
        // TODO check data type
    }
    if fact_col_conf_set.len() != req_data.len() {
        let latest_data = fact_get_latest_record_raw(fact_conf_key, fact_record_key, &conn, ctx).await?.ok_or_else(|| {
            funs.err().not_found(
                "fact_record",
                "load",
                &format!("The fact latest instance record [{}][{}] not exists.", &fact_conf_key, &fact_record_key),
                "404-spi-stats-fact-inst-record-not-exist",
            )
        })?;
        for fact_col_conf in fact_col_conf_set {
            if !req_data.contains_key(&fact_col_conf.key) {
                fields.push(fact_col_conf.key.to_string());
                if fact_col_conf.kind == StatsFactColKind::Dimension {
                    let dim_conf = stats_pg_conf_dim_serv::get(&fact_col_conf.dim_rel_conf_dim_key.unwrap(), &conn, ctx).await?.unwrap();
                    if fact_col_conf.dim_multi_values.unwrap_or(false) {
                        values.push(dim_conf.data_type.result_to_sea_orm_value_array(&latest_data, &fact_col_conf.key));
                    } else {
                        values.push(dim_conf.data_type.result_to_sea_orm_value(&latest_data, &fact_col_conf.key));
                    }
                } else if fact_col_conf.kind == StatsFactColKind::Measure {
                    values.push(fact_col_conf.mes_data_type.unwrap().result_to_sea_orm_value(&latest_data, &fact_col_conf.key));
                } else {
                    values.push(Value::from(latest_data.try_get::<String>("", &fact_col_conf.key).unwrap()));
                }
            }
        }
    }

    let table_name = package_table_name(&format!("stats_inst_fact_{fact_conf_key}"), ctx);
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

pub(crate) async fn fact_records_load(fact_conf_key: &str, add_req_set: Vec<StatsFactRecordsLoadReq>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, _) = common_pg::init_conn(bs_inst).await?;
    conn.begin().await?;
    if !stats_pg_conf_fact_serv::online(fact_conf_key, &conn, ctx).await? {
        return Err(funs.err().conflict("fact_record", "load_set", "The fact config not online.", "409-spi-stats-fact-conf-not-online"));
    }

    let fact_col_conf_set = stats_pg_conf_fact_col_serv::find_by_fact_conf_key(fact_conf_key, &conn, ctx).await?;

    let mut has_fields_init = false;
    let mut fields = vec!["key".to_string(), "own_paths".to_string(), "ct".to_string()];
    let mut value_sets = vec![];

    for add_req in add_req_set {
        let req_data = add_req.data.as_object().unwrap();
        let mut values = vec![Value::from(&add_req.key), Value::from(add_req.own_paths), Value::from(add_req.ct)];

        for fact_col_conf in &fact_col_conf_set {
            if !has_fields_init {
                fields.push(fact_col_conf.key.to_string());
            }

            let req_fact_col_value = req_data.get(&fact_col_conf.key).ok_or_else(|| {
                funs.err().bad_request(
                    "fact_record",
                    "load_set",
                    &format!(
                        "The fact instance record [{}][{}] is missing a required column [{}].",
                        fact_conf_key, add_req.key, fact_col_conf.key
                    ),
                    "400-spi-stats-fact-inst-record-missing-column",
                )
            })?;
            if fact_col_conf.kind == StatsFactColKind::Dimension {
                let dim_conf = stats_pg_conf_dim_serv::get(fact_col_conf.dim_rel_conf_dim_key.as_ref().unwrap(), &conn, ctx).await?.unwrap();
                // TODO check value enum when stable_ds =true
                if fact_col_conf.dim_multi_values.unwrap_or(false) {
                    values.push(dim_conf.data_type.json_to_sea_orm_value_array(req_fact_col_value, false));
                } else {
                    values.push(dim_conf.data_type.json_to_sea_orm_value(req_fact_col_value, false));
                }
            } else if fact_col_conf.kind == StatsFactColKind::Measure {
                values.push(fact_col_conf.mes_data_type.as_ref().unwrap().json_to_sea_orm_value(req_fact_col_value, false));
            } else {
                values.push(Value::from(req_fact_col_value.as_str().unwrap()));
            }
            // TODO check data type
        }
        value_sets.push(values);
        has_fields_init = true;
    }

    let table_name = package_table_name(&format!("stats_inst_fact_{fact_conf_key}"), ctx);
    let fields_placeholders = fields.iter().enumerate().map(|(i, _)| format!("${}", i + 1)).collect::<Vec<String>>().join(",");
    let fields = fields.join(",");
    for values in value_sets {
        conn.execute_one(
            &format!(
                r#"INSERT INTO {table_name}
    ({fields})
    VALUES
    ({fields_placeholders})
    "#,
            ),
            values,
        )
        .await?;
    }
    conn.commit().await?;
    Ok(())
}

pub(crate) async fn fact_record_delete(fact_conf_key: &str, fact_record_key: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, _) = common_pg::init_conn(bs_inst).await?;
    conn.begin().await?;
    if !stats_pg_conf_fact_serv::online(fact_conf_key, &conn, ctx).await? {
        return Err(funs.err().conflict("fact_record", "delete", "The fact config not online.", "409-spi-stats-fact-conf-not-online"));
    }

    let table_name = package_table_name(&format!("stats_inst_fact_{fact_conf_key}_del"), ctx);
    conn.execute_one(
        &format!(
            r#"INSERT INTO {table_name}
(key)
VALUES
($1)
"#,
        ),
        vec![Value::from(fact_record_key)],
    )
    .await?;
    conn.commit().await?;
    Ok(())
}

pub(crate) async fn fact_records_delete(fact_conf_key: &str, fact_record_delete_keys: &[String], funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, _) = common_pg::init_conn(bs_inst).await?;
    conn.begin().await?;
    if !stats_pg_conf_fact_serv::online(fact_conf_key, &conn, ctx).await? {
        return Err(funs.err().conflict("fact_record", "delete_set", "The fact config not online.", "409-spi-stats-fact-conf-not-online"));
    }

    let table_name = package_table_name(&format!("stats_inst_fact_{fact_conf_key}_del"), ctx);
    for delete_key in fact_record_delete_keys {
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

pub(crate) async fn fact_records_clean(fact_conf_key: &str, before_ct: Option<DateTime<Utc>>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, _) = common_pg::init_conn(bs_inst).await?;
    conn.begin().await?;
    if !stats_pg_conf_fact_serv::online(fact_conf_key, &conn, ctx).await? {
        return Err(funs.err().conflict("fact_record", "clean", "The fact config not online.", "409-spi-stats-fact-conf-not-online"));
    }

    let table_name = package_table_name(&format!("stats_inst_fact_{fact_conf_key}"), ctx);
    if let Some(before_ct) = before_ct {
        conn.execute_one(&format!("DELETE FROM {table_name} WHERE ct <= $1"), vec![Value::from(before_ct)]).await?;
    } else {
        conn.execute_one(&format!("DELETE FROM {table_name}"), vec![]).await?;
    }
    conn.commit().await?;
    Ok(())
}

pub(crate) async fn dim_record_add(dim_conf_key: String, add_req: StatsDimRecordAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, _) = common_pg::init_conn(bs_inst).await?;
    conn.begin().await?;
    if !stats_pg_conf_dim_serv::online(&dim_conf_key, &conn, ctx).await? {
        return Err(funs.err().conflict("dim_record", "add", "The dimension config not online.", "409-spi-stats-dim-conf-not-online"));
    }
    let dim_conf = stats_pg_conf_dim_serv::get(&dim_conf_key, &conn, ctx).await?.unwrap();
    if !dim_conf.stable_ds {
        return Err(funs.err().bad_request(
            "dim_record",
            "add",
            &format!("The dimension config [{}] stable_ds is false, so adding dimension records is not supported.", &dim_conf_key),
            "400-spi-stats-dim-conf-stable-ds-false",
        ));
    }
    if dim_conf.hierarchy.is_empty() && add_req.parent_key.is_some() {
        return Err(funs.err().bad_request(
            "dim_record",
            "add",
            &format!("The dimension config [{}] not allow hierarchy.", &dim_conf_key),
            "400-spi-stats-dim-conf-not-hierarchy",
        ));
    }

    let table_name = package_table_name(&format!("stats_inst_dim_{}", dim_conf.key), ctx);
    let dim_record_key_value = dim_conf.data_type.json_to_sea_orm_value(&add_req.key, false);
    if conn.count_by_sql(&format!("SELECT 1 FROM {table_name} WHERE key = $1"), vec![dim_record_key_value.clone()]).await? != 0 {
        return Err(funs.err().conflict(
            "dim_record",
            "add",
            "The dimension instance record already exists, please delete it and then add it.",
            "409-spi-stats-dim-inst-record-exist",
        ));
    }
    let mut sql_fields = vec![];
    let mut params = vec![dim_record_key_value.clone(), Value::from(add_req.show_name.clone())];

    if let Some(parent_key) = add_req.parent_key {
        let parent_record = dim_record_get(&dim_conf_key, parent_key.clone(), &conn, funs, ctx).await?.ok_or_else(|| {
            funs.err().not_found(
                "dim_record",
                "add",
                &format!("The parent dimension instance record [{parent_key}] not exists."),
                "404-spi-stats-dim-inst-record-not-exist",
            )
        })?;
        let parent_hierarchy = parent_record.get("hierarchy").unwrap().as_i64().unwrap();
        if (parent_hierarchy + 1) as usize >= dim_conf.hierarchy.len() {
            return Err(funs.err().conflict(
                "dim_record",
                "add",
                "The dimension instance record hierarchy is too deep.",
                "409-spi-stats-dim-inst-record-hierarchy-too-deep",
            ));
        }
        params.push(Value::from(parent_hierarchy + 1));
        sql_fields.push("hierarchy".to_string());
        params.push(dim_record_key_value);
        sql_fields.push(format!("key{}", parent_hierarchy + 1));
        for i in 0..parent_hierarchy + 1 {
            params.push(Value::from(parent_record.get(&format!("key{i}")).unwrap().as_str().unwrap()));
            sql_fields.push(format!("key{i}"));
        }
    } else if dim_conf.hierarchy.len() > 1 {
        params.push(Value::from(0));
        sql_fields.push("hierarchy".to_string());
        params.push(dim_record_key_value);
        sql_fields.push("key0".to_string());
    }
    conn.execute_one(
        &format!(
            r#"INSERT INTO {table_name}
(key, show_name {})
VALUES
($1, $2 {})
"#,
            if sql_fields.is_empty() { "".to_string() } else { format!(",{}", sql_fields.join(",")) },
            if sql_fields.is_empty() {
                "".to_string()
            } else {
                format!(",{}", sql_fields.iter().enumerate().map(|(i, _)| format!("${}", i + 3)).collect::<Vec<String>>().join(","))
            }
        ),
        params,
    )
    .await?;
    conn.commit().await?;
    Ok(())
}

pub(in crate::serv::pg) async fn dim_record_get(
    dim_conf_key: &str,
    dim_record_key: serde_json::Value,
    conn: &TardisRelDBlConnection,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<Option<serde_json::Value>> {
    dim_do_record_paginate(dim_conf_key.to_string(), Some(dim_record_key), None, 1, 1, None, None, conn, funs, ctx).await.map(|page| page.records.into_iter().next())
}

pub(crate) async fn dim_record_paginate(
    dim_conf_key: String,
    dim_record_key: Option<serde_json::Value>,
    show_name: Option<String>,
    page_number: u32,
    page_size: u32,
    desc_by_create: Option<bool>,
    desc_by_update: Option<bool>,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<TardisPage<serde_json::Value>> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (conn, _) = common_pg::init_conn(bs_inst).await?;
    dim_do_record_paginate(
        dim_conf_key,
        dim_record_key,
        show_name,
        page_number,
        page_size,
        desc_by_create,
        desc_by_update,
        &conn,
        funs,
        ctx,
    )
    .await
}

async fn dim_do_record_paginate(
    dim_conf_key: String,
    dim_record_key: Option<serde_json::Value>,
    show_name: Option<String>,
    page_number: u32,
    page_size: u32,
    desc_by_create: Option<bool>,
    desc_by_update: Option<bool>,
    conn: &TardisRelDBlConnection,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<TardisPage<serde_json::Value>> {
    let dim_conf = stats_pg_conf_dim_serv::get(&dim_conf_key, conn, ctx)
        .await?
        .ok_or_else(|| funs.err().not_found("dim_record", "find", "The dimension config does not exist.", "404-spi-stats-dim-conf-not-exist"))?;

    let table_name = package_table_name(&format!("stats_inst_dim_{dim_conf_key}"), ctx);
    let mut sql_where = vec!["1 = 1".to_string()];
    let mut sql_order = vec![];
    let mut params: Vec<Value> = vec![Value::from(page_size), Value::from((page_number - 1) * page_size)];
    if let Some(dim_record_key) = &dim_record_key {
        sql_where.push(format!("key = ${}", params.len() + 1));
        params.push(dim_conf.data_type.json_to_sea_orm_value(dim_record_key, false));
    }
    if let Some(show_name) = &show_name {
        sql_where.push(format!("show_name LIKE ${}", params.len() + 1));
        params.push(Value::from(format!("%{show_name}%")));
    }
    if let Some(desc_by_create) = desc_by_create {
        sql_order.push(format!("create_time {}", if desc_by_create { "DESC" } else { "ASC" }));
    }
    if let Some(desc_by_update) = desc_by_update {
        sql_order.push(format!("update_time {}", if desc_by_update { "DESC" } else { "ASC" }));
    }

    let result = conn
        .query_all(
            &format!(
                r#"SELECT *, count(*) OVER() AS total
FROM {table_name}
WHERE 
    {}
LIMIT $1 OFFSET $2
{}"#,
                sql_where.join(" AND "),
                if sql_order.is_empty() {
                    "".to_string()
                } else {
                    format!("ORDER BY {}", sql_order.join(","))
                }
            ),
            params,
        )
        .await?;
    let mut total_size: i64 = 0;
    let mut final_result = vec![];
    for item in result {
        if total_size == 0 {
            total_size = item.try_get("", "total").unwrap();
        }
        let values = serde_json::Value::from_query_result_optional(&item, "")?.unwrap();
        final_result.push(values);
    }
    Ok(TardisPage {
        page_size: page_size as u64,
        page_number: page_number as u64,
        total_size: total_size as u64,
        records: final_result,
    })
}

pub(crate) async fn dim_record_delete(dim_conf_key: String, dim_record_key: serde_json::Value, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, _) = common_pg::init_conn(bs_inst).await?;
    conn.begin().await?;
    if !stats_pg_conf_dim_serv::online(&dim_conf_key, &conn, ctx).await? {
        return Err(funs.err().conflict("dim_record", "delete", "The dimension config not online.", "409-spi-stats-dim-conf-not-online"));
    }
    let dim_conf = stats_pg_conf_dim_serv::get(&dim_conf_key, &conn, ctx).await?.unwrap();

    let table_name = package_table_name(&format!("stats_inst_dim_{}", dim_conf.key), ctx);
    let values = vec![dim_conf.data_type.json_to_sea_orm_value(&dim_record_key, false)];

    conn.execute_one(
        &format!(
            r#"UPDATE {table_name}
SET et = now()
WHERE key = $1
"#,
        ),
        values,
    )
    .await?;
    conn.commit().await?;
    Ok(())
}

async fn fact_get_latest_record_raw(
    fact_conf_key: &str,
    dim_record_key: &str,
    conn: &TardisRelDBlConnection,
    ctx: &TardisContext,
) -> TardisResult<Option<tardis::db::sea_orm::QueryResult>> {
    let table_name = package_table_name(&format!("stats_inst_fact_{fact_conf_key}"), ctx);
    let result = conn.query_one(&format!("SELECT * FROM {table_name} WHERE key = $1"), vec![Value::from(dim_record_key)]).await?;
    Ok(result)
}
