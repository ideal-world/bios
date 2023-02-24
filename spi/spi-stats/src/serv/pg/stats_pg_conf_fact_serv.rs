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

use crate::{
    dto::stats_conf_dto::{StatsConfFactAddReq, StatsConfFactColInfoResp, StatsConfFactInfoResp, StatsConfFactModifyReq},
    stats_enumeration::{StatsDataTypeKind, StatsFactColKind},
};

use super::{stats_pg_conf_dim_serv, stats_pg_conf_fact_col_serv, stats_pg_initializer};

pub async fn online(key: &str, conn: &TardisRelDBlConnection, ctx: &TardisContext) -> TardisResult<bool> {
    common_pg::check_table_exit(&format!("starsys_stats_inst_fact_{key}"), conn, ctx).await
}

pub(crate) async fn add(add_req: &StatsConfFactAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = stats_pg_initializer::init_conf_fact_table_and_conn(bs_inst, ctx, true).await?;
    conn.begin().await?;
    if conn.query_one(&format!("SELECT 1 FROM {table_name} WHERE key = $1"), vec![Value::from(&add_req.key)]).await?.is_some() {
        return Err(funs.err().conflict(
            "fact_conf",
            "add",
            "The fact config already exists, please delete it and then add it.",
            "409-spi-stats-fact-conf-exist",
        ));
    }
    if online(&add_req.key, &conn, ctx).await? {
        return Err(funs.err().conflict(
            "fact_conf",
            "add",
            "The fact instance table already exists, please delete it and then add it.",
            "409-spi-stats-fact-inst-exist",
        ));
    }
    let params = vec![
        Value::from(add_req.key.to_string()),
        Value::from(add_req.show_name.clone()),
        Value::from(add_req.query_limit),
        Value::from(add_req.remark.as_ref().unwrap_or(&"".to_string()).as_str()),
    ];

    conn.execute_one(
        &format!(
            r#"INSERT INTO {table_name}
(key, show_name, query_limit, remark)
VALUES
($1, $2, $3, $4)
"#,
        ),
        params,
    )
    .await?;
    conn.commit().await?;
    Ok(())
}

pub(crate) async fn modify(key: &str, modify_req: &StatsConfFactModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = stats_pg_initializer::init_conf_fact_table_and_conn(bs_inst, ctx, true).await?;
    conn.begin().await?;
    if online(key, &conn, ctx).await? {
        return Err(funs.err().conflict(
            "fact_conf",
            "modify",
            "The fact instance table already exists, please delete it and then modify it.",
            "409-spi-stats-fact-inst-exist",
        ));
    }
    let mut sql_sets = vec![];
    let mut params = vec![Value::from(key.to_string())];
    if let Some(show_name) = &modify_req.show_name {
        sql_sets.push(format!("show_name = ${}", params.len() + 1));
        params.push(Value::from(show_name.to_string()));
    }
    if let Some(query_limit) = modify_req.query_limit {
        sql_sets.push(format!("query_limit = ${}", params.len() + 1));
        params.push(Value::from(query_limit));
    }
    if let Some(remark) = &modify_req.remark {
        sql_sets.push(format!("remark = ${}", params.len() + 1));
        params.push(Value::from(remark.to_string()));
    }
    conn.execute_one(
        &format!(
            r#"UPDATE {table_name}
SET {}
WHERE key = $1
"#,
            sql_sets.join(",")
        ),
        params,
    )
    .await?;
    conn.commit().await?;
    Ok(())
}

pub(crate) async fn delete(key: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = stats_pg_initializer::init_conf_fact_table_and_conn(bs_inst, ctx, true).await?;
    conn.begin().await?;
    conn.execute_one(&format!("DELETE FROM {table_name} WHERE key = $1"), vec![Value::from(key)]).await?;
    conn.execute_one(
        &format!("DELETE FROM {} WHERE rel_conf_fact_key = $1", package_table_name("starsys_stats_conf_fact_col", ctx)),
        vec![Value::from(key)],
    )
    .await?;
    if online(key, &conn, ctx).await? {
        conn.execute_one(&format!("DROP TABLE {}{key}", package_table_name("starsys_stats_inst_fact_", ctx)), vec![]).await?;
        conn.execute_one(&format!("DROP TABLE {}{key}_del", package_table_name("starsys_stats_inst_fact_", ctx)), vec![]).await?;
    }
    conn.commit().await?;
    Ok(())
}

pub(in crate::serv::pg) async fn get(key: &str, conn: &TardisRelDBlConnection, ctx: &TardisContext) -> TardisResult<Option<StatsConfFactInfoResp>> {
    do_paginate(Some(key.to_string()), None, 1, 1, None, None, conn, ctx).await.map(|page| page.records.into_iter().next())
}

pub(crate) async fn paginate(
    key: Option<String>,
    show_name: Option<String>,
    page_number: u64,
    page_size: u64,
    desc_by_create: Option<bool>,
    desc_by_update: Option<bool>,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<TardisPage<StatsConfFactInfoResp>> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (conn, _) = stats_pg_initializer::init_conf_fact_table_and_conn(bs_inst, ctx, true).await?;

    do_paginate(key, show_name, page_number, page_size, desc_by_create, desc_by_update, &conn, ctx).await
}

async fn do_paginate(
    key: Option<String>,
    show_name: Option<String>,
    page_number: u64,
    page_size: u64,
    desc_by_create: Option<bool>,
    desc_by_update: Option<bool>,
    conn: &TardisRelDBlConnection,
    ctx: &TardisContext,
) -> TardisResult<TardisPage<StatsConfFactInfoResp>> {
    let table_name = package_table_name("starsys_stats_conf_fact", ctx);
    let mut sql_where = vec!["1 = 1".to_string()];
    let mut sql_order = vec![];
    let mut params: Vec<Value> = vec![Value::from(page_size), Value::from((page_number - 1) * page_size)];
    if let Some(key) = &key {
        sql_where.push(format!("key = ${}", params.len() + 1));
        params.push(Value::from(key.to_string()));
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
                r#"SELECT key, show_name, query_limit, remark, create_time, update_time, count(*) OVER() AS total
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
        final_result.push(StatsConfFactInfoResp {
            key: item.try_get("", "key").unwrap(),
            show_name: item.try_get("", "show_name").unwrap(),
            query_limit: item.try_get("", "query_limit").unwrap(),
            remark: item.try_get("", "remark").unwrap(),
            create_time: item.try_get("", "create_time").unwrap(),
            update_time: item.try_get("", "update_time").unwrap(),
            online: online(&item.try_get::<String>("", "key").unwrap(), &conn, ctx).await.unwrap(),
        });
    }
    Ok(TardisPage {
        page_size: page_size as u64,
        page_number: page_number as u64,
        total_size: total_size as u64,
        records: final_result,
    })
}

pub(crate) async fn create_inst(key: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, _) = common_pg::init_conn(bs_inst).await?;
    conn.begin().await?;

    let fact_conf = get(key, &conn, ctx).await?.ok_or(funs.err().not_found("fact_conf", "create_inst", "The fact config does not exist.", "404-spi-stats-fact-conf-not-exist"))?;

    let fact_col_conf = stats_pg_conf_fact_col_serv::find_by_fact_key(&fact_conf.key, &conn, ctx).await?;
    if fact_col_conf.len() == 0 {
        return Err(funs.err().not_found(
            "fact_conf",
            "create_inst",
            "The fact column config does not exist.",
            "404-spi-stats-fact-col-conf-not-exist",
        ));
    }

    if online(key, &conn, ctx).await? {
        return Err(funs.err().conflict(
            "fact_inst",
            "create_inst",
            "The fact instance table already exists, please delete it and then create it.",
            "409-spi-stats-fact-inst-exist",
        ));
    }
    create_inst_table(&fact_conf, &fact_col_conf, &conn, funs, ctx).await?;
    conn.commit().await?;
    Ok(())
}

async fn create_inst_table(
    fact_conf: &StatsConfFactInfoResp,
    fact_col_conf_set: &Vec<StatsConfFactColInfoResp>,
    conn: &TardisRelDBlConnection,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<()> {
    // Create fact inst table
    let mut sql = vec![];
    let mut index = vec![];
    sql.push("key character varying NOT NULL".to_string());
    sql.push("own_paths character varying NOT NULL".to_string());
    index.push(("own_paths".to_string(), "btree"));
    for fact_col_conf in fact_col_conf_set {
        if fact_col_conf.kind == StatsFactColKind::Dimension {
            let dim_conf_key = &fact_col_conf.dim_rel_conf_dim_key.clone().unwrap();
            if !stats_pg_conf_dim_serv::online(dim_conf_key, &conn, ctx).await? {
                return Err(funs.err().conflict(
                    "fact_inst",
                    "create",
                    &format!("The dimension config [{dim_conf_key}] not online."),
                    "409-spi-stats-dim-conf-not-online",
                ));
            }
            let dim_conf = stats_pg_conf_dim_serv::get(dim_conf_key, conn, ctx).await?.unwrap();
            if dim_conf.stable_ds {
                sql.push(format!("{} integer NOT NULL", &fact_col_conf.key));
                index.push((fact_col_conf.key.clone(), "btree"));
            } else if fact_col_conf.dim_multi_values.unwrap_or(false) {
                sql.push(format!("{} character varying[] NOT NULL", &fact_col_conf.key));
                index.push((fact_col_conf.key.clone(), "gin"));
            } else {
                match dim_conf.data_type {
                    StatsDataTypeKind::Number => {
                        sql.push(format!("{} integer NOT NULL", &fact_col_conf.key));
                        index.push((fact_col_conf.key.clone(), "btree"));
                    }
                    StatsDataTypeKind::Boolean => {
                        sql.push(format!("{} boolean NOT NULL", &fact_col_conf.key));
                        index.push((fact_col_conf.key.clone(), "btree"));
                    }
                    StatsDataTypeKind::DateTime => {
                        sql.push(format!("{} timestamp with time zone NOT NULL", &fact_col_conf.key));
                        index.push((fact_col_conf.key.clone(), "btree"));
                        index.push((format!("({}::date)", fact_col_conf.key), "btree"));
                        index.push((format!("date_part('hour',{})", fact_col_conf.key), "btree"));
                        index.push((format!("date_part('day',{})", fact_col_conf.key), "btree"));
                    }
                    StatsDataTypeKind::Date => {
                        sql.push(format!("{} date NOT NULL", &fact_col_conf.key));
                        index.push((fact_col_conf.key.clone(), "btree"));
                    }
                    StatsDataTypeKind::String => {
                        sql.push(format!("{} character varying NOT NULL", &fact_col_conf.key));
                        index.push((fact_col_conf.key.clone(), "btree"));
                    }
                }
            }
        } else if fact_col_conf.kind == StatsFactColKind::Measure {
            match fact_col_conf.mes_data_type {
                Some(StatsDataTypeKind::Number) => sql.push(format!("{} integer NOT NULL", &fact_col_conf.key)),
                Some(StatsDataTypeKind::Boolean) => sql.push(format!("{} boolean NOT NULL", &fact_col_conf.key)),
                Some(StatsDataTypeKind::DateTime) => sql.push(format!("{} timestamp with time zone NOT NULL", &fact_col_conf.key)),
                Some(StatsDataTypeKind::Date) => sql.push(format!("{} date NOT NULL", &fact_col_conf.key)),
                Some(StatsDataTypeKind::String) => sql.push(format!("{} character varying NOT NULL", &fact_col_conf.key)),
                None => sql.push(format!("{} integer NOT NULL", &fact_col_conf.key)),
            }
        } else {
            sql.push(format!("{} character varying", &fact_col_conf.key));
        }
    }
    sql.push("ct timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP".to_string());
    index.push(("ct".to_string(), "btree"));
    index.push(("(ct::date)".to_string(), "btree"));
    index.push(("date_part('hour',ct)".to_string(), "btree"));
    index.push(("date_part('day',ct)".to_string(), "btree"));

    let mut swap_index = vec![];
    for i in &index {
        swap_index.push((&i.0[..], i.1));
    }
    common_pg::init_table(conn, Some(&fact_conf.key), "stats_inst_fact", sql.join(",\r\n").as_str(), swap_index, None, ctx).await?;

    // Create fact inst delete status table
    common_pg::init_table(
        conn,
        Some(&format!("{}_del", fact_conf.key)),
        "stats_inst_fact",
        r#"key character varying NOT NULL,
    ct timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP"#,
        vec![],
        None,
        ctx,
    )
    .await?;

    Ok(())
}
