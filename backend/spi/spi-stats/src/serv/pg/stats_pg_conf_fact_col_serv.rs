use bios_basic::spi::{
    spi_funs::SpiBsInst,
    spi_initializer::{
        self,
        common_pg::{self, package_table_name, AlterColumnKind},
    },
};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    chrono::{DateTime, Utc},
    db::{
        reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
        sea_orm::Value,
    },
    web::web_resp::TardisPage,
    TardisFunsInst,
};

use crate::{
    dto::stats_conf_dto::{StatsConfFactColAddReq, StatsConfFactColInfoResp, StatsConfFactColModifyReq},
    stats_enumeration::{StatsDataTypeKind, StatsFactColKind},
};

use super::{stats_pg_conf_dim_serv, stats_pg_conf_fact_serv, stats_pg_initializer, stats_pg_sync_serv};

pub(crate) async fn add(fact_conf_key: &str, add_req: &StatsConfFactColAddReq, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = stats_pg_initializer::init_conf_fact_col_table_and_conn(bs_inst, ctx, true).await?;
    let (_, conf_fact_table) = stats_pg_initializer::init_conf_fact_table_and_conn(bs_inst, ctx, true).await?;
    conn.begin().await?;
    // check if this fact exists
    if conn.count_by_sql(&format!("SELECT 1 FROM {conf_fact_table} WHERE key = $1"), vec![Value::from(fact_conf_key)]).await? == 0 {
        return Err(funs.err().conflict("fact_col_conf", "add", "The fact config not exists.", "409-spi-stats-fact-conf-not-exist"));
    }
    if let Some(rel_sql) = &add_req.rel_sql {
        if !stats_pg_sync_serv::validate_fact_col_sql(rel_sql) {
            return Err(funs.err().conflict("fact_col_conf", "add", "The rel_sql is not a valid sql.", "409-spi-stats-fact-col-conf-rel-sql-not-valid"));
        }
    }
    // todo cancel check if fact_conf_table online
    // if add_req.rel_external_id.is_none() && stats_pg_conf_fact_serv::online(fact_conf_key, &conn, ctx).await? {
    //     return Err(funs.err().conflict(
    //         "fact_col_conf",
    //         "add",
    //         "The fact instance table already exists, please delete it and then modify it.",
    //         "409-spi-stats-fact-inst-exist",
    //     ));
    // }
    let conf_params = if let Some(rel_external_ids) = add_req.rel_external_id.clone() {
        vec![
            Value::from(&add_req.key),
            Value::from(fact_conf_key),
            Value::from(add_req.kind.to_string()),
            Value::from("".to_string()),
            Value::from(rel_external_ids),
        ]
    } else {
        vec![Value::from(&add_req.key), Value::from(fact_conf_key), Value::from(add_req.kind.to_string())]
    };
    if conn
        .count_by_sql(
            &format!(
                "SELECT 1 FROM {table_name} WHERE key = $1 AND rel_conf_fact_key = $2 AND kind =$3 {}",
                if add_req.rel_external_id.is_some() {
                    "AND rel_external_id IN ($4,$5)".to_string()
                } else {
                    "AND rel_external_id  = ''".to_string()
                }
            ),
            conf_params,
        )
        .await?
        != 0
    {
        return Err(funs.err().conflict(
            "fact_col_conf",
            "add",
            "The fact column config already exists, please delete it and then add it.",
            "409-spi-stats-fact-conf-col-exist",
        ));
    }
    if let Some(dim_rel_conf_dim_key) = &add_req.dim_rel_conf_dim_key {
        if add_req.rel_external_id.is_none() && !stats_pg_conf_dim_serv::online(dim_rel_conf_dim_key, &conn, ctx).await? {
            return Err(funs.err().conflict("fact_col_conf", "add", "The dimension config not online.", "409-spi-stats-dim-conf-not-online"));
        }
    }
    let mut sql_fields = vec![];
    let mut params = vec![
        Value::from(add_req.key.to_string()),
        Value::from(add_req.show_name.clone()),
        Value::from(add_req.kind.to_string()),
        Value::from(fact_conf_key.to_string()),
        Value::from(add_req.remark.as_ref().unwrap_or(&"".to_string()).as_str()),
        Value::from(add_req.rel_external_id.as_ref().unwrap_or(&"".to_string()).as_str()),
    ];
    if let Some(dim_rel_conf_dim_key) = &add_req.dim_rel_conf_dim_key {
        params.push(Value::from(dim_rel_conf_dim_key.to_string()));
        sql_fields.push("dim_rel_conf_dim_key");
    }
    if let Some(dim_multi_values) = add_req.dim_multi_values {
        params.push(Value::from(dim_multi_values));
        sql_fields.push("dim_multi_values");
    }
    if let Some(mes_data_distinct) = add_req.mes_data_distinct {
        params.push(Value::from(mes_data_distinct));
        sql_fields.push("mes_data_distinct");
    }
    if let Some(mes_data_type) = &add_req.mes_data_type {
        params.push(Value::from(mes_data_type.to_string()));
        sql_fields.push("mes_data_type");
    }
    if let Some(mes_frequency) = &add_req.mes_frequency {
        params.push(Value::from(mes_frequency.to_string()));
        sql_fields.push("mes_frequency");
    }
    if let Some(mes_unit) = &add_req.mes_unit {
        params.push(Value::from(mes_unit));
        sql_fields.push("mes_unit");
    }
    if let Some(mes_act_by_dim_conf_keys) = &add_req.mes_act_by_dim_conf_keys {
        params.push(Value::from(mes_act_by_dim_conf_keys.clone()));
        sql_fields.push("mes_act_by_dim_conf_keys");
    }
    if let Some(rel_conf_fact_and_col_key) = &add_req.rel_conf_fact_and_col_key {
        params.push(Value::from(rel_conf_fact_and_col_key.to_string()));
        sql_fields.push("rel_conf_fact_and_col_key");
    }
    if let Some(dim_exclusive_rec) = add_req.dim_exclusive_rec {
        params.push(Value::from(dim_exclusive_rec));
        sql_fields.push("dim_exclusive_rec");
    }
    if let Some(dim_data_type) = &add_req.dim_data_type {
        params.push(Value::from(dim_data_type.to_string()));
        sql_fields.push("dim_data_type");
    }
    if let Some(dim_dynamic_url) = &add_req.dim_dynamic_url {
        params.push(Value::from(dim_dynamic_url.to_string()));
        sql_fields.push("dim_dynamic_url");
    }
    if let Some(rel_cert_id) = &add_req.rel_cert_id {
        params.push(Value::from(rel_cert_id.to_string()));
        sql_fields.push("rel_cert_id");
    }
    if let Some(rel_field) = &add_req.rel_field {
        params.push(Value::from(rel_field.to_string()));
        sql_fields.push("rel_field");
    }
    if let Some(rel_sql) = &add_req.rel_sql {
        params.push(Value::from(rel_sql.to_string()));
        sql_fields.push("rel_sql");
    }
    conn.execute_one(
        &format!(
            r#"INSERT INTO {table_name}
(key, show_name, kind, rel_conf_fact_key, remark, rel_external_id {})
VALUES
($1, $2, $3, $4, $5, $6 {})
"#,
            if sql_fields.is_empty() { "".to_string() } else { format!(",{}", sql_fields.join(",")) },
            if sql_fields.is_empty() {
                "".to_string()
            } else {
                format!(",{}", sql_fields.iter().enumerate().map(|(i, _)| format!("${}", i + 7)).collect::<Vec<String>>().join(","))
            }
        ),
        params,
    )
    .await?;
    // alter inst table column
    if add_req.rel_external_id.is_none() {
        let fact_col_conf = StatsConfFactColInfoResp {
            key: add_req.key.to_string(),
            show_name: add_req.show_name.clone(),
            kind: add_req.kind.clone(),
            dim_rel_conf_dim_key: add_req.dim_rel_conf_dim_key.clone(),
            dim_multi_values: add_req.dim_multi_values,
            dim_data_type: add_req.dim_data_type.clone(),
            dim_dynamic_url: add_req.dim_dynamic_url.clone(),
            mes_data_distinct: add_req.mes_data_distinct,
            mes_data_type: add_req.mes_data_type.clone(),
            mes_frequency: add_req.mes_frequency.clone(),
            mes_unit: add_req.mes_unit.clone(),
            mes_act_by_dim_conf_keys: add_req.mes_act_by_dim_conf_keys.clone(),
            rel_conf_fact_key: Some(fact_conf_key.to_owned()),
            rel_conf_fact_and_col_key: add_req.rel_conf_fact_and_col_key.clone(),
            rel_external_id: add_req.rel_external_id.clone(),
            dim_exclusive_rec: add_req.dim_exclusive_rec,
            remark: add_req.remark.clone(),
            create_time: Utc::now(),
            update_time: Utc::now(),
            rel_field: add_req.rel_field.clone(),
            rel_sql: add_req.rel_sql.clone(),
            rel_cert_id: add_req.rel_field.clone(),
        };
        alter_inst_table_column(fact_conf_key, fact_col_conf, &AlterColumnKind::Add, &conn, funs, ctx, inst).await?;
    }
    conn.commit().await?;
    Ok(())
}

pub(crate) async fn modify(
    fact_conf_key: &str,
    fact_col_conf_key: &str,
    modify_req: &StatsConfFactColModifyReq,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = stats_pg_initializer::init_conf_fact_col_table_and_conn(bs_inst, ctx, true).await?;
    conn.begin().await?;
    // todo cancel check if fact_conf_table online
    // if stats_pg_conf_fact_serv::online(fact_conf_key, &conn, ctx).await? {
    //     return Err(funs.err().conflict(
    //         "fact_col_conf",
    //         "modify",
    //         "The fact instance table already exists, please delete it and then modify it.",
    //         "409-spi-stats-fact-inst-exist",
    //     ));
    // }
    if let Some(rel_sql) = &modify_req.rel_sql {
        if !stats_pg_sync_serv::validate_fact_col_sql(rel_sql) {
            return Err(funs.err().conflict("fact_col_conf", "add", "The rel_sql is not a valid sql.", "409-spi-stats-fact-col-conf-rel-sql-not-valid"));
        }
    }
    let mut sql_sets = vec![];
    let mut params = vec![Value::from(fact_col_conf_key.to_string()), Value::from(fact_conf_key.to_string())];
    if let Some(rel_external_id) = &modify_req.rel_external_id {
        params.push(Value::from(rel_external_id.to_string()));
    } else {
        params.push(Value::from("".to_string()));
    }
    if let Some(show_name) = &modify_req.show_name {
        sql_sets.push(format!("show_name = ${}", params.len() + 1));
        params.push(Value::from(show_name.to_string()));
    }
    if let Some(kind) = &modify_req.kind {
        sql_sets.push(format!("kind = ${}", params.len() + 1));
        params.push(Value::from(kind.to_string()));
    }
    if let Some(dim_rel_conf_dim_key) = &modify_req.dim_rel_conf_dim_key {
        sql_sets.push(format!("dim_rel_conf_dim_key = ${}", params.len() + 1));
        params.push(Value::from(dim_rel_conf_dim_key.to_string()));
    }
    if let Some(dim_multi_values) = modify_req.dim_multi_values {
        sql_sets.push(format!("dim_multi_values = ${}", params.len() + 1));
        params.push(Value::from(dim_multi_values));
    }
    if let Some(mes_data_distinct) = modify_req.mes_data_distinct {
        sql_sets.push(format!("mes_data_distinct = ${}", params.len() + 1));
        params.push(Value::from(mes_data_distinct));
    }
    if let Some(mes_data_type) = &modify_req.mes_data_type {
        sql_sets.push(format!("mes_data_type = ${}", params.len() + 1));
        params.push(Value::from(mes_data_type.to_string()));
    }
    if let Some(mes_frequency) = &modify_req.mes_frequency {
        sql_sets.push(format!("mes_frequency = ${}", params.len() + 1));
        params.push(Value::from(mes_frequency.to_string()));
    }
    if let Some(mes_unit) = &modify_req.mes_unit {
        sql_sets.push(format!("mes_unit = ${}", params.len() + 1));
        params.push(Value::from(mes_unit.to_string()));
    }
    if let Some(mes_act_by_dim_conf_keys) = &modify_req.mes_act_by_dim_conf_keys {
        sql_sets.push(format!("mes_act_by_dim_conf_keys = ${}", params.len() + 1));
        params.push(Value::from(mes_act_by_dim_conf_keys.clone()));
    }
    if let Some(rel_conf_fact_and_col_key) = &modify_req.rel_conf_fact_and_col_key {
        sql_sets.push(format!("rel_conf_fact_and_col_key = ${}", params.len() + 1));
        params.push(Value::from(rel_conf_fact_and_col_key.to_string()));
    }
    if let Some(remark) = &modify_req.remark {
        sql_sets.push(format!("remark = ${}", params.len() + 1));
        params.push(Value::from(remark.to_string()));
    }
    if let Some(dim_exclusive_rec) = modify_req.dim_exclusive_rec {
        sql_sets.push(format!("rel_conf_fact_and_col_key = ${}", params.len() + 1));
        params.push(Value::from(dim_exclusive_rec));
    }
    if let Some(dim_data_type) = &modify_req.dim_data_type {
        sql_sets.push(format!("dim_data_type = ${}", params.len() + 1));
        params.push(Value::from(dim_data_type.to_string()));
    }
    if let Some(dim_dynamic_url) = &modify_req.dim_dynamic_url {
        sql_sets.push(format!("dim_dynamic_url = ${}", params.len() + 1));
        params.push(Value::from(dim_dynamic_url.to_string()));
    }
    if let Some(rel_field) = &modify_req.rel_field {
        sql_sets.push(format!("rel_field = ${}", params.len() + 1));
        params.push(Value::from(rel_field.to_string()));
    }
    if let Some(rel_sql) = &modify_req.rel_sql {
        sql_sets.push(format!("rel_sql = ${}", params.len() + 1));
        params.push(Value::from(rel_sql.to_string()));
    }
    if let Some(rel_cert_id) = &modify_req.rel_cert_id {
        sql_sets.push(format!("rel_cert_id = ${}", params.len() + 1));
        params.push(Value::from(rel_cert_id.to_string()));
    }
    conn.execute_one(
        &format!(
            r#"UPDATE {table_name}
SET {}
WHERE key = $1 AND rel_conf_fact_key = $2 AND rel_external_id = $3
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
    fact_col_conf_key: Option<&str>,
    rel_external_id: Option<String>,
    kind: Option<StatsFactColKind>,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = stats_pg_initializer::init_conf_fact_col_table_and_conn(bs_inst, ctx, true).await?;
    conn.begin().await?;
    if rel_external_id.is_none() {
        let fact_col_confs: Vec<StatsConfFactColInfoResp> = if let Some(fact_col_conf_key) = fact_col_conf_key {
            if let Some(fact_col_conf) = find_by_fact_key_and_col_conf_key(fact_conf_key, fact_col_conf_key, funs, ctx, inst).await? {
                vec![fact_col_conf]
            } else {
                vec![]
            }
        } else {
            find_by_fact_conf_key(fact_conf_key, funs, ctx, inst).await?
        };
        for fact_col_conf in fact_col_confs {
            alter_inst_table_column(fact_conf_key, fact_col_conf, &AlterColumnKind::Delete, &conn, funs, ctx, inst).await?;
        }
    }
    let mut where_clause = String::from("rel_conf_fact_key = $1");
    let mut params = vec![Value::from(fact_conf_key.to_string())];
    if let Some(fact_col_conf_key) = fact_col_conf_key {
        params.push(Value::from(fact_col_conf_key));
        where_clause.push_str(&format!(" AND key = ${param_idx}", param_idx = params.len()));
    }
    if let Some(kind) = kind {
        params.push(Value::from(kind.to_string()));
        where_clause.push_str(&format!(" AND kind = ${param_idx}", param_idx = params.len()));
    }
    if let Some(rel_external_id) = rel_external_id {
        params.push(Value::from(rel_external_id));
        where_clause.push_str(&format!(" AND rel_external_id = ${param_idx}", param_idx = params.len()));
    } else if fact_col_conf_key.is_some() {
        params.push(Value::from("".to_string()));
        where_clause.push_str(&format!(" AND rel_external_id = ${param_idx}", param_idx = params.len()));
    }
    conn.execute_one(&format!("DELETE FROM {table_name} WHERE {where_clause}"), params).await?;
    conn.commit().await?;
    Ok(())
}

pub(crate) async fn find_by_fact_conf_key(fact_conf_key: &str, _funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<Vec<StatsConfFactColInfoResp>> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (conn, _) = stats_pg_initializer::init_conf_fact_col_table_and_conn(bs_inst, ctx, true).await?;
    if !common_pg::check_table_exit("stats_conf_fact_col", &conn, ctx).await? {
        return Ok(vec![]);
    }
    do_paginate(Some(fact_conf_key.to_string()), None, None, None, None, None, 1, u32::MAX, None, None, &conn, ctx).await.map(|page| page.records)
}

pub(crate) async fn find_by_fact_key_and_col_conf_key(
    fact_conf_key: &str,
    fact_col_conf_key: &str,
    _funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<Option<StatsConfFactColInfoResp>> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (conn, _) = stats_pg_initializer::init_conf_fact_col_table_and_conn(bs_inst, ctx, true).await?;
    if !common_pg::check_table_exit("stats_conf_fact_col", &conn, ctx).await? {
        return Ok(None);
    }
    let result = do_paginate(
        Some(fact_conf_key.to_string()),
        Some(fact_col_conf_key.to_string()),
        None,
        None,
        None,
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
    fact_col_conf_key: Option<String>,
    dim_key: Option<String>,
    dim_group_key: Option<String>,
    show_name: Option<String>,
    rel_external_id: Option<String>,
    page_number: u32,
    page_size: u32,
    desc_by_create: Option<bool>,
    desc_by_update: Option<bool>,
    _funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<TardisPage<StatsConfFactColInfoResp>> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (conn, _) = stats_pg_initializer::init_conf_fact_col_table_and_conn(bs_inst, ctx, true).await?;

    do_paginate(
        fact_conf_key,
        fact_col_conf_key,
        dim_key,
        dim_group_key,
        show_name,
        rel_external_id,
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
    fact_col_conf_key: Option<String>,
    dim_key: Option<String>,
    dim_group_key: Option<String>,
    show_name: Option<String>,
    rel_external_id: Option<String>,
    page_number: u32,
    page_size: u32,
    desc_by_create: Option<bool>,
    desc_by_update: Option<bool>,
    conn: &TardisRelDBlConnection,
    ctx: &TardisContext,
) -> TardisResult<TardisPage<StatsConfFactColInfoResp>> {
    let table_name = package_table_name("stats_conf_fact_col", ctx);
    let dim_table_name = package_table_name("stats_conf_dim", ctx);
    let mut sql_where = vec![];
    let mut sql_order = vec![];
    let mut params: Vec<Value> = vec![Value::from(page_size), Value::from((page_number - 1) * page_size)];
    if let Some(fact_conf_key) = fact_conf_key {
        sql_where.push(format!("fact_col.rel_conf_fact_key = ${}", params.len() + 1));
        params.push(Value::from(fact_conf_key));
    }
    if let Some(fact_col_conf_key) = &fact_col_conf_key {
        sql_where.push(format!("fact_col.key = ${}", params.len() + 1));
        params.push(Value::from(fact_col_conf_key));
    }
    if let Some(dim_key) = &dim_key {
        sql_where.push(format!("fact_col.dim_rel_conf_dim_key = ${}", params.len() + 1));
        params.push(Value::from(dim_key));
    }
    if let Some(show_name) = &show_name {
        sql_where.push(format!("fact_col.show_name LIKE ${}", params.len() + 1));
        params.push(Value::from(format!("%{show_name}%")));
    }
    if let Some(rel_external_id) = &rel_external_id {
        sql_where.push(format!(
            "(fact_col.rel_external_id = ${} OR fact_col.rel_external_id = ${} )",
            params.len() + 1,
            params.len() + 2
        ));
        params.push(Value::from("".to_string()));
        params.push(Value::from(rel_external_id));
    } else {
        sql_where.push(format!("rel_external_id = ${}", params.len() + 1));
        params.push(Value::from("".to_string()));
    }

    if let Some(desc_by_create) = desc_by_create {
        sql_order.push(format!("fact_col.create_time {}", if desc_by_create { "DESC" } else { "ASC" }));
    }
    if let Some(desc_by_update) = desc_by_update {
        sql_order.push(format!("fact_col.update_time {}", if desc_by_update { "DESC" } else { "ASC" }));
    }

    let result;
    if let Some(dim_group_key) = &dim_group_key {
        sql_where.push(format!("dim.dim_group_key = ${}", params.len() + 1));
        params.push(Value::from(dim_group_key));
    }
    result = conn
        .query_all(
            &format!(
                r#"SELECT 
              fact_col.key, 
              fact_col.show_name, 
              fact_col.kind, 
              fact_col.remark, 
              fact_col.dim_rel_conf_dim_key, 
              fact_col.rel_external_id, 
              fact_col.dim_multi_values, 
              fact_col.dim_exclusive_rec, 
              COALESCE(dim.data_type,COALESCE(fact_col.dim_data_type,'String')) as dim_data_type, 
              fact_col.dim_dynamic_url, 
              fact_col.mes_data_distinct, 
              fact_col.mes_data_type, 
              fact_col.mes_frequency, 
              fact_col.mes_unit, 
              fact_col.mes_act_by_dim_conf_keys, 
              fact_col.rel_conf_fact_key, 
              fact_col.rel_conf_fact_and_col_key, 
              fact_col.create_time, fact_col.update_time, 
              fact_col.rel_field, 
              fact_col.rel_cert_id, 
              fact_col.rel_sql,
              count(*) OVER() AS total
FROM {table_name} AS fact_col left join {dim_table_name} AS dim on fact_col.dim_rel_conf_dim_key = dim.key
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
            Ok(StatsConfFactColInfoResp {
                key: item.try_get("", "key")?,
                show_name: item.try_get("", "show_name")?,
                kind: item.try_get("", "kind")?,
                dim_rel_conf_dim_key: item.try_get("", "dim_rel_conf_dim_key")?,
                dim_multi_values: item.try_get("", "dim_multi_values")?,
                dim_exclusive_rec: item.try_get("", "dim_exclusive_rec")?,
                dim_data_type: if item.try_get::<Option<String>>("", "dim_data_type")?.is_none() {
                    None
                } else {
                    Some(item.try_get("", "dim_data_type")?)
                },
                dim_dynamic_url: item.try_get("", "dim_dynamic_url")?,
                mes_data_distinct: item.try_get("", "mes_data_distinct")?,
                mes_data_type: if item.try_get::<Option<String>>("", "mes_data_type")?.is_none() {
                    None
                } else {
                    Some(item.try_get("", "mes_data_type")?)
                },
                mes_frequency: item.try_get("", "mes_frequency")?,
                mes_unit: item.try_get("", "mes_unit")?,
                mes_act_by_dim_conf_keys: item.try_get("", "mes_act_by_dim_conf_keys")?,
                rel_conf_fact_key: item.try_get("", "rel_conf_fact_key")?,
                rel_conf_fact_and_col_key: item.try_get("", "rel_conf_fact_and_col_key")?,
                remark: item.try_get("", "remark")?,
                create_time: item.try_get("", "create_time")?,
                update_time: item.try_get("", "update_time")?,
                rel_external_id: item.try_get("", "rel_external_id")?,
                rel_field: item.try_get("", "rel_field")?,
                rel_sql: item.try_get("", "rel_sql")?,
                rel_cert_id: item.try_get("", "rel_cert_id")?,
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

async fn alter_inst_table_column(
    fact_conf_key: &str,
    fact_col_conf: StatsConfFactColInfoResp,
    alter_column_kind: &AlterColumnKind,
    conn: &TardisRelDBlConnection,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<()> {
    if stats_pg_conf_fact_serv::online(fact_conf_key, &conn, ctx).await? {
        let mut indix: Vec<(String, &str)> = vec![];
        let col_sql = fact_col_column_sql(fact_col_conf.clone(), &mut indix, false, conn, funs, ctx, inst).await?;
        let mut swap_index = vec![];
        for i in &indix {
            swap_index.push((&i.0[..], i.1));
        }
        spi_initializer::common_pg::alter_table_column(
            &conn,
            Some(fact_conf_key),
            "stats_inst_fact",
            alter_column_kind,
            fact_col_conf.key.as_str(),
            &col_sql,
            swap_index,
            ctx,
        )
        .await?;
    }
    Ok(())
}

pub async fn fact_col_column_sql(
    fact_col_conf: StatsConfFactColInfoResp,
    index: &mut Vec<(String, &str)>,
    is_not_null: bool,
    conn: &TardisRelDBlConnection,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<String> {
    let sql;
    if fact_col_conf.kind == StatsFactColKind::Dimension {
        let Some(dim_conf_key) = &fact_col_conf.dim_rel_conf_dim_key else {
            return Err(funs.err().bad_request("fact_inst", "create", "Fail to get dimension config", "400-spi-stats-fail-to-get-dim-config-key"));
        };
        if !stats_pg_conf_dim_serv::online(dim_conf_key, conn, ctx).await? {
            return Err(funs.err().conflict(
                "fact_inst",
                "create",
                &format!("The dimension config [{dim_conf_key}] not online."),
                "409-spi-stats-dim-conf-not-online",
            ));
        }
        let Some(dim_conf) = stats_pg_conf_dim_serv::get(dim_conf_key, None, None, conn, ctx, inst).await? else {
            return Err(funs.err().conflict(
                "fact_inst",
                "create",
                &format!("Fail to get dimension config by key [{dim_conf_key}]"),
                "409-spi-stats-fail-to-get-dim-config",
            ));
        };
        if fact_col_conf.dim_multi_values.unwrap_or(false) {
            sql = format!(
                "{} {}[] {}",
                &fact_col_conf.key,
                dim_conf.data_type.to_pg_data_type(),
                if is_not_null { "NOT NULL" } else { "" }
            );
            index.push((fact_col_conf.key.clone(), "gin"));
        } else {
            sql = format!(
                "{} {} {}",
                &fact_col_conf.key,
                dim_conf.data_type.to_pg_data_type(),
                if is_not_null { "NOT NULL" } else { "" }
            );
            index.push((fact_col_conf.key.clone(), "btree"));
            match dim_conf.data_type {
                StatsDataTypeKind::DateTime => {
                    index.push((format!("date(timezone('UTC', {}))", fact_col_conf.key), "btree"));
                    index.push((format!("date_part('hour',timezone('UTC', {}))", fact_col_conf.key), "btree"));
                    index.push((format!("date_part('day',timezone('UTC', {}))", fact_col_conf.key), "btree"));
                    index.push((format!("date_part('month',timezone('UTC', {}))", fact_col_conf.key), "btree"));
                    index.push((format!("date_part('year',timezone('UTC', {}))", fact_col_conf.key), "btree"));
                }
                StatsDataTypeKind::Date => {
                    index.push((format!("date_part('day', {})", fact_col_conf.key), "btree"));
                    index.push((format!("date_part('month', {})", fact_col_conf.key), "btree"));
                    index.push((format!("date_part('year', {})", fact_col_conf.key), "btree"));
                }
                _ => {}
            }
        }
    } else if fact_col_conf.kind == StatsFactColKind::Measure {
        let Some(mes_data_type) = fact_col_conf.mes_data_type.as_ref() else {
            return Err(funs.err().conflict(
                "fact_inst",
                "create",
                "Config of kind StatsFactColKind::Measure should have a mes_data_type",
                "409-spi-stats-miss-mes-data-type",
            ));
        };
        sql = format!("{} {} {}", &fact_col_conf.key, mes_data_type.to_pg_data_type(), if is_not_null { "NOT NULL" } else { "" });
    } else {
        sql = format!("{} character varying", &fact_col_conf.key);
    }
    Ok(sql)
}
