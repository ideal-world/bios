use bios_basic::spi::{
    spi_funs::SpiBsInstExtractor,
    spi_initializer::common_pg::{self, package_table_name},
};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    db::{
        reldb_client::TardisRelDBClient,
        sea_orm::{self, Value},
    },
    TardisFunsInst,
};

use crate::{
    dto::stats_query_dto::{StatsQueryMetricsReq, StatsQueryMetricsResp},
    stats_enumeration::{StatsDataTypeKind, StatsFactColKind},
};

pub async fn query_metrics(query_req: &StatsQueryMetricsReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<StatsQueryMetricsResp>> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (conn, _) = common_pg::init_conn(bs_inst).await?;

    let fact_conf_table_name = package_table_name("stats_conf_fact", ctx);
    let fact_col_conf_table_name = package_table_name("stats_conf_fact_col", ctx);
    let dim_conf_table_name = package_table_name("stats_conf_dim", ctx);

    let conf_info: Vec<StatsConfInfo> = conn
        .find_dtos_by_sql(
            &format!(
                r#"SELECT
    col.key as col_key,
    col.kind as col_kind,
    col.dim_multi_values as dim_multi_values,
    col.mes_data_type as mes_data_type,
    col.mes_act_by_dim_conf_keys as mes_act_by_dim_conf_keys,
    dim.key as dim_key,
    dim.stable_ds as stable_ds
  FROM
    {fact_col_conf_table_name} col
    LEFT JOIN {fact_conf_table_name} fact ON fact.key = col.rel_conf_fact_key
    LEFT JOIN {dim_conf_table_name} dim ON dim.key = col.dim_rel_conf_dim_key
  WHERE
    fact.key = $1"#
            ),
            vec![Value::from(&query_req.from)],
        )
        .await?;

    //     let mut sql_where = vec!["rel_conf_fact_key = $1".to_string()];
    //     let mut sql_order = vec![];
    //     let mut params: Vec<Value> = vec![Value::from(fact_conf_key), Value::from(page_size), Value::from((page_number - 1) * page_size)];
    //     if let Some(fact_col_conf_key) = &fact_col_conf_key {
    //         sql_where.push(format!("key = ${}", params.len() + 1));
    //         params.push(Value::from(fact_col_conf_key.to_string()));
    //     }
    //     if let Some(show_name) = &show_name {
    //         sql_where.push(format!("show_name LIKE ${}", params.len() + 1));
    //         params.push(Value::from(format!("%{show_name}%")));
    //     }
    //     if let Some(desc_by_create) = desc_by_create {
    //         sql_order.push(format!("create_time {}", if desc_by_create { "DESC" } else { "ASC" }));
    //     }
    //     if let Some(desc_by_update) = desc_by_update {
    //         sql_order.push(format!("update_time {}", if desc_by_update { "DESC" } else { "ASC" }));
    //     }

    //     let result = conn
    //         .query_all(
    //             &format!(
    //                 r#"SELECT key, show_name, kind, remark, dim_rel_conf_dim_key, dim_multi_values, mes_data_type, mes_frequency, mes_act_by_dim_conf_keys, rel_conf_fact_and_col_key, create_time, update_time, count(*) OVER() AS total
    // FROM {table_name}
    // WHERE
    //     {}
    // LIMIT $2 OFFSET $3
    // {}"#,
    //                 sql_where.join(" AND "),
    //                 if sql_order.is_empty() {
    //                     "".to_string()
    //                 } else {
    //                     format!("ORDER BY {}", sql_order.join(","))
    //                 }
    //             ),
    //             params,
    //         )
    //         .await?;

    //     let mut total_size: i64 = 0;
    //     let result = result
    //         .into_iter()
    //         .map(|item| {
    //             if total_size == 0 {
    //                 total_size = item.try_get("", "total").unwrap();
    //             }
    //             StatsConfFactColInfoResp {
    //                 key: item.try_get("", "key").unwrap(),
    //                 show_name: item.try_get("", "show_name").unwrap(),
    //                 kind: item.try_get("", "kind").unwrap(),
    //                 dim_rel_conf_dim_key: item.try_get("", "dim_rel_conf_dim_key").unwrap(),
    //                 dim_multi_values: item.try_get("", "dim_multi_values").unwrap(),
    //                 mes_data_type: if item.try_get::<Option<String>>("", "mes_data_type").unwrap().is_none() {
    //                     None
    //                 } else {
    //                     Some(item.try_get("", "mes_data_type").unwrap())
    //                 },
    //                 mes_frequency: item.try_get("", "mes_frequency").unwrap(),
    //                 mes_act_by_dim_conf_keys: item.try_get("", "mes_act_by_dim_conf_keys").unwrap(),
    //                 rel_conf_fact_and_col_key: item.try_get("", "rel_conf_fact_and_col_key").unwrap(),
    //                 remark: item.try_get("", "remark").unwrap(),
    //                 create_time: item.try_get("", "create_time").unwrap(),
    //                 update_time: item.try_get("", "update_time").unwrap(),
    //             }
    //         })
    //         .collect();
    //     Ok(TardisPage {
    //         page_size: page_size as u64,
    //         page_number: page_number as u64,
    //         total_size: total_size as u64,
    //         records: result,
    //     })

    Ok(vec![])
}

#[derive(sea_orm::FromQueryResult)]
pub struct StatsConfInfo {
    pub col_key: String,
    pub col_kind: StatsFactColKind,
    pub dim_multi_values: Option<bool>,
    pub mes_data_type: Option<StatsDataTypeKind>,
    pub mes_act_by_dim_conf_keys: Option<Vec<String>>,
    pub dim_key: Option<bool>,
    pub stable_ds: Option<bool>,
}
