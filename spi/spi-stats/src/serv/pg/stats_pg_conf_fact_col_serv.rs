use bios_basic::spi::spi_funs::SpiBsInstExtractor;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    db::{reldb_client::TardisRelDBClient, sea_orm::Value},
    web::web_resp::TardisPage,
    TardisFunsInst,
};

use crate::dto::stats_conf_dto::{StatsConfFactColAddReq, StatsConfFactColInfoResp, StatsConfFactColModifyReq};

use super::stats_pg_initializer;

pub(crate) async fn add(rel_conf_fact_key: &str, add_req: &StatsConfFactColAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let conn = stats_pg_initializer::init_conf_fact_col_table_and_conn(bs_inst, ctx, true).await?;
    if conn
        .query_one(
            "SELECT 1 starsys_stats_conf_fact_col WHERE key = $1 AND rel_conf_fact_key = $2",
            vec![Value::from(&add_req.key), Value::from(rel_conf_fact_key)],
        )
        .await?
        .is_some()
    {
        return Err(funs.err().conflict(
            "fact_col_conf",
            "add",
            "The fact column config already exists, please delete it and then add it.",
            "409-spi-stats-fact-conf-col-exist",
        ));
    }
    let mut sql_fields = vec![];
    let mut params = vec![
        Value::from(add_req.key.to_string()),
        Value::from(add_req.show_name.clone()),
        Value::from(add_req.kind.to_string()),
        Value::from(rel_conf_fact_key.to_string()),
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
    if let Some(dim_exclusive_rec) = add_req.dim_exclusive_rec {
        params.push(Value::from(dim_exclusive_rec));
        sql_fields.push("dim_exclusive_rec");
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
            r#"INSERT INTO starsys_stats_conf_fact_col
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

pub(crate) async fn modify(key: &str, modify_req: &StatsConfFactColModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let conn = stats_pg_initializer::init_conf_fact_col_table_and_conn(bs_inst, ctx, true).await?;
    let mut sql_sets = vec![];
    let mut params = vec![Value::from(key.to_string())];
    if let Some(show_name) = &modify_req.show_name {
        sql_sets.push(format!("show_name = ${}", sql_sets.len() + 2));
        params.push(Value::from(show_name.to_string()));
    }
    if let Some(kind) = &modify_req.kind {
        sql_sets.push(format!("kind = ${}", sql_sets.len() + 2));
        params.push(Value::from(kind.to_string()));
    }
    if let Some(dim_rel_conf_dim_key) = &modify_req.dim_rel_conf_dim_key {
        sql_sets.push(format!("dim_rel_conf_dim_key = ${}", sql_sets.len() + 2));
        params.push(Value::from(dim_rel_conf_dim_key.to_string()));
    }
    if let Some(dim_multi_values) = modify_req.dim_multi_values {
        sql_sets.push(format!("dim_multi_values = ${}", sql_sets.len() + 2));
        params.push(Value::from(dim_multi_values));
    }
    if let Some(dim_exclusive_rec) = modify_req.dim_exclusive_rec {
        sql_sets.push(format!("dim_exclusive_rec = ${}", sql_sets.len() + 2));
        params.push(Value::from(dim_exclusive_rec));
    }
    if let Some(mes_data_type) = &modify_req.mes_data_type {
        sql_sets.push(format!("mes_data_type = ${}", sql_sets.len() + 2));
        params.push(Value::from(mes_data_type.to_string()));
    }
    if let Some(mes_frequency) = &modify_req.mes_frequency {
        sql_sets.push(format!("mes_frequency = ${}", sql_sets.len() + 2));
        params.push(Value::from(mes_frequency.to_string()));
    }
    if let Some(mes_act_by_dim_conf_keys) = &modify_req.mes_act_by_dim_conf_keys {
        sql_sets.push(format!("mes_act_by_dim_conf_keys = ${}", sql_sets.len() + 2));
        params.push(Value::from(mes_act_by_dim_conf_keys.clone()));
    }
    if let Some(rel_conf_fact_and_col_key) = &modify_req.rel_conf_fact_and_col_key {
        sql_sets.push(format!("rel_conf_fact_and_col_key = ${}", sql_sets.len() + 2));
        params.push(Value::from(rel_conf_fact_and_col_key.to_string()));
    }

    conn.execute_one(
        &format!(
            r#"UPDATE starsys_stats_conf_fact_col
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
    let conn = stats_pg_initializer::init_conf_fact_col_table_and_conn(bs_inst, ctx, true).await?;
    conn.execute_one("DELETE FROM starsys_stats_conf_fact_col WHERE key = $1", vec![Value::from(key)]).await?;
    conn.commit().await?;
    Ok(())
}

pub(crate) async fn paginate(
    key: Option<String>,
    show_name: Option<String>,
    rel_conf_fact_key: Option<String>,
    page_number: u64,
    page_size: u64,
    desc_by_create: Option<bool>,
    desc_by_update: Option<bool>,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<TardisPage<StatsConfFactColInfoResp>> {
    let mut sql_where = vec![];
    let mut sql_order = vec![];
    let mut params: Vec<Value> = vec![];
    if let Some(key) = &key {
        sql_where.push(format!("key = ${}", sql_where.len() + 1));
        params.push(Value::from(key.to_string()));
    }
    if let Some(show_name) = &show_name {
        sql_where.push(format!("show_name LIKE ${}", sql_where.len() + 1));
        params.push(Value::from(format!("%{show_name}%")));
    }
    if let Some(rel_conf_fact_key) = &rel_conf_fact_key {
        sql_where.push(format!("rel_conf_fact_key = ${}", sql_where.len() + 1));
        params.push(Value::from(rel_conf_fact_key.to_string()));
    }
    params.push(Value::from(page_size));
    params.push(Value::from((page_number - 1) * page_size));
    if let Some(desc_by_create) = desc_by_create {
        sql_order.push(format!("create_time {}", if desc_by_create { "DESC" } else { "ASC" }));
    }
    if let Some(desc_by_update) = desc_by_update {
        sql_order.push(format!("update_time {}", if desc_by_update { "DESC" } else { "ASC" }));
    }

    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let conn = stats_pg_initializer::init_conf_fact_col_table_and_conn(bs_inst, ctx, true).await?;
    let result = conn
        .query_all(
            &format!(
                r#"SELECT key, show_name, kind, remark, dim_rel_conf_dim_key, dim_multi_values, dim_exclusive_rec, mes_data_type, mes_frequency, mes_act_by_dim_conf_keys, rel_conf_fact_and_col_key, update_time, count(*) OVER() AS total
FROM starsys_stats_conf_fact
WHERE 
    {}
LIMIT $2 OFFSET $3
{}"#,
                sql_where.join(","),
                if sql_order.is_empty() {
                    "".to_string()
                } else {
                    format!("ORDER BY {}", sql_order.join(","))
                }
            ),
            params,
        )
        .await?;
    conn.commit().await?;

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
                dim_exclusive_rec: item.try_get("", "dim_exclusive_rec").unwrap(),
                mes_data_type: item.try_get("", "mes_data_type").unwrap(),
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
