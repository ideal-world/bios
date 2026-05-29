use bios_basic::spi::{
    spi_funs::SpiBsInst,
    spi_initializer::common_pg::{self, package_table_name},
};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    chrono::{DateTime, NaiveDate, Utc},
    db::{
        reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
        sea_orm::{FromQueryResult, Value},
    },
    serde_json,
    web::web_resp::TardisPage,
    TardisFunsInst,
};

use crate::{
    dto::stats_conf_dto::{StatsConfDimColAddReq, StatsConfDimColInfoResp, StatsConfDimColModifyReq},
    serv::{stats_cert_serv, stats_valid_serv},
};

use super::stats_pg_initializer;

pub(crate) async fn add(dim_conf_key: &str, add_req: &StatsConfDimColAddReq, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = stats_pg_initializer::init_conf_dim_col_table_and_conn(bs_inst, ctx, true).await?;
    let (_, conf_dim_table) = stats_pg_initializer::init_conf_dim_table_and_conn(bs_inst, ctx, true).await?;
    conn.begin().await?;
    if conn.count_by_sql(&format!("SELECT 1 FROM {conf_dim_table} WHERE key = $1"), vec![Value::from(dim_conf_key)]).await? == 0 {
        return Err(funs.err().conflict("dim_col_conf", "add", "The dimension config not exists.", "409-spi-stats-dim-conf-not-exist"));
    }
    if let Some(rel_sql) = &add_req.rel_sql {
        if !stats_valid_serv::validate_select_sql(rel_sql) {
            return Err(funs.err().conflict("dim_col_conf", "add", "The rel_sql is not a valid sql.", "409-spi-stats-dim-col-conf-rel-sql-not-valid"));
        }
    }
    if conn
        .count_by_sql(
            &format!("SELECT 1 FROM {table_name} WHERE key = $1 AND rel_conf_dim_key = $2"),
            vec![Value::from(&add_req.key), Value::from(dim_conf_key)],
        )
        .await?
        != 0
    {
        return Err(funs.err().conflict(
            "dim_col_conf",
            "add",
            "The dimension column config already exists, please delete it and then add it.",
            "409-spi-stats-dim-conf-col-exist",
        ));
    }
    let mut sql_fields = vec![];
    let mut params = vec![
        Value::from(add_req.key.to_string()),
        Value::from(add_req.show_name.clone()),
        Value::from(dim_conf_key.to_string()),
        Value::from(add_req.remark.as_ref().unwrap_or(&"".to_string()).as_str()),
    ];
    if let Some(data_type) = &add_req.data_type {
        params.push(Value::from(data_type.to_string()));
        sql_fields.push("data_type");
    }
    if let Some(rel_cert_id) = &add_req.rel_cert_id {
        params.push(Value::from(rel_cert_id.to_string()));
        sql_fields.push("rel_cert_id");
    }
    if let Some(rel_sql) = &add_req.rel_sql {
        params.push(Value::from(rel_sql.to_string()));
        sql_fields.push("rel_sql");
    }
    if let Some(dict_kind) = &add_req.dict_kind {
        params.push(Value::from(dict_kind.to_string()));
        sql_fields.push("dict_kind");
    }
    if let Some(dict_dyn_interface) = &add_req.dict_dyn_interface {
        params.push(Value::from(dict_dyn_interface.to_string()));
        sql_fields.push("dict_dyn_interface");
    }
    conn.execute_one(
        &format!(
            r#"INSERT INTO {table_name}
(key, show_name, rel_conf_dim_key, remark {})
VALUES
($1, $2, $3, $4 {})
"#,
            if sql_fields.is_empty() {
                "".to_string()
            } else {
                format!(",{}", sql_fields.join(","))
            },
            if sql_fields.is_empty() {
                "".to_string()
            } else {
                format!(",{}", sql_fields.iter().enumerate().map(|(i, _)| format!("${}", i + 5)).collect::<Vec<String>>().join(","))
            }
        ),
        params,
    )
    .await?;
    conn.commit().await?;
    Ok(())
}

pub(crate) async fn modify(
    dim_conf_key: &str,
    dim_col_conf_key: &str,
    modify_req: &StatsConfDimColModifyReq,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = stats_pg_initializer::init_conf_dim_col_table_and_conn(bs_inst, ctx, true).await?;
    conn.begin().await?;
    if let Some(rel_sql) = &modify_req.rel_sql {
        if !stats_valid_serv::validate_select_sql(rel_sql) {
            return Err(funs.err().conflict("dim_col_conf", "modify", "The rel_sql is not a valid sql.", "409-spi-stats-dim-col-conf-rel-sql-not-valid"));
        }
    }
    let mut sql_sets = vec![];
    let mut params = vec![Value::from(dim_col_conf_key.to_string()), Value::from(dim_conf_key.to_string())];
    if let Some(show_name) = &modify_req.show_name {
        sql_sets.push(format!("show_name = ${}", params.len() + 1));
        params.push(Value::from(show_name.to_string()));
    }
    if let Some(data_type) = &modify_req.data_type {
        sql_sets.push(format!("data_type = ${}", params.len() + 1));
        params.push(Value::from(data_type.to_string()));
    }
    if let Some(rel_cert_id) = &modify_req.rel_cert_id {
        sql_sets.push(format!("rel_cert_id = ${}", params.len() + 1));
        params.push(Value::from(rel_cert_id.to_string()));
    }
    if let Some(rel_sql) = &modify_req.rel_sql {
        sql_sets.push(format!("rel_sql = ${}", params.len() + 1));
        params.push(Value::from(rel_sql.to_string()));
    }
    if let Some(dict_kind) = &modify_req.dict_kind {
        sql_sets.push(format!("dict_kind = ${}", params.len() + 1));
        params.push(Value::from(dict_kind.to_string()));
    }
    if let Some(dict_dyn_interface) = &modify_req.dict_dyn_interface {
        sql_sets.push(format!("dict_dyn_interface = ${}", params.len() + 1));
        params.push(Value::from(dict_dyn_interface.to_string()));
    }
    if let Some(remark) = &modify_req.remark {
        sql_sets.push(format!("remark = ${}", params.len() + 1));
        params.push(Value::from(remark.to_string()));
    }
    if sql_sets.is_empty() {
        return Err(funs.err().bad_request("dim_col_conf", "modify", "No fields to modify.", "400-spi-stats-dim-col-conf-nothing-to-modify"));
    }
    conn.execute_one(
        &format!(
            r#"UPDATE {table_name}
SET {}
WHERE key = $1 AND rel_conf_dim_key = $2
"#,
            sql_sets.join(","),
        ),
        params,
    )
    .await?;
    conn.commit().await?;
    Ok(())
}

pub(crate) async fn delete(dim_conf_key: &str, dim_col_conf_key: Option<&str>, _funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = stats_pg_initializer::init_conf_dim_col_table_and_conn(bs_inst, ctx, true).await?;
    conn.begin().await?;
    let mut where_clause = "rel_conf_dim_key = $1".to_string();
    let mut params = vec![Value::from(dim_conf_key.to_string())];
    if let Some(dim_col_conf_key) = dim_col_conf_key {
        params.push(Value::from(dim_col_conf_key));
        where_clause.push_str(&format!(" AND key = ${}", params.len()));
    }
    conn.execute_one(&format!("DELETE FROM {table_name} WHERE {where_clause}"), params).await?;
    conn.commit().await?;
    Ok(())
}

pub(crate) async fn paginate(
    dim_conf_key: Option<String>,
    dim_col_conf_key: Option<String>,
    show_name: Option<String>,
    page_number: u32,
    page_size: u32,
    desc_by_create: Option<bool>,
    desc_by_update: Option<bool>,
    _funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<TardisPage<StatsConfDimColInfoResp>> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (conn, _) = stats_pg_initializer::init_conf_dim_col_table_and_conn(bs_inst, ctx, true).await?;
    do_paginate(
        dim_conf_key,
        dim_col_conf_key,
        show_name,
        page_number,
        page_size,
        desc_by_create,
        desc_by_update,
        &conn,
        ctx,
    )
    .await
}

async fn do_paginate(
    dim_conf_key: Option<String>,
    dim_col_conf_key: Option<String>,
    show_name: Option<String>,
    page_number: u32,
    page_size: u32,
    desc_by_create: Option<bool>,
    desc_by_update: Option<bool>,
    conn: &TardisRelDBlConnection,
    ctx: &TardisContext,
) -> TardisResult<TardisPage<StatsConfDimColInfoResp>> {
    let table_name = package_table_name("stats_conf_dim_col", ctx);
    let mut sql_where = vec!["1 = 1".to_string()];
    let mut sql_order = vec![];
    let mut params: Vec<Value> = vec![Value::from(page_size), Value::from((page_number - 1) * page_size)];
    if let Some(dim_conf_key) = dim_conf_key {
        sql_where.push(format!("rel_conf_dim_key = ${}", params.len() + 1));
        params.push(Value::from(dim_conf_key));
    }
    if let Some(dim_col_conf_key) = &dim_col_conf_key {
        sql_where.push(format!("key = ${}", params.len() + 1));
        params.push(Value::from(dim_col_conf_key));
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
                r#"SELECT
    key,
    show_name,
    rel_conf_dim_key,
    data_type,
    rel_cert_id,
    rel_sql,
    dict_kind,
    dict_dyn_interface,
    remark,
    create_time,
    update_time,
    count(*) OVER() AS total
FROM {table_name}
WHERE {}
{}
LIMIT $1 OFFSET $2"#,
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
    let result = result
        .into_iter()
        .map(|item| {
            if total_size == 0 {
                total_size = item.try_get("", "total")?;
            }
            Ok(StatsConfDimColInfoResp {
                key: item.try_get("", "key")?,
                show_name: item.try_get("", "show_name")?,
                rel_conf_dim_key: item.try_get("", "rel_conf_dim_key")?,
                data_type: if item.try_get::<Option<String>>("", "data_type")?.is_none() {
                    None
                } else {
                    Some(item.try_get("", "data_type")?)
                },
                rel_cert_id: item.try_get("", "rel_cert_id")?,
                rel_sql: item.try_get("", "rel_sql")?,
                dict_kind: item.try_get("", "dict_kind")?,
                dict_dyn_interface: item.try_get("", "dict_dyn_interface")?,
                remark: item.try_get("", "remark")?,
                create_time: item.try_get("", "create_time")?,
                update_time: item.try_get("", "update_time")?,
            })
        })
        .collect::<TardisResult<_>>()?;
    Ok(TardisPage {
        page_size: page_size as u64,
        page_number: page_number as u64,
        total_size: total_size as u64,
        records: result,
    })
}

pub(crate) async fn exec_rel_sql(
    dim_conf_key: &str,
    dim_col_conf_key: &str,
    params: &[String],
    funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<Vec<serde_json::Value>> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (conn, _) = stats_pg_initializer::init_conf_dim_col_table_and_conn(bs_inst, ctx, true).await?;
    let page = do_paginate(
        Some(dim_conf_key.to_string()),
        Some(dim_col_conf_key.to_string()),
        None,
        1,
        1,
        None,
        None,
        &conn,
        ctx,
    )
    .await?;
    let dim_col = page.records.into_iter().next().ok_or_else(|| {
        funs.err().not_found(
            "dim_col_conf",
            "exec_rel_sql",
            &format!("The dimension column config [{dim_col_conf_key}] not found under dimension [{dim_conf_key}]."),
            "404-spi-stats-dim-col-conf-not-exist",
        )
    })?;
    let cert_id = dim_col.rel_cert_id.ok_or_else(|| {
        funs.err().bad_request("dim_col_conf", "exec_rel_sql", "The rel_cert_id is required.", "400-spi-stats-dim-col-conf-rel-cert-id-required")
    })?;
    let sql = dim_col.rel_sql.ok_or_else(|| {
        funs.err().bad_request("dim_col_conf", "exec_rel_sql", "The rel_sql is required.", "400-spi-stats-dim-col-conf-rel-sql-required")
    })?;
    if cert_id.is_empty() || sql.is_empty() {
        return Err(funs.err().bad_request(
            "dim_col_conf",
            "exec_rel_sql",
            "The rel_cert_id and rel_sql must not be empty.",
            "400-spi-stats-dim-col-conf-rel-sql-empty",
        ));
    }
    if !stats_valid_serv::validate_select_sql(&sql) {
        return Err(funs.err().bad_request(
            "dim_col_conf",
            "exec_rel_sql",
            "The rel_sql is not a valid select sql.",
            "400-spi-stats-dim-col-conf-rel-sql-not-valid",
        ));
    }
    let data_source_conn = stats_cert_serv::get_db_conn_by_cert_id(&cert_id, funs, ctx).await?;
    // 根据维度列的数据类型转换参数，避免类型不匹配错误（如 timestamp >= text）
    let data_type = dim_col.data_type.as_ref();
    let sql_params: TardisResult<Vec<Value>> = params
        .iter()
        .map(|s| match data_type {
            Some(crate::stats_enumeration::StatsDataTypeKind::Int) => s.parse::<i32>()
                .map(Value::from)
                .map_err(|e| funs.err().bad_request("dim_col_conf", "exec_rel_sql", &format!("Failed to parse int parameter: {e}"), "400-spi-stats-dim-col-conf-param-parse-error")),
            Some(crate::stats_enumeration::StatsDataTypeKind::Float) => s.parse::<f32>()
                .map(Value::from)
                .map_err(|e| funs.err().bad_request("dim_col_conf", "exec_rel_sql", &format!("Failed to parse float parameter: {e}"), "400-spi-stats-dim-col-conf-param-parse-error")),
            Some(crate::stats_enumeration::StatsDataTypeKind::Double) => s.parse::<f64>()
                .map(Value::from)
                .map_err(|e| funs.err().bad_request("dim_col_conf", "exec_rel_sql", &format!("Failed to parse double parameter: {e}"), "400-spi-stats-dim-col-conf-param-parse-error")),
            Some(crate::stats_enumeration::StatsDataTypeKind::Boolean) => s.parse::<bool>()
                .map(Value::from)
                .map_err(|e| funs.err().bad_request("dim_col_conf", "exec_rel_sql", &format!("Failed to parse bool parameter: {e}"), "400-spi-stats-dim-col-conf-param-parse-error")),
            Some(crate::stats_enumeration::StatsDataTypeKind::Date) => NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .map(Value::from)
                .map_err(|e| funs.err().bad_request("dim_col_conf", "exec_rel_sql", &format!("Failed to parse date parameter (expected format YYYY-MM-DD): {e}"), "400-spi-stats-dim-col-conf-param-parse-error")),
            Some(crate::stats_enumeration::StatsDataTypeKind::DateTime) => DateTime::parse_from_rfc3339(s)
                .map(|dt| Value::from(dt.with_timezone(&Utc)))
                .map_err(|e| funs.err().bad_request("dim_col_conf", "exec_rel_sql", &format!("Failed to parse datetime parameter (expected RFC3339 format): {e}"), "400-spi-stats-dim-col-conf-param-parse-error")),
            Some(_) | None => Ok(Value::from(s.as_str())),
        })
        .collect();
    let results = data_source_conn.query_all(&sql, sql_params?).await?;
    results
        .iter()
        .map(|item| {
            serde_json::Value::from_query_result_optional(item, "")
                .map(|x| x.unwrap_or(serde_json::Value::Null))
                .map_err(Into::into)
        })
        .collect()
}
