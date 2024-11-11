use crate::dto::stats_conf_dto::{StatsConfDimGroupAddReq, StatsConfDimGroupInfoResp, StatsConfDimGroupModifyReq};
use bios_basic::spi::{spi_funs::SpiBsInst, spi_initializer::common_pg::package_table_name};
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

pub(crate) async fn add(add_req: &StatsConfDimGroupAddReq, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = stats_pg_initializer::init_conf_dim_group_table_and_conn(bs_inst, ctx, true).await?;
    conn.begin().await?;

    if conn.count_by_sql(&format!("SELECT 1 FROM {table_name} WHERE key = $1"), vec![Value::from(&add_req.key)]).await? != 0 {
        return Err(funs.err().conflict(
            "dim_group",
            "add",
            "The dimension group already exists, please delete it and then add it.",
            "409-spi-stats-dim-group-exist",
        ));
    }

    let params = vec![
        Value::from(add_req.key.to_string()),
        Value::from(add_req.show_name.clone()),
        Value::from(add_req.data_type.to_string()),
        Value::from(add_req.remark.as_ref().unwrap_or(&"".to_string()).as_str()),
        Value::from(add_req.dynamic_url.as_deref()),
        Value::from(add_req.rel_attribute_code.as_ref().unwrap_or(&vec![]).clone()),
        Value::from(add_req.rel_attribute_url.as_deref()),
    ];

    conn.execute_one(
        &format!("INSERT INTO {table_name} (key, show_name, data_type, remark, dynamic_url, rel_attribute_code, rel_attribute_url) VALUES ($1, $2, $3, $4, $5, $6, $7)"),
        params,
    )
    .await?;

    conn.commit().await?;
    Ok(())
}

pub(crate) async fn modify(dim_conf_key: &str, modify_req: &StatsConfDimGroupModifyReq, _funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = stats_pg_initializer::init_conf_dim_group_table_and_conn(bs_inst, ctx, true).await?;
    conn.begin().await?;

    let mut sql_sets = vec![];
    let mut params = vec![Value::from(dim_conf_key.to_string())];

    if let Some(show_name) = &modify_req.show_name {
        sql_sets.push(format!("show_name = ${}", params.len() + 1));
        params.push(Value::from(show_name.to_string()));
    }
    if let Some(data_type) = &modify_req.data_type {
        sql_sets.push(format!("data_type = ${}", params.len() + 1));
        params.push(Value::from(data_type.to_string()));
    }
    if let Some(remark) = &modify_req.remark {
        sql_sets.push(format!("remark = ${}", params.len() + 1));
        params.push(Value::from(remark.to_string()));
    }
    if let Some(dynamic_url) = &modify_req.dynamic_url {
        sql_sets.push(format!("dynamic_url = ${}", params.len() + 1));
        params.push(Value::from(dynamic_url.to_string()));
    }
    if let Some(rel_attribute_code) = &modify_req.rel_attribute_code {
        sql_sets.push(format!("rel_attribute_code = ${}", params.len() + 1));
        params.push(Value::from(rel_attribute_code.clone()));
    }
    if let Some(rel_attribute_url) = &modify_req.rel_attribute_url {
        sql_sets.push(format!("rel_attribute_url = ${}", params.len() + 1));
        params.push(Value::from(rel_attribute_url.clone()));
    }

    conn.execute_one(&format!("UPDATE {table_name} SET {} WHERE key = $1", sql_sets.join(", ")), params).await?;
    conn.commit().await?;
    Ok(())
}

pub(crate) async fn paginate(
    page_number: u32,
    page_size: u32,
    desc_by_create: Option<bool>,
    desc_by_update: Option<bool>,
    _funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<TardisPage<StatsConfDimGroupInfoResp>> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, _) = stats_pg_initializer::init_conf_dim_group_table_and_conn(bs_inst, ctx, true).await?;
    conn.begin().await?;

    do_paginate(page_number, page_size, desc_by_create, desc_by_update, &conn, ctx, inst).await
}

async fn do_paginate(
    page_number: u32,
    page_size: u32,
    desc_by_create: Option<bool>,
    desc_by_update: Option<bool>,
    conn: &TardisRelDBlConnection,
    ctx: &TardisContext,
    _inst: &SpiBsInst,
) -> TardisResult<TardisPage<StatsConfDimGroupInfoResp>> {
    let table_name = package_table_name("stats_conf_dim_group", ctx);
    let sql_where = vec!["1 = 1".to_string()];
    let mut sql_order = vec![];
    let params: Vec<Value> = vec![Value::from(page_size), Value::from((page_number - 1) * page_size)];

    if let Some(desc_by_create) = desc_by_create {
        sql_order.push(format!("create_time {}", if desc_by_create { "DESC" } else { "ASC" }));
    }
    if let Some(desc_by_update) = desc_by_update {
        sql_order.push(format!("update_time {}", if desc_by_update { "DESC" } else { "ASC" }));
    }

    let result = conn
        .query_all(
            &format!(
                r#"SELECT key, show_name, data_type, remark, dynamic_url, rel_attribute_code, rel_attribute_url, create_time, update_time, count(*) OVER() AS total
FROM {table_name}
WHERE 
    {}
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
    let mut final_result = vec![];
    for item in result {
        if total_size == 0 {
            total_size = item.try_get("", "total")?;
        }
        final_result.push(StatsConfDimGroupInfoResp {
            key: item.try_get("", "key")?,
            show_name: item.try_get("", "show_name")?,
            data_type: item.try_get("", "data_type")?,
            remark: item.try_get("", "remark")?,
            dynamic_url: item.try_get("", "dynamic_url")?,
            rel_attribute_code: item.try_get("", "rel_attribute_code")?,
            rel_attribute_url: item.try_get("", "rel_attribute_url")?,
            create_time: item.try_get("", "create_time")?,
            update_time: item.try_get("", "update_time")?,
        });
    }
    Ok(TardisPage {
        page_size: page_size as u64,
        page_number: page_number as u64,
        total_size: total_size as u64,
        records: final_result,
    })
}
