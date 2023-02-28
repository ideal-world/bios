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

use super::stats_pg_initializer;
use crate::{
    dto::stats_conf_dto::{StatsConfDimAddReq, StatsConfDimInfoResp, StatsConfDimModifyReq},
    stats_enumeration::StatsDataTypeKind,
};

pub async fn online(dim_conf_key: &str, conn: &TardisRelDBlConnection, ctx: &TardisContext) -> TardisResult<bool> {
    common_pg::check_table_exit(&format!("stats_inst_dim_{dim_conf_key}"), conn, ctx).await
}

pub(crate) async fn add(add_req: &StatsConfDimAddReq, funs: &&TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = stats_pg_initializer::init_conf_dim_table_and_conn(bs_inst, ctx, true).await?;
    conn.begin().await?;
    if conn.count_by_sql(&format!("SELECT 1 FROM {table_name} WHERE key = $1"), vec![Value::from(&add_req.key)]).await? != 0 {
        return Err(funs.err().conflict(
            "dim_conf",
            "add",
            "The dimension config already exists, please delete it and then add it.",
            "409-spi-stats-dim-conf-exist",
        ));
    }
    let params = vec![
        Value::from(add_req.key.to_string()),
        Value::from(add_req.show_name.clone()),
        Value::from(add_req.stable_ds),
        Value::from(add_req.data_type.to_string()),
        Value::from(add_req.hierarchy.as_ref().unwrap_or(&vec![]).clone()),
        Value::from(add_req.remark.as_ref().unwrap_or(&"".to_string()).as_str()),
    ];

    conn.execute_one(
        &format!(
            r#"INSERT INTO {table_name}
(key, show_name, stable_ds, data_type, hierarchy, remark)
VALUES
($1, $2, $3, $4, $5, $6)
"#,
        ),
        params,
    )
    .await?;
    conn.commit().await?;
    Ok(())
}

pub(crate) async fn modify(dim_conf_key: &str, modify_req: &StatsConfDimModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = stats_pg_initializer::init_conf_dim_table_and_conn(bs_inst, ctx, true).await?;
    conn.begin().await?;
    if online(dim_conf_key, &conn, ctx).await? {
        return Err(funs.err().conflict(
            "dim_conf",
            "modify",
            "The dimension instance table already exists, please delete it and then modify it.",
            "409-spi-stats-dim-inst-exist",
        ));
    }
    let mut sql_sets = vec![];
    let mut params = vec![Value::from(dim_conf_key.to_string())];
    if let Some(show_name) = &modify_req.show_name {
        sql_sets.push(format!("show_name = ${}", params.len() + 1));
        params.push(Value::from(show_name.to_string()));
    }
    if let Some(stable_ds) = modify_req.stable_ds {
        sql_sets.push(format!("stable_ds = ${}", params.len() + 1));
        params.push(Value::from(stable_ds));
    }
    if let Some(data_type) = &modify_req.data_type {
        sql_sets.push(format!("data_type = ${}", params.len() + 1));
        params.push(Value::from(data_type.to_string()));
    }
    if let Some(hierarchy) = &modify_req.hierarchy {
        sql_sets.push(format!("hierarchy = ${}", params.len() + 1));
        params.push(Value::from(hierarchy.clone()));
    }
    if let Some(remark) = &modify_req.remark {
        sql_sets.push(format!("remark = ${}", params.len() + 1));
        params.push(Value::from(remark.to_string()));
    }
    conn.execute_one(
        &format!(
            r#"UPDATE {table_name}
SET {}
WHERE key = $1"#,
            sql_sets.join(",")
        ),
        params,
    )
    .await?;
    conn.commit().await?;
    Ok(())
}

pub(crate) async fn delete(dim_conf_key: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = stats_pg_initializer::init_conf_dim_table_and_conn(bs_inst, ctx, true).await?;
    conn.begin().await?;
    conn.execute_one(&format!("DELETE FROM {table_name} WHERE key = $1"), vec![Value::from(dim_conf_key)]).await?;
    if online(dim_conf_key, &conn, ctx).await? {
        conn.execute_one(&format!("DROP TABLE {}_{dim_conf_key}", package_table_name("stats_inst_dim", ctx)), vec![]).await?;
    }
    conn.commit().await?;
    Ok(())
}

pub(in crate::serv::pg) async fn get(dim_conf_key: &str, conn: &TardisRelDBlConnection, ctx: &TardisContext) -> TardisResult<Option<StatsConfDimInfoResp>> {
    do_paginate(Some(dim_conf_key.to_string()), None, 1, 1, None, None, conn, ctx).await.map(|page| page.records.into_iter().next())
}

pub(crate) async fn paginate(
    dim_conf_key: Option<String>,
    show_name: Option<String>,
    page_number: u32,
    page_size: u32,
    desc_by_create: Option<bool>,
    desc_by_update: Option<bool>,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<TardisPage<StatsConfDimInfoResp>> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (conn, _) = stats_pg_initializer::init_conf_dim_table_and_conn(bs_inst, ctx, true).await?;
    do_paginate(dim_conf_key, show_name, page_number, page_size, desc_by_create, desc_by_update, &conn, ctx).await
}

async fn do_paginate(
    dim_conf_key: Option<String>,
    show_name: Option<String>,
    page_number: u32,
    page_size: u32,
    desc_by_create: Option<bool>,
    desc_by_update: Option<bool>,
    conn: &TardisRelDBlConnection,
    ctx: &TardisContext,
) -> TardisResult<TardisPage<StatsConfDimInfoResp>> {
    let table_name = package_table_name("stats_conf_dim", ctx);
    let mut sql_where = vec!["1 = 1".to_string()];
    let mut sql_order = vec![];
    let mut params: Vec<Value> = vec![Value::from(page_size), Value::from((page_number - 1) * page_size)];
    if let Some(dim_conf_key) = &dim_conf_key {
        sql_where.push(format!("key = ${}", params.len() + 1));
        params.push(Value::from(dim_conf_key.to_string()));
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
                r#"SELECT key, show_name, stable_ds, data_type, hierarchy, remark, create_time, update_time, count(*) OVER() AS total
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
        final_result.push(StatsConfDimInfoResp {
            key: item.try_get("", "key").unwrap(),
            show_name: item.try_get("", "show_name").unwrap(),
            stable_ds: item.try_get("", "stable_ds").unwrap(),
            data_type: item.try_get("", "data_type").unwrap(),
            hierarchy: item.try_get("", "hierarchy").unwrap(),
            remark: item.try_get("", "remark").unwrap(),
            create_time: item.try_get("", "create_time").unwrap(),
            update_time: item.try_get("", "update_time").unwrap(),
            online: online(&item.try_get::<String>("", "key").unwrap(), conn, ctx).await.unwrap(),
        });
    }
    Ok(TardisPage {
        page_size: page_size as u64,
        page_number: page_number as u64,
        total_size: total_size as u64,
        records: final_result,
    })
}

pub(crate) async fn create_inst(dim_conf_key: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, _) = common_pg::init_conn(bs_inst).await?;
    conn.begin().await?;

    let dim_conf = get(dim_conf_key, &conn, ctx)
        .await?
        .ok_or_else(|| funs.err().not_found("dim_fact", "create_inst", "The dimension config does not exist.", "404-spi-stats-dim-conf-not-exist"))?;

    if online(dim_conf_key, &conn, ctx).await? {
        return Err(funs.err().conflict(
            "dim_inst",
            "create_inst",
            "The dimension instance table already exists, please delete it and then create it.",
            "409-spi-stats-dim-inst-exist",
        ));
    }
    create_inst_table(&dim_conf, &conn, ctx).await?;
    conn.commit().await?;
    Ok(())
}

async fn create_inst_table(dim_conf: &StatsConfDimInfoResp, conn: &TardisRelDBlConnection, ctx: &TardisContext) -> TardisResult<()> {
    let mut sql = vec![];
    let mut index = vec![];
    sql.push("id serial PRIMARY KEY".to_string());
    sql.push(format!("key {} NOT NULL", dim_conf.data_type.to_pg_data_type()));
    index.push(("key", "btree"));
    match dim_conf.data_type {
        StatsDataTypeKind::DateTime => {
            index.push(("date(timezone('UTC', key))", "btree"));
            index.push(("date_part('hour',timezone('UTC', key))", "btree"));
            index.push(("date_part('day',timezone('UTC', key))", "btree"));
        }
        _ => {}
    }
    sql.push("show_name character varying NOT NULL".to_string());
    if !dim_conf.hierarchy.is_empty() {
        sql.push("hierarchy smallint NOT NULL".to_string());
    }
    for hierarchy in 0..dim_conf.hierarchy.len() {
        sql.push(format!("key{hierarchy} character varying NOT NULL DEFAULT ''"));
    }
    sql.push("ct timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP".to_string());
    sql.push("et timestamp with time zone".to_string());

    common_pg::init_table(conn, Some(&dim_conf.key), "stats_inst_dim", sql.join(",\r\n").as_str(), vec![], None, ctx).await?;
    Ok(())
}
