use bios_basic::spi::{
    spi_funs::SpiBsInst,
    spi_initializer::common_pg::{self, package_table_name},
};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    chrono::Utc,
    db::{
        reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
        sea_orm::Value,
    },
    web::web_resp::TardisPage,
    TardisFuns, TardisFunsInst,
};

use crate::{
    dto::stats_conf_dto::{StatsConfFactDetailAddReq, StatsConfFactDetailInfoResp, StatsConfFactDetailModifyReq},
    serv::{stats_cert_serv, stats_valid_serv},
    stats_enumeration::{StatsFactColKind, StatsFactDetailKind, StatsFactDetailMethodKind},
};

use super::stats_pg_initializer;

pub(crate) async fn add(
    fact_conf_key: &str,
    fact_conf_col_key: Option<&str>,
    add_req: &StatsConfFactDetailAddReq,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = stats_pg_initializer::init_conf_fact_detail_table_and_conn(bs_inst, ctx, true).await?;
    let (_, conf_fact_table) = stats_pg_initializer::init_conf_fact_table_and_conn(bs_inst, ctx, true).await?;
    let (_, conf_fact_col_table) = stats_pg_initializer::init_conf_fact_col_table_and_conn(bs_inst, ctx, true).await?;
    conn.begin().await?;
    // check if this fact exists
    if conn.count_by_sql(&format!("SELECT 1 FROM {conf_fact_table} WHERE key = $1"), vec![Value::from(fact_conf_key)]).await? == 0 {
        return Err(funs.err().conflict("fact_detail_conf", "add", "The fact config not exists.", "409-spi-stats-fact-detail-not-exist"));
    }
    // check if this fact col exists
    if let Some(fact_conf_col_key) = fact_conf_col_key {
        if conn
            .count_by_sql(
                &format!("SELECT 1 FROM {conf_fact_col_table} WHERE key = $1 AND rel_conf_fact_key = $2 and rel_external_id = $3 and kind = $4",),
                vec![
                    Value::from(fact_conf_col_key),
                    Value::from(fact_conf_key),
                    Value::from(""),
                    Value::from(StatsFactColKind::Measure.to_string()),
                ],
            )
            .await?
            == 0
        {
            return Err(funs.err().conflict("fact_detail_conf", "add", "The fact column config not exists.", "409-spi-stats-fact-detail-conf-not-exist"));
        }
    }
    // check if the fact col kind is dimension when the detail kind is dimension
    match add_req.kind {
        StatsFactDetailKind::Dimension => {
            if conn
                .count_by_sql(
                    &format!("SELECT 1 FROM {conf_fact_col_table} WHERE key = $1 AND rel_conf_fact_key = $2 AND rel_external_id = $3",),
                    vec![Value::from(add_req.key.clone()), Value::from(fact_conf_key), Value::from("")],
                )
                .await?
                == 0
            {
                return Err(funs.err().conflict(
                    "fact_detail_conf",
                    "add",
                    "The fact column config kind is not dimension.",
                    "409-spi-stats-fact-detail-conf-fact-col-not-dimension",
                ));
            }
        }
        StatsFactDetailKind::External => {
            if add_req.method.is_none() {
                return Err(funs.err().conflict(
                    "fact_detail_conf",
                    "add",
                    "The method is required when the kind is External.",
                    "409-spi-stats-fact-detail-conf-method-required",
                ));
            }
        }
    }
    match add_req.method.clone().unwrap_or(StatsFactDetailMethodKind::Sql) {
        StatsFactDetailMethodKind::Sql => {
            if let Some(rel_sql) = &add_req.rel_sql {
                if !stats_valid_serv::validate_select_sql(rel_sql) {
                    return Err(funs.err().conflict(
                        "fact_detail_conf",
                        "add",
                        "The rel_sql is not a valid sql.",
                        "409-spi-stats-fact-detail-conf-rel-sql-not-valid",
                    ));
                }
            }
        }
        StatsFactDetailMethodKind::Url => {
            if let Some(rel_url) = &add_req.rel_url {
                if !stats_valid_serv::validate_url(rel_url) {
                    return Err(funs.err().conflict(
                        "fact_detail_conf",
                        "add",
                        "The rel_url is not a valid url.",
                        "409-spi-stats-fact-detail-conf-rel-url-not-valid",
                    ));
                }
            }
        }
    }
    // check if the fact detail exists
    let conf_params = vec![Value::from(&add_req.key), Value::from(fact_conf_key), Value::from(fact_conf_col_key.unwrap_or(""))];
    if conn
        .count_by_sql(
            &format!("SELECT 1 FROM {table_name} WHERE key = $1 AND rel_conf_fact_key = $2 AND rel_conf_fact_col_key =$3",),
            conf_params,
        )
        .await?
        != 0
    {
        return Err(funs.err().conflict(
            "fact_detail_conf",
            "add",
            "The fact detail config already exists, please delete it and then add it.",
            "409-spi-stats-fact-detail-conf-exist",
        ));
    }

    let mut sql_fields = vec![];
    let mut params = vec![
        Value::from(add_req.key.to_string()),
        Value::from(add_req.show_name.clone()),
        Value::from(add_req.kind.to_string()),
        Value::from(fact_conf_key.to_string()),
        Value::from(fact_conf_col_key.unwrap_or("").to_string()),
        Value::from(add_req.remark.as_ref().unwrap_or(&"".to_string()).as_str()),
        Value::from(add_req.sort.unwrap_or(0)),
    ];
    if let Some(method) = &add_req.method {
        params.push(Value::from(method.to_string()));
        sql_fields.push("method");
    }
    if let Some(rel_cert_id) = &add_req.rel_cert_id {
        params.push(Value::from(rel_cert_id.to_string()));
        sql_fields.push("rel_cert_id");
    }
    if let Some(rel_sql) = &add_req.rel_sql {
        params.push(Value::from(rel_sql.to_string()));
        sql_fields.push("rel_sql");
    }
    if let Some(rel_url) = &add_req.rel_url {
        params.push(Value::from(rel_url.to_string()));
        sql_fields.push("rel_url");
    }
    conn.execute_one(
        &format!(
            r#"INSERT INTO {table_name}
(key, show_name, kind, rel_conf_fact_key, rel_conf_fact_col_key, remark, sort {})
VALUES
($1, $2, $3, $4, $5, $6, $7 {})
"#,
            if sql_fields.is_empty() { "".to_string() } else { format!(",{}", sql_fields.join(",")) },
            if sql_fields.is_empty() {
                "".to_string()
            } else {
                format!(",{}", sql_fields.iter().enumerate().map(|(i, _)| format!("${}", i + 8)).collect::<Vec<String>>().join(","))
            }
        ),
        params,
    )
    .await?;
    conn.commit().await?;
    Ok(())
}

pub(crate) async fn modify(
    fact_conf_key: &str,
    fact_conf_col_key: Option<&str>,
    fact_conf_detail_key: &str,
    modify_req: &StatsConfFactDetailModifyReq,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = stats_pg_initializer::init_conf_fact_detail_table_and_conn(bs_inst, ctx, true).await?;
    conn.begin().await?;
    if let Some(fact_detail_info) = get_fact_detail(fact_conf_key, fact_conf_col_key.unwrap_or(""), fact_conf_detail_key, funs, ctx, inst).await? {
        let method = fact_detail_info.method.unwrap_or(StatsFactDetailMethodKind::Sql);
        match method {
            StatsFactDetailMethodKind::Sql => {
                if let Some(rel_sql) = &modify_req.rel_sql {
                    if !stats_valid_serv::validate_select_sql(rel_sql) {
                        return Err(funs.err().conflict(
                            "fact_detail_conf",
                            "modify",
                            "The rel_sql is not a valid sql.",
                            "409-spi-stats-fact-detail-conf-rel-sql-not-valid",
                        ));
                    }
                }
            }
            StatsFactDetailMethodKind::Url => {
                if let Some(rel_url) = &modify_req.rel_url {
                    if !stats_valid_serv::validate_url(rel_url) {
                        return Err(funs.err().conflict(
                            "fact_detail_conf",
                            "modify",
                            "The rel_url is not a valid url.",
                            "409-spi-stats-fact-detail-conf-rel-url-not-valid",
                        ));
                    }
                }
            }
        }
    } else {
        return Err(funs.err().not_found(
            "fact_col_conf",
            "modify",
            &format!("The fact detail config {} not exists.", fact_conf_detail_key),
            "404-spi-stats-fact-col-conf-not-exist",
        ));
    }
    let mut sql_sets = vec![];
    let mut params = vec![
        Value::from(fact_conf_detail_key.to_string()),
        Value::from(fact_conf_key.to_string()),
        Value::from(fact_conf_col_key.unwrap_or("").to_string()),
    ];
    if let Some(show_name) = &modify_req.show_name {
        sql_sets.push(format!("show_name = ${}", params.len() + 1));
        params.push(Value::from(show_name.to_string()));
    }
    if let Some(remark) = &modify_req.remark {
        sql_sets.push(format!("remark = ${}", params.len() + 1));
        params.push(Value::from(remark.to_string()));
    }
    if let Some(rel_sql) = &modify_req.rel_sql {
        sql_sets.push(format!("rel_sql = ${}", params.len() + 1));
        params.push(Value::from(rel_sql.to_string()));
    }
    if let Some(rel_cert_id) = &modify_req.rel_cert_id {
        sql_sets.push(format!("rel_cert_id = ${}", params.len() + 1));
        params.push(Value::from(rel_cert_id.to_string()));
    }
    if let Some(sort) = &modify_req.sort {
        sql_sets.push(format!("sort = ${}", params.len() + 1));
        params.push(Value::from(*sort));
    }
    if sql_sets.is_empty() {
        return Ok(());
    }
    sql_sets.push(format!("update_time = ${}", params.len() + 1));
    params.push(Value::from(Utc::now()));
    conn.execute_one(
        &format!(
            r#"UPDATE {table_name}
SET {}
WHERE key = $1 AND rel_conf_fact_key = $2 AND rel_conf_fact_col_key = $3
"#,
            sql_sets.join(","),
        ),
        params,
    )
    .await?;
    conn.commit().await?;
    Ok(())
}

pub(crate) async fn delete(
    fact_conf_key: &str,
    fact_conf_col_key: Option<&str>,
    fact_conf_detail_key: &str,
    _funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = stats_pg_initializer::init_conf_fact_detail_table_and_conn(bs_inst, ctx, true).await?;
    conn.begin().await?;

    let params = vec![
        Value::from(fact_conf_detail_key.to_string()),
        Value::from(fact_conf_key.to_string()),
        Value::from(fact_conf_col_key.unwrap_or("").to_string()),
    ];
    conn.execute_one(
        &format!("DELETE FROM {table_name} WHERE key = $1 and rel_conf_fact_key = $2 and rel_conf_fact_col_key = $3",),
        params,
    )
    .await?;
    conn.commit().await?;
    Ok(())
}

pub(crate) async fn delete_by_fact_conf_key_and_col_key(
    fact_conf_key: &str,
    fact_conf_col_key: Option<&str>,
    _funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = stats_pg_initializer::init_conf_fact_detail_table_and_conn(bs_inst, ctx, true).await?;
    conn.begin().await?;

    let params = vec![Value::from(fact_conf_key.to_string()), Value::from(fact_conf_col_key.unwrap_or("").to_string())];
    conn.execute_one(&format!("DELETE FROM {table_name} WHERE rel_conf_fact_key = $1 and rel_conf_fact_col_key = $2",), params).await?;
    conn.commit().await?;
    Ok(())
}

pub(crate) async fn delete_by_fact_conf_key(fact_conf_key: &str, _funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = stats_pg_initializer::init_conf_fact_detail_table_and_conn(bs_inst, ctx, true).await?;
    conn.begin().await?;

    let params = vec![Value::from(fact_conf_key.to_string())];
    conn.execute_one(&format!("DELETE FROM {table_name} WHERE rel_conf_fact_key = $1",), params).await?;
    conn.commit().await?;
    Ok(())
}

pub(crate) async fn find_by_fact_conf_key(fact_conf_key: &str, _funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<Vec<StatsConfFactDetailInfoResp>> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (conn, _) = stats_pg_initializer::init_conf_fact_detail_table_and_conn(bs_inst, ctx, true).await?;
    if !common_pg::check_table_exit("stats_conf_fact_detail", &conn, ctx).await? {
        return Ok(vec![]);
    }
    do_paginate(Some(fact_conf_key.to_string()), None, None, None, 1, u32::MAX, None, None, &conn, ctx).await.map(|page| page.records)
}

pub(crate) async fn find_by_fact_key_and_col_conf_key(
    fact_conf_key: &str,
    fact_conf_col_key: &str,
    _funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<Vec<StatsConfFactDetailInfoResp>> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (conn, _) = stats_pg_initializer::init_conf_fact_detail_table_and_conn(bs_inst, ctx, true).await?;
    if !common_pg::check_table_exit("stats_conf_fact_detail", &conn, ctx).await? {
        return Ok(vec![]);
    }
    do_paginate(
        Some(fact_conf_key.to_string()),
        Some(fact_conf_col_key.to_string()),
        None,
        None,
        1,
        u32::MAX,
        None,
        None,
        &conn,
        ctx,
    )
    .await
    .map(|page| page.records)
}

pub(crate) async fn find_up_by_fact_key_and_col_conf_key(
    fact_conf_key: &str,
    fact_conf_col_key: &str,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<Vec<StatsConfFactDetailInfoResp>> {
    let fact_col_details = find_by_fact_key_and_col_conf_key(fact_conf_key, fact_conf_col_key, funs, ctx, inst).await?;
    if fact_col_details.is_empty() {
        let fact_details = find_by_fact_key_and_col_conf_key(fact_conf_key, "", funs, ctx, inst).await?;
        return Ok(fact_details);
    }
    Ok(fact_col_details)
}

pub(crate) async fn get_fact_detail(
    fact_conf_key: &str,
    fact_conf_col_key: &str,
    fact_conf_detail_key: &str,
    _funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<Option<StatsConfFactDetailInfoResp>> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (conn, _) = stats_pg_initializer::init_conf_fact_detail_table_and_conn(bs_inst, ctx, true).await?;
    if !common_pg::check_table_exit("stats_conf_fact_detail", &conn, ctx).await? {
        return Ok(None);
    }
    let result = do_paginate(
        Some(fact_conf_key.to_string()),
        Some(fact_conf_col_key.to_string()),
        Some(fact_conf_detail_key.to_string()),
        None,
        1,
        1,
        None,
        None,
        &conn,
        ctx,
    )
    .await?
    .records;
    Ok(result.into_iter().next())
}

pub(crate) async fn paginate(
    fact_conf_key: Option<String>,
    fact_conf_col_key: Option<String>,
    fact_conf_detail_key: Option<String>,
    show_name: Option<String>,
    page_number: u32,
    page_size: u32,
    desc_by_create: Option<bool>,
    desc_by_update: Option<bool>,
    _funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<TardisPage<StatsConfFactDetailInfoResp>> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (conn, _) = stats_pg_initializer::init_conf_fact_detail_table_and_conn(bs_inst, ctx, true).await?;

    do_paginate(
        fact_conf_key,
        fact_conf_col_key,
        fact_conf_detail_key,
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
    fact_conf_key: Option<String>,
    fact_conf_col_key: Option<String>,
    fact_conf_detail_key: Option<String>,
    show_name: Option<String>,
    page_number: u32,
    page_size: u32,
    desc_by_create: Option<bool>,
    desc_by_update: Option<bool>,
    conn: &TardisRelDBlConnection,
    ctx: &TardisContext,
) -> TardisResult<TardisPage<StatsConfFactDetailInfoResp>> {
    let table_name = package_table_name("stats_conf_fact_detail", ctx);
    let mut sql_where = vec![];
    let mut sql_order = vec![];
    let mut params: Vec<Value> = vec![Value::from(page_size), Value::from((page_number - 1) * page_size)];
    if let Some(fact_conf_key) = fact_conf_key {
        sql_where.push(format!("fact_detail.rel_conf_fact_key = ${}", params.len() + 1));
        params.push(Value::from(fact_conf_key));
    }
    if let Some(fact_detail_conf_key) = &fact_conf_detail_key {
        sql_where.push(format!("fact_detail.key = ${}", params.len() + 1));
        params.push(Value::from(fact_detail_conf_key));
    }
    if let Some(fact_col_conf_key) = &fact_conf_col_key {
        sql_where.push(format!("fact_detail.rel_conf_fact_col_key = ${}", params.len() + 1));
        params.push(Value::from(fact_col_conf_key));
    }
    if let Some(show_name) = &show_name {
        sql_where.push(format!("fact_detail.show_name LIKE ${}", params.len() + 1));
        params.push(Value::from(format!("%{show_name}%")));
    }

    if let Some(desc_by_create) = desc_by_create {
        sql_order.push(format!("fact_detail.create_time {}", if desc_by_create { "DESC" } else { "ASC" }));
    }
    if let Some(desc_by_update) = desc_by_update {
        sql_order.push(format!("fact_detail.update_time {}", if desc_by_update { "DESC" } else { "ASC" }));
    }
    sql_order.push(format!("fact_detail.sort ASC"));

    let result = conn
        .query_all(
            &format!(
                r#"SELECT 
              fact_detail.key, 
              fact_detail.show_name, 
              fact_detail.kind, 
              fact_detail.method, 
              fact_detail.remark, 
              fact_detail.rel_conf_fact_key, 
              fact_detail.rel_conf_fact_col_key, 
              fact_detail.sort,
              fact_detail.create_time,
              fact_detail.update_time, 
              fact_detail.rel_cert_id, 
              fact_detail.rel_sql,
              fact_detail.rel_url,
              count(*) OVER() AS total
FROM {table_name} AS fact_detail 
WHERE 
  {}
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
    let result = result
        .into_iter()
        .map(|item| {
            if total_size == 0 {
                total_size = item.try_get("", "total")?;
            }
            Ok(StatsConfFactDetailInfoResp {
                key: item.try_get("", "key")?,
                show_name: item.try_get("", "show_name")?,
                kind: item.try_get("", "kind")?,
                method: if item.try_get::<Option<String>>("", "method")?.is_none() {
                    None
                } else {
                    Some(item.try_get("", "method")?)
                },
                rel_conf_fact_key: item.try_get("", "rel_conf_fact_key")?,
                rel_conf_fact_col_key: item.try_get("", "rel_conf_fact_col_key")?,
                rel_cert_id: item.try_get("", "rel_cert_id")?,
                rel_sql: item.try_get("", "rel_sql")?,
                rel_url: item.try_get("", "rel_url")?,
                remark: item.try_get("", "remark")?,
                create_time: item.try_get("", "create_time")?,
                update_time: item.try_get("", "update_time")?,
                sort: item.try_get("", "sort")?,
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

/// Execute the sql or url to get the external detail value
/// If the method is Sql, the sql will be executed to get the value.
/// If the method is Url, the url will be called to get the value.
/// e.g: sql: "SELECT count(1) FROM user WHERE org_id = '{org_id}'"
/// e.g: url: "http://example.com/api/user/count?org_id={org_id}"
///
/// 执行sql或url以获取外部明细值
/// 如果方法是Sql，则执行sql以获取值。
/// 如果方法是Url，则调用url以获取值。
/// 例如: sql: "SELECT count(1) FROM user WHERE org_id = '{org_id}'"
/// 例如: url: "http://example.com/api/user/count?org_id={org_id}"
pub async fn sql_or_url_execute(
    stats_conf_fact_detail_info: StatsConfFactDetailInfoResp,
    fact_record: serde_json::Value,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<Option<serde_json::Value>> {
    match stats_conf_fact_detail_info.method.clone().unwrap_or(StatsFactDetailMethodKind::Sql) {
        StatsFactDetailMethodKind::Sql => do_sql_execute(stats_conf_fact_detail_info, fact_record, funs, ctx).await,
        StatsFactDetailMethodKind::Url => do_url_execute(stats_conf_fact_detail_info, fact_record, funs, ctx).await,
    }
}

async fn do_sql_execute(
    stats_conf_fact_detail_info: StatsConfFactDetailInfoResp,
    fact_record: serde_json::Value,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<Option<serde_json::Value>> {
    let Some(cert_id) = stats_conf_fact_detail_info.rel_cert_id else {
        return Ok(None);
    };
    let Some(sql) = stats_conf_fact_detail_info.rel_sql else {
        return Ok(None);
    };
    if cert_id.is_empty() || sql.is_empty() {
        return Ok(None);
    }
    let data_source_conn = stats_cert_serv::get_db_conn_by_cert_id(&cert_id, funs, ctx).await?;
    match stats_valid_serv::process_sql_json(&sql, fact_record) {
        Err(_) => return Ok(None),
        Ok((processed_sql, params)) => {
            if let Some(rel_record) = data_source_conn.query_one(&processed_sql, params).await? {
                if let Some(first_column) = rel_record.column_names().get(0) {
                    if let Ok(int_val) = rel_record.try_get::<i64>("", first_column) {
                        return Ok(Some(serde_json::Value::from(int_val)));
                    } else if let Ok(float_val) = rel_record.try_get::<f64>("", first_column) {
                        return Ok(Some(serde_json::Value::from(float_val)));
                    } else if let Ok(bool_val) = rel_record.try_get::<bool>("", first_column) {
                        return Ok(Some(serde_json::Value::from(bool_val)));
                    } else {
                        let str_val: String = rel_record.try_get("", first_column)?;
                        return Ok(Some(serde_json::Value::from(str_val)));
                    }
                }
            }
        }
    }
    Ok(None)
}

async fn do_url_execute(
    stats_conf_fact_detail_info: StatsConfFactDetailInfoResp,
    fact_record: serde_json::Value,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<Option<serde_json::Value>> {
    let Some(rel_url) = stats_conf_fact_detail_info.rel_url else {
        return Ok(None);
    };
    if rel_url.is_empty() {
        return Ok(None);
    }
    let url = stats_valid_serv::process_url_json(&rel_url, fact_record)?;
    let headers: Vec<(String, String)> = vec![("Tardis-Context".to_string(), TardisFuns::crypto.base64.encode(TardisFuns::json.obj_to_string(&ctx)?))];
    let resp = funs.web_client().get_to_str(url, headers).await;
    match resp {
        Ok(resp) => return Ok(Some(serde_json::Value::from(resp.body))),
        Err(_) => return Ok(None),
    }
}
