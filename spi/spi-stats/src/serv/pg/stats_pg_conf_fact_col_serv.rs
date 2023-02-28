use bios_basic::spi::{
    spi_funs::SpiBsInstExtractor,
    spi_initializer::common_pg::{self, package_table_name},
};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    db::{
        reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
        sea_orm::Value,
    },
    web::web_resp::TardisPage,
    TardisFunsInst,
};

use crate::dto::stats_conf_dto::{StatsConfFactColAddReq, StatsConfFactColInfoResp, StatsConfFactColModifyReq};

use super::{stats_pg_conf_dim_serv, stats_pg_conf_fact_serv, stats_pg_initializer};

pub(crate) async fn add(fact_conf_key: &str, add_req: &StatsConfFactColAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = stats_pg_initializer::init_conf_fact_col_table_and_conn(bs_inst, ctx, true).await?;
    conn.begin().await?;
    if stats_pg_conf_fact_serv::online(fact_conf_key, &conn, ctx).await? {
        return Err(funs.err().conflict(
            "fact_col_conf",
            "add",
            "The fact instance table already exists, please delete it and then modify it.",
            "409-spi-stats-fact-inst-exist",
        ));
    }
    if conn
        .count_by_sql(
            &format!("SELECT 1 FROM {table_name} WHERE key = $1 AND rel_conf_fact_key = $2"),
            vec![Value::from(&add_req.key), Value::from(fact_conf_key)],
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
        if !stats_pg_conf_dim_serv::online(dim_rel_conf_dim_key, &conn, ctx).await? {
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
    ];
    if let Some(dim_rel_conf_dim_key) = &add_req.dim_rel_conf_dim_key {
        params.push(Value::from(dim_rel_conf_dim_key.to_string()));
        sql_fields.push("dim_rel_conf_dim_key");
    }
    if let Some(dim_multi_values) = add_req.dim_multi_values {
        params.push(Value::from(dim_multi_values));
        sql_fields.push("dim_multi_values");
    }
    if let Some(mes_data_type) = &add_req.mes_data_type {
        params.push(Value::from(mes_data_type.to_string()));
        sql_fields.push("mes_data_type");
    }
    if let Some(mes_frequency) = &add_req.mes_frequency {
        params.push(Value::from(mes_frequency.to_string()));
        sql_fields.push("mes_frequency");
    }
    if let Some(mes_act_by_dim_conf_keys) = &add_req.mes_act_by_dim_conf_keys {
        params.push(Value::from(mes_act_by_dim_conf_keys.clone()));
        sql_fields.push("mes_act_by_dim_conf_keys");
    }
    if let Some(rel_conf_fact_and_col_key) = &add_req.rel_conf_fact_and_col_key {
        params.push(Value::from(rel_conf_fact_and_col_key.to_string()));
        sql_fields.push("rel_conf_fact_and_col_key");
    }

    conn.execute_one(
        &format!(
            r#"INSERT INTO {table_name}
(key, show_name, kind, rel_conf_fact_key, remark {})
VALUES
($1, $2, $3, $4, $5 {})
"#,
            if sql_fields.is_empty() { "".to_string() } else { format!(",{}", sql_fields.join(",")) },
            if sql_fields.is_empty() {
                "".to_string()
            } else {
                format!(",{}", sql_fields.iter().enumerate().map(|(i, _)| format!("${}", i + 6)).collect::<Vec<String>>().join(","))
            }
        ),
        params,
    )
    .await?;
    conn.commit().await?;
    Ok(())
}

pub(crate) async fn modify(fact_conf_key: &str, fact_col_conf_key: &str, modify_req: &StatsConfFactColModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = stats_pg_initializer::init_conf_fact_col_table_and_conn(bs_inst, ctx, true).await?;
    conn.begin().await?;
    if stats_pg_conf_fact_serv::online(fact_conf_key, &conn, ctx).await? {
        return Err(funs.err().conflict(
            "fact_col_conf",
            "modify",
            "The fact instance table already exists, please delete it and then modify it.",
            "409-spi-stats-fact-inst-exist",
        ));
    }
    let mut sql_sets = vec![];
    let mut params = vec![Value::from(fact_col_conf_key.to_string()), Value::from(fact_conf_key.to_string())];
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
    if let Some(mes_data_type) = &modify_req.mes_data_type {
        sql_sets.push(format!("mes_data_type = ${}", params.len() + 1));
        params.push(Value::from(mes_data_type.to_string()));
    }
    if let Some(mes_frequency) = &modify_req.mes_frequency {
        sql_sets.push(format!("mes_frequency = ${}", params.len() + 1));
        params.push(Value::from(mes_frequency.to_string()));
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
    conn.execute_one(
        &format!(
            r#"UPDATE {table_name}
SET {}
WHERE key = $1 AND rel_conf_fact_key = $2
"#,
            sql_sets.join(",")
        ),
        params,
    )
    .await?;
    conn.commit().await?;
    Ok(())
}

pub(crate) async fn delete(fact_conf_key: &str, fact_col_conf_key: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = stats_pg_initializer::init_conf_fact_col_table_and_conn(bs_inst, ctx, true).await?;
    conn.begin().await?;
    if stats_pg_conf_fact_serv::online(fact_conf_key, &conn, ctx).await? {
        return Err(funs.err().conflict(
            "fact_col_conf",
            "delete",
            "The fact instance table already exists, please delete it and then modify it.",
            "409-spi-stats-fact-inst-exist",
        ));
    }
    conn.execute_one(
        &format!("DELETE FROM {table_name} WHERE key = $1 AND rel_conf_fact_key = $2"),
        vec![Value::from(fact_col_conf_key), Value::from(fact_conf_key)],
    )
    .await?;
    conn.commit().await?;
    Ok(())
}

pub(in crate::serv::pg) async fn find_by_fact_conf_key(fact_conf_key: &str, conn: &TardisRelDBlConnection, ctx: &TardisContext) -> TardisResult<Vec<StatsConfFactColInfoResp>> {
    if !common_pg::check_table_exit("stats_conf_fact_col", conn, ctx).await? {
        return Ok(vec![]);
    }
    do_paginate(fact_conf_key.to_string(), None, None, 1, u32::MAX, None, None, conn, ctx).await.map(|page| page.records)
}

pub(crate) async fn paginate(
    fact_conf_key: String,
    fact_col_conf_key: Option<String>,
    show_name: Option<String>,
    page_number: u32,
    page_size: u32,
    desc_by_create: Option<bool>,
    desc_by_update: Option<bool>,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<TardisPage<StatsConfFactColInfoResp>> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (conn, _) = stats_pg_initializer::init_conf_fact_col_table_and_conn(bs_inst, ctx, true).await?;

    do_paginate(
        fact_conf_key,
        fact_col_conf_key,
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
    fact_conf_key: String,
    fact_col_conf_key: Option<String>,
    show_name: Option<String>,
    page_number: u32,
    page_size: u32,
    desc_by_create: Option<bool>,
    desc_by_update: Option<bool>,
    conn: &TardisRelDBlConnection,
    ctx: &TardisContext,
) -> TardisResult<TardisPage<StatsConfFactColInfoResp>> {
    let table_name = package_table_name("stats_conf_fact_col", ctx);
    let mut sql_where = vec!["rel_conf_fact_key = $1".to_string()];
    let mut sql_order = vec![];
    let mut params: Vec<Value> = vec![Value::from(fact_conf_key), Value::from(page_size), Value::from((page_number - 1) * page_size)];
    if let Some(fact_col_conf_key) = &fact_col_conf_key {
        sql_where.push(format!("key = ${}", params.len() + 1));
        params.push(Value::from(fact_col_conf_key.to_string()));
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
                r#"SELECT key, show_name, kind, remark, dim_rel_conf_dim_key, dim_multi_values, mes_data_type, mes_frequency, mes_act_by_dim_conf_keys, rel_conf_fact_and_col_key, create_time, update_time, count(*) OVER() AS total
FROM {table_name}
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
                total_size = item.try_get("", "total").unwrap();
            }
            StatsConfFactColInfoResp {
                key: item.try_get("", "key").unwrap(),
                show_name: item.try_get("", "show_name").unwrap(),
                kind: item.try_get("", "kind").unwrap(),
                dim_rel_conf_dim_key: item.try_get("", "dim_rel_conf_dim_key").unwrap(),
                dim_multi_values: item.try_get("", "dim_multi_values").unwrap(),
                mes_data_type: if item.try_get::<Option<String>>("", "mes_data_type").unwrap().is_none() {
                    None
                } else {
                    Some(item.try_get("", "mes_data_type").unwrap())
                },
                mes_frequency: item.try_get("", "mes_frequency").unwrap(),
                mes_act_by_dim_conf_keys: item.try_get("", "mes_act_by_dim_conf_keys").unwrap(),
                rel_conf_fact_and_col_key: item.try_get("", "rel_conf_fact_and_col_key").unwrap(),
                remark: item.try_get("", "remark").unwrap(),
                create_time: item.try_get("", "create_time").unwrap(),
                update_time: item.try_get("", "update_time").unwrap(),
            }
        })
        .collect();
    Ok(TardisPage {
        page_size: page_size as u64,
        page_number: page_number as u64,
        total_size: total_size as u64,
        records: result,
    })
}
