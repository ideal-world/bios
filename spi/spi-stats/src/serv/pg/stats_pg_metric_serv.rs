use std::collections::HashMap;

use bios_basic::spi::{
    spi_funs::SpiBsInstExtractor,
    spi_initializer::common_pg::{self, package_table_name},
};
use itertools::Itertools;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    db::{
        reldb_client::TardisRelDBClient,
        sea_orm::{self, FromQueryResult, Value},
    },
    serde_json::{self, Map},
    TardisFunsInst,
};

use crate::{
    dto::stats_query_dto::{StatsQueryMetricsReq, StatsQueryMetricsResp},
    stats_enumeration::{StatsDataTypeKind, StatsFactColKind},
};

const FUNCTION_SUFFIX_FLAG: &str = "__";

pub async fn query_metrics(query_req: &StatsQueryMetricsReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<StatsQueryMetricsResp> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (conn, _) = common_pg::init_conn(bs_inst).await?;

    let fact_conf_table_name = package_table_name("stats_conf_fact", ctx);
    let fact_col_conf_table_name = package_table_name("stats_conf_fact_col", ctx);
    let dim_conf_table_name = package_table_name("stats_conf_dim", ctx);
    let fact_inst_table_name = package_table_name(&format!("stats_inst_fact_{}", query_req.from), ctx);
    let fact_inst_del_table_name = package_table_name(&format!("stats_inst_fact_{}_del", query_req.from), ctx);

    // Fetch config
    let conf_info = conn
        .query_all(
            &format!(
                r#"SELECT
    col.key as col_key,
    col.show_name as show_name,
    col.kind as col_kind,
    col.dim_multi_values as dim_multi_values,
    col.mes_data_type as mes_data_type,
    dim.data_type as dim_data_type,
    fact.query_limit as query_limit
  FROM
    {fact_col_conf_table_name} col
    LEFT JOIN {fact_conf_table_name} fact ON fact.key = col.rel_conf_fact_key
    LEFT JOIN {dim_conf_table_name} dim ON dim.key = col.dim_rel_conf_dim_key
  WHERE
    fact.key = $1
    AND col.kind != $2"#
            ),
            vec![Value::from(&query_req.from), Value::from(StatsFactColKind::Ext.to_string())],
        )
        .await?;
    if conf_info.is_empty() {
        return Err(funs.err().not_found(
            "metric",
            "query",
            &format!("The query fact [{}] does not exist.", query_req.from),
            "404-spi-stats-metric-fact-not-exist",
        ));
    }
    let conf_info = conf_info
        .into_iter()
        .map(|item| StatsConfInfo {
            col_key: item.try_get("", "col_key").unwrap(),
            show_name: item.try_get("", "show_name").unwrap(),
            col_kind: item.try_get("", "col_kind").unwrap(),
            dim_multi_values: item.try_get("", "dim_multi_values").unwrap(),
            mes_data_type: if item.try_get::<Option<String>>("", "mes_data_type").unwrap().is_none() {
                None
            } else {
                Some(item.try_get("", "mes_data_type").unwrap())
            },
            dim_data_type: if item.try_get::<Option<String>>("", "dim_data_type").unwrap().is_none() {
                None
            } else {
                Some(item.try_get("", "dim_data_type").unwrap())
            },
            query_limit: item.try_get("", "query_limit").unwrap(),
        })
        .collect::<Vec<StatsConfInfo>>();

    let conf_limit = conf_info.get(0).unwrap().query_limit;
    let conf_info = conf_info.into_iter().map(|v| (v.col_key.clone(), v)).collect::<HashMap<String, StatsConfInfo>>();
    if query_req.select.iter().any(|i| !conf_info.contains_key(&i.code))
        || query_req.group.iter().any(|i| !conf_info.contains_key(&i.code) || conf_info.get(&i.code).unwrap().col_kind != StatsFactColKind::Dimension)
        || query_req
            .order
            .as_ref()
            .map(|orders| orders.iter().any(|i| !conf_info.contains_key(&i.code) || conf_info.get(&i.code).unwrap().col_kind != StatsFactColKind::Dimension))
            .unwrap_or(false)
        || query_req.having.as_ref().map(|orders| orders.iter().any(|i| !conf_info.contains_key(&i.code))).unwrap_or(false)
        || query_req._where.as_ref().map(|or_wheres| or_wheres.iter().any(|and_wheres| and_wheres.iter().any(|where_| !conf_info.contains_key(&where_.code)))).unwrap_or(false)
    {
        return Err(funs.err().not_found(
            "metric",
            "query",
            &format!("The query some dimension does not exist."),
            "404-spi-stats-metric-dim-not-exist",
        ));
    }

    let mut params = vec![
        Value::from(format!("{}%", ctx.own_paths)),
        Value::from(query_req.start_time),
        Value::from(query_req.end_time),
    ];

    // Package filter
    let mut sql_part_wheres = vec![];
    if let Some(wheres) = &query_req._where {
        let mut sql_part_or_wheres = vec![];
        for or_wheres in wheres {
            let mut sql_part_and_wheres = vec![];
            for and_where in or_wheres {
                let col_conf = conf_info.get(&and_where.code).unwrap();
                let col_data_type = if &col_conf.col_kind == &StatsFactColKind::Dimension {
                    col_conf.dim_data_type.as_ref().unwrap()
                } else {
                    col_conf.mes_data_type.as_ref().unwrap()
                };
                if let Some((sql_part, value)) = col_data_type.to_pg_where(
                    col_conf.dim_multi_values.unwrap(),
                    &format!("fact.{}", &and_where.code),
                    &and_where.op,
                    params.len() + 1,
                    &and_where.value,
                    &and_where.time_window,
                ) {
                    params.push(value);
                    sql_part_and_wheres.push(sql_part);
                } else {
                    return Err(funs.err().not_found(
                        "metric",
                        "query",
                        &format!(
                            "The query column=[{}] operation=[{}] time_window=[{}] multi_values=[{}] is not legal.",
                            &and_where.code,
                            &and_where.op.to_sql(),
                            &and_where.time_window.is_some(),
                            col_conf.dim_multi_values.unwrap()
                        ),
                        "404-spi-stats-metric-op-not-legal",
                    ));
                }
            }
            sql_part_or_wheres.push(sql_part_and_wheres.join(" AND "));
        }
        sql_part_wheres.push(format!("( {} )", sql_part_or_wheres.join(" OR ")));
    }
    let sql_part_wheres = if sql_part_wheres.is_empty() {
        "".to_string()
    } else {
        format!(" AND {}", sql_part_wheres.join(" AND "))
    };

    // Package inner select
    // Add measures
    let mut sql_part_inner_selects = vec![];
    for select in &query_req.select {
        sql_part_inner_selects.push(format!("fact.{} AS {}", &select.code, &select.code));
    }
    for group in &query_req.group {
        sql_part_inner_selects.push(format!("fact.{} AS {}", &group.code, &group.code));
    }
    let sql_part_inner_selects = sql_part_inner_selects.join(",");

    // Package group
    // (column name with fun, alias name, show name)
    let mut sql_part_group_infos = vec![];
    for group in &query_req.group {
        let col_conf = conf_info.get(&group.code).unwrap();
        let col_data_type = col_conf.dim_data_type.as_ref().unwrap();
        if let Some(column_name_with_fun) = col_data_type.to_pg_group(&format!("_.{}", &group.code), &group.time_window) {
            let alias_name = format!(
                "{}{FUNCTION_SUFFIX_FLAG}{}",
                group.code,
                group.time_window.as_ref().map(|i| i.to_string()).unwrap_or("".to_string())
            );
            sql_part_group_infos.push((column_name_with_fun, alias_name, col_conf.show_name.clone()));
        } else {
            return Err(funs.err().not_found(
                "metric",
                "query",
                &format!("The group column=[{}] time_window=[{}] is not legal.", &group.code, &group.time_window.is_some(),),
                "404-spi-stats-metric-op-not-legal",
            ));
        }
    }
    let sql_part_groups = sql_part_group_infos.iter().map(|group| group.0.clone()).collect::<Vec<String>>().join(",");

    // Package outer select
    // (column name with fun, alias name, show_name, is dimension)
    let mut sql_part_outer_select_infos = vec![];
    for (column_name_with_fun, alias_name, show_name) in sql_part_group_infos {
        sql_part_outer_select_infos.push((column_name_with_fun, alias_name, show_name, true));
    }
    for select in &query_req.select {
        let col_conf = conf_info.get(&select.code).unwrap();
        let col_data_type = col_conf.mes_data_type.as_ref().unwrap();
        let column_name_with_fun = col_data_type.to_pg_select(&format!("_.{}", &select.code), &select.fun);
        let alias_name = format!("{}{FUNCTION_SUFFIX_FLAG}{}", select.code, select.fun.to_string().to_lowercase());
        sql_part_outer_select_infos.push((column_name_with_fun, alias_name, col_conf.show_name.clone(), false));
    }
    let sql_part_outer_selects =
        sql_part_outer_select_infos.iter().map(|(column_name_with_fun, alias_name, _, _)| format!("{column_name_with_fun} AS {alias_name}")).collect::<Vec<String>>().join(",");

    // Package having
    let sql_part_havings = if let Some(havings) = &query_req.having {
        let mut sql_part_havings = vec![];
        for having in havings {
            let col_conf = conf_info.get(&having.code).unwrap();
            let col_data_type = if &col_conf.col_kind == &StatsFactColKind::Dimension {
                col_conf.dim_data_type.as_ref().unwrap()
            } else {
                col_conf.mes_data_type.as_ref().unwrap()
            };
            if let Some((sql_part, value)) = col_data_type.to_pg_having(
                col_conf.dim_multi_values.unwrap(),
                &format!("_.{}", &having.code),
                &having.op,
                params.len() + 1,
                &having.value,
                &having.fun,
            ) {
                params.push(value);
                sql_part_havings.push(sql_part);
            } else {
                return Err(funs.err().not_found(
                    "metric",
                    "query",
                    &format!(
                        "The query column=[{}] operation=[{}] fun=[{}] multi_values=[{}] is not legal.",
                        &having.code,
                        &having.op.to_sql(),
                        &having.fun.is_some(),
                        col_conf.dim_multi_values.unwrap()
                    ),
                    "404-spi-stats-metric-op-not-legal",
                ));
            }
        }
        format!("HAVING {}", sql_part_havings.join(","))
    } else {
        "".to_string()
    };

    // Package order
    let sql_part_orders = if let Some(orders) = &query_req.order {
        let sql_part_orders = orders
            .iter()
            .map(|order| {
                format!(
                    "{}_{} {}",
                    order.code,
                    order.fun.as_ref().map(|i| i.to_string()).unwrap_or("".to_string()),
                    if order.asc { "ASC" } else { "DESC" }
                )
            })
            .collect::<Vec<String>>();
        format!("ORDER BY {}", sql_part_orders.join(","))
    } else {
        "".to_string()
    };

    // package limit
    let query_limit = if let Some(limit) = &query_req.limit {
        format!("LIMIT {}", limit)
    } else {
        "".to_string()
    };

    let final_sql = format!(
        r#"SELECT {sql_part_outer_selects}
    FROM (
        SELECT
            DISTINCT ON (fact.key) fact.key, {sql_part_inner_selects}
        FROM {fact_inst_table_name} fact
        LEFT JOIN {fact_inst_del_table_name} del ON del.key = fact.key AND del.ct >= $2 and del.ct <= $3
        WHERE
            fact.own_paths LIKE $1
            AND del.key IS NULL
            AND fact.ct >= $2 and fact.ct <= $3
            {sql_part_wheres}
        ORDER BY fact.key,fact.ct DESC
        LIMIT {conf_limit}
    ) _
    GROUP BY ROLLUP({sql_part_groups})
    {sql_part_havings}
    {sql_part_orders}
    {query_limit}"#
    );

    let result = conn.query_all(&final_sql, params).await?.iter().map(|record| serde_json::Value::from_query_result(record, "").unwrap()).collect::<Vec<serde_json::Value>>();

    let select_dimension_keys =
        sql_part_outer_select_infos.iter().filter(|(_, _, _, is_dimension)| *is_dimension).map(|(_, alias_name, _, _)| alias_name.to_string()).collect::<Vec<String>>();
    let select_measure_keys =
        sql_part_outer_select_infos.iter().filter(|(_, _, _, is_dimension)| !*is_dimension).map(|(_, alias_name, _, _)| alias_name.to_string()).collect::<Vec<String>>();
    let show_names = sql_part_outer_select_infos.into_iter().map(|(_, alias_name, show_name, _)| (alias_name, show_name)).collect::<HashMap<String, String>>();

    println!(">>>>>>>>>>>>{:?}", tardis::TardisFuns::json.obj_to_string(&result));

    Ok(StatsQueryMetricsResp {
        from: query_req.from.to_string(),
        show_names,
        group: package_groups(select_dimension_keys, &select_measure_keys, result),
    })
}

fn package_groups(curr_select_dimension_keys: Vec<String>, select_measure_keys: &Vec<String>, result: Vec<serde_json::Value>) -> serde_json::Value {
    if curr_select_dimension_keys.len() == 0 {
        let mut leaf_node = Map::new();
        select_measure_keys.iter().for_each(|measure_key| {
            let val = result.get(0).unwrap().get(measure_key).unwrap();
            let val = if measure_key.ends_with(&format!("{FUNCTION_SUFFIX_FLAG}avg")) {
                // Fix avg function return type error
                serde_json::Value::from(val.as_str().unwrap().parse::<f64>().unwrap())
            } else {
                val.clone()
            };
            leaf_node.insert(measure_key.to_string(), val);
        });
        return serde_json::Value::Object(leaf_node);
    }
    let mut node = Map::new();
    result
        .iter()
        .group_by(|record| {
            let dimension_key = curr_select_dimension_keys.get(0).unwrap();
            println!(">>>>>>>1>>>>>{:?}", dimension_key);
            println!(">>>>>>>2>>>>>{:?}", record);
            record.get(dimension_key).unwrap()
        })
        .into_iter()
        .for_each(|(key, group)| {
            let key = if key.is_null() { "".to_string() } else { key.as_str().unwrap().to_string() };
            let sub = package_groups(
                curr_select_dimension_keys[1..].to_vec(),
                select_measure_keys,
                group.into_iter().map(|r| r.clone()).collect::<Vec<serde_json::Value>>(),
            );
            node.insert(key, sub);
        });
    serde_json::Value::Object(node)
}

#[derive(sea_orm::FromQueryResult)]
struct StatsConfInfo {
    pub col_key: String,
    pub show_name: String,
    pub col_kind: StatsFactColKind,
    pub dim_multi_values: Option<bool>,
    pub mes_data_type: Option<StatsDataTypeKind>,
    pub dim_data_type: Option<StatsDataTypeKind>,
    pub query_limit: i32,
}
