use std::collections::HashMap;

use bios_basic::spi::{
    spi_funs::SpiBsInst,
    spi_initializer::common_pg::{self, package_table_name},
};

use itertools::Itertools;
use tardis::{
    basic::{dto::TardisContext, error::TardisError, result::TardisResult},
    config::config_dto::log,
    db::{
        reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
        sea_orm::{self, FromQueryResult, Value},
    },
    log::info,
    serde_json::{self, json, Map},
    web::poem_openapi::types::{ToJSON, Type},
    TardisFunsInst,
};

use crate::{
    dto::stats_query_dto::{StatsQueryMetricsReq, StatsQueryMetricsResp, StatsQueryStatementReq, StatsQueryStatementResp},
    serv::stats_record_serv::dim_record_paginate,
    stats_enumeration::{StatsDataTypeKind, StatsFactColKind},
};

const FUNCTION_SUFFIX_FLAG: &str = "__";

/// 查询指标.
///
/// # Examples
///
/// An example SQL assembled from a completed query is as follows:
/// ```
/// -- Outer SQL, responsible for grouping and sorting, filtering, and length limitation after grouping
/// SELECT
///  -- The format of the returned field is: `field name__<function name>`
///   date(timezone('UTC', _.ct)) AS ct__date,
///   _.status AS status__,
///   sum(_.act_hours) AS act_hours__sum,
///   sum(_.plan_hours) AS plan_hours__sum
/// FROM
///   (
///     -- Inner SQL, responsible for filtering, deduplication, and length limitation
///     SELECT
///       -- Query dimensions and measures
///       fact.act_hours AS act_hours,
///       fact.plan_hours AS plan_hours,
///       fact.ct AS ct,
///       fact.status AS status
///     FROM
///         (
///             SELECT
///                 -- Built-in statement for deduplication
///                 DISTINCT ON (fact.key) fact.key,fact.*
///             FROM
///             -- Query fact instance table
///             xxx.starsys_stats_inst_fact_req fact
///             -- Association instance delete table
///             LEFT JOIN xxx.starsys_stats_inst_fact_req_del del ON del.key = fact.key
///             AND del.ct >= '2023-01-01 12:00:00 +00:00'
///             AND del.ct <= '2023-02-01 12:00:00 +00:00'
///             WHERE
///                 -- Built-in statement for permission control
///                 fact.own_paths LIKE 't1/a1%'
///                 -- Built-in statement for filter deleted records
///                 AND del.key IS NULL
///                  -- Time filter
///                 AND fact.ct >= '2023-01-01 12:00:00 +00:00'
///                 AND fact.ct <= '2023-02-01 12:00:00 +00:00'
///                 ORDER BY
///                 -- Built-in statement for deduplication
///                 fact.key, fact.ct DESC
///         ) fact
///       -- Other filter conditions, optional
///       AND (
///         fact.act_hours > 10
///         AND date_part('day', timezone('UTC', fact.ct)) != 1
///         OR fact.status = 'open'
///       )
///     ORDER BY
///         -- Order of the dimension values
///         fact.status DESC
///     LIMIT
///     -- built-in statement, the value is the limit length in the fact configuration
///       2000
///   ) _
/// GROUP BY
/// -- List of dimensions to grouping
/// -- Omit adding rollup dimensions according to the ignore_group_rollup configuration, Default is false
///   ROLLUP(date(timezone('UTC', _.ct)), _.status)
/// HAVING
/// -- Filter condition of the measure value after grouping and aggregation, optional
///   sum(_.act_hours) > 30
/// ORDER BY
/// -- Order of the measure and group values ​​after grouping ad aggregation, optional
///   act_hours__sum DESC
/// LIMIT
/// -- Length limit after grouping, optional
///   2
/// ```
pub async fn query_metrics(query_req: &StatsQueryMetricsReq, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<StatsQueryMetricsResp> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
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
    col.rel_external_id as rel_external_id,
    col.dim_rel_conf_dim_key as dim_rel_conf_dim_key,
    col.dim_multi_values as dim_multi_values,
    col.mes_data_distinct as mes_data_distinct,
    col.mes_data_type as mes_data_type,
    col.mes_unit as mes_unit,
    dim.data_type as dim_data_type,
    dim.hierarchy as dim_hierarchy,
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

    let mut conf_info = conf_info
        .into_iter()
        .map(|item: sea_orm::prelude::QueryResult| {
            Ok(StatsConfInfo {
                col_key: item.try_get("", "col_key")?,
                show_name: item.try_get("", "show_name")?,
                col_kind: item.try_get("", "col_kind")?,
                dim_multi_values: item.try_get("", "dim_multi_values")?,
                mes_data_distinct: item.try_get("", "mes_data_distinct")?,
                mes_data_type: if item.try_get::<Option<String>>("", "mes_data_type")?.is_none() {
                    None
                } else {
                    Some(item.try_get("", "mes_data_type")?)
                },
                dim_data_type: if item.try_get::<Option<String>>("", "dim_data_type")?.is_none() {
                    None
                } else {
                    Some(item.try_get("", "dim_data_type")?)
                },
                query_limit: item.try_get("", "query_limit")?,
                dim_rel_conf_dim_key: if item.try_get::<Option<String>>("", "dim_rel_conf_dim_key")?.is_none() {
                    None
                } else {
                    Some(item.try_get("", "dim_rel_conf_dim_key")?)
                },
                dim_hierarchy: if item.try_get::<Option<Vec<String>>>("", "dim_hierarchy")?.is_none() {
                    None
                } else {
                    Some(item.try_get("", "dim_hierarchy")?)
                },
                rel_external_id: if item.try_get::<Option<String>>("", "rel_external_id")?.is_none() {
                    None
                } else {
                    Some(item.try_get("", "rel_external_id")?)
                },
            })
        })
        .collect::<TardisResult<Vec<StatsConfInfo>>>()?;

    let query_limit = match conf_info.as_slice() {
        [] => {
            return Err(funs.err().not_found(
                "metric",
                "query",
                &format!("The query fact [{}] does not exist.", query_req.from),
                "404-spi-stats-metric-fact-not-exist",
            ));
        }
        [first, ..] => first.query_limit,
    };
    // Add default dimension
    conf_info.push(StatsConfInfo {
        col_key: "key".to_string(),
        show_name: "主键".to_string(),
        col_kind: StatsFactColKind::Measure,
        dim_multi_values: Some(false),
        mes_data_distinct: Some(true),
        mes_data_type: Some(StatsDataTypeKind::String),
        dim_rel_conf_dim_key: None,
        dim_data_type: None,
        dim_hierarchy: None,
        rel_external_id: None,
        query_limit,
    });
    conf_info.push(StatsConfInfo {
        col_key: "ct".to_string(),
        show_name: "创建时间".to_string(),
        col_kind: StatsFactColKind::Dimension,
        dim_multi_values: Some(false),
        mes_data_distinct: Some(true),
        mes_data_type: None,
        dim_rel_conf_dim_key: None,
        dim_data_type: Some(StatsDataTypeKind::DateTime),
        dim_hierarchy: None,
        rel_external_id: None,
        query_limit,
    });
    conf_info.push(StatsConfInfo {
        col_key: "_count".to_string(),
        show_name: "虚构计算数".to_string(),
        col_kind: StatsFactColKind::Measure,
        dim_multi_values: Some(false),
        mes_data_distinct: Some(true),
        mes_data_type: Some(StatsDataTypeKind::Int),
        dim_rel_conf_dim_key: None,
        dim_data_type: None,
        dim_hierarchy: None,
        rel_external_id: None,
        query_limit,
    });

    let conf_limit = query_limit;
    let conf_info =
        conf_info.into_iter().map(|v| (format!("{}{}", v.col_key.clone(), v.rel_external_id.clone().unwrap_or_default()), v)).collect::<HashMap<String, StatsConfInfo>>();
    if query_req.select.iter().any(|i| !conf_info.contains_key(&format!("{}{}",&i.code,i.rel_external_id.clone().unwrap_or_default())))
        // should be equivalent: 
        // original: || query_req.group.iter().any(|i| !conf_info.contains_key(&i.code) || conf_info.get(&i.code).unwrap().col_kind != StatsFactColKind::Dimension))
        // (!contain || not_dim) => !(contain && is_dim)
        || query_req.group.iter().any(|i| !conf_info.get(&format!("{}{}",&i.code,i.rel_external_id.clone().unwrap_or("".to_string()))).is_some_and(|i|i.col_kind == StatsFactColKind::Dimension))
        || query_req
            .group_order
            .as_ref()
            .map(|orders| orders.iter().any(|order| !query_req.group.iter().any(|group| group.code == order.code && group.time_window == order.time_window && group.rel_external_id == order.rel_external_id)))
            .unwrap_or(false)
        || query_req
            .metrics_order
            .as_ref()
            .map(|orders| orders.iter().any(|order| !query_req.select.iter().any(|select| order.code == select.code && order.fun == select.fun&& order.rel_external_id == select.rel_external_id)))
            .unwrap_or(false)
        || query_req
            .having
            .as_ref()
            .map(|havings| havings.iter().any(|having| !query_req.select.iter().any(|select| having.code == select.code && having.fun == select.fun&& having.rel_external_id == select.rel_external_id)))
            .unwrap_or(false)
        || query_req._where.as_ref().map(|or_wheres| or_wheres.iter().any(|and_wheres| and_wheres.iter().any(|where_| !conf_info.contains_key(&format!("{}{}",where_.code,where_.rel_external_id.clone().unwrap_or_default()))))).unwrap_or(false)
    {
        return Err(funs.err().not_found(
            "metric",
            "query",
            "The query some dimension or measures does not exist.",
            "404-spi-stats-metric-dim-mea-not-exist",
        ));
    }
    let mes_distinct = query_req.select.iter().any(|i| {
        if let Some(conf) = conf_info.get(&format!("{}{}", i.code, i.rel_external_id.clone().unwrap_or_default())) {
            return conf.mes_data_distinct.unwrap_or(false);
        }
        false
    });

    let mut params = if let Some(own_paths) = &query_req.own_paths {
        own_paths.iter().map(Value::from).collect_vec()
    } else {
        vec![Value::from(format!("{}%", ctx.own_paths))]
    };
    let own_paths_count = params.len();
    params.push(Value::from(query_req.start_time));
    params.push(Value::from(query_req.end_time));

    // Package filter
    let mut sql_part_wheres = vec![];
    if let Some(wheres) = &query_req._where {
        let mut sql_part_or_wheres = vec![];
        for or_wheres in wheres {
            let mut sql_part_and_wheres = vec![];
            for and_where in or_wheres {
                let col_conf = conf_info.get(&format!("{}{}", and_where.code, and_where.rel_external_id.clone().unwrap_or_default())).ok_or(funs.err().internal_error(
                    "metric",
                    "query",
                    &format!("missing config with code [{code}]", code = and_where.code),
                    "500-spi-stats-internal-error",
                ))?;
                let col_data_type = if col_conf.col_kind == StatsFactColKind::Dimension {
                    col_conf.dim_data_type.as_ref().ok_or(funs.err().internal_error(
                        "metric",
                        "query",
                        &format!("config missing dim_data_type with code [{code}]", code = and_where.code),
                        "500-spi-stats-internal-error",
                    ))?
                } else {
                    col_conf.mes_data_type.as_ref().ok_or(funs.err().internal_error(
                        "metric",
                        "query",
                        &format!("config missing mes_data_type with code [{code}]", code = and_where.code),
                        "500-spi-stats-internal-error",
                    ))?
                };
                info!("col_data_type={:?}", col_data_type);
                if let Some((sql_part, value)) = col_data_type.to_pg_where(
                    col_conf.dim_multi_values.unwrap_or(false),
                    &format!("fact.{}", &and_where.code),
                    &and_where.op,
                    params.len() + 1,
                    &and_where.value,
                    &and_where.time_window,
                )? {
                    value.iter().for_each(|v| params.push(v.clone()));
                    sql_part_and_wheres.push(sql_part);
                } else {
                    return Err(funs.err().not_found(
                        "metric",
                        "query",
                        &format!(
                            "The query column=[{}] type=[{}] operation=[{}] time_window=[{}] multi_values=[{}] is not legal.",
                            &and_where.code,
                            col_data_type.to_string().to_lowercase(),
                            &and_where.op.to_sql(),
                            &and_where.time_window.is_some(),
                            col_conf.dim_multi_values.unwrap_or_default()
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
        if let Some(rel_external_id) = &select.rel_external_id {
            sql_part_inner_selects.push(format!(
                "fact.ext ->> '{}' AS {}",
                &select.code,
                &format!("{}{FUNCTION_SUFFIX_FLAG}{}", select.code, rel_external_id)
            ));
        } else {
            sql_part_inner_selects.push(format!("fact.{} AS {}", &select.code, &select.code));
        }
    }
    for group in &query_req.group {
        if let Some(rel_external_id) = &group.rel_external_id {
            sql_part_inner_selects.push(format!(
                "fact.ext ->> '{}' AS {}",
                &group.code,
                &format!("{}{FUNCTION_SUFFIX_FLAG}{}", group.code, rel_external_id)
            ));
        } else {
            sql_part_inner_selects.push(format!("fact.{} AS {}", &group.code, &group.code));
        }
    }
    let sql_part_inner_selects = sql_part_inner_selects.join(",");

    // Package group
    // (column name with fun, alias name, show name)
    let mut sql_part_group_infos = vec![];
    for group in &query_req.group {
        let col_conf = conf_info.get(&format!("{}{}", group.code, group.rel_external_id.clone().unwrap_or_default())).ok_or(funs.err().not_found(
            "metric",
            "query",
            &format!("Missing config for group code [{code}] does not exist.", code = group.code),
            "500-spi-stats-internal-error",
        ))?;
        let col_data_type = col_conf.dim_data_type.as_ref().ok_or(funs.err().not_found(
            "metric",
            "query",
            &format!("Missing col_data_type for group code [{code}] does not exist.", code = group.code),
            "500-spi-stats-internal-error",
        ))?;
        if let Some(column_name_with_fun) = col_data_type.to_pg_group(
            &format!(
                "_.{}",
                if let Some(rel_external_id) = &group.rel_external_id {
                    format!("{}{FUNCTION_SUFFIX_FLAG}{}", group.code, rel_external_id)
                } else {
                    group.code.clone()
                }
            ),
            col_conf.dim_multi_values.unwrap_or(false),
            &group.time_window,
        ) {
            let alias_name = format!(
                "{}{FUNCTION_SUFFIX_FLAG}{}",
                if let Some(rel_external_id) = &group.rel_external_id {
                    format!("{}{FUNCTION_SUFFIX_FLAG}{}", group.code, rel_external_id)
                } else {
                    group.code.clone()
                },
                group.time_window.as_ref().map(|i| i.to_string().to_lowercase()).unwrap_or("".to_string())
            );
            sql_part_group_infos.push((column_name_with_fun, alias_name, col_conf.show_name.clone()));
        } else {
            return Err(funs.err().not_found(
                "metric",
                "query",
                &format!(
                    "The group column=[{}] type=[{}] time_window=[{}] is not legal.",
                    &group.code,
                    col_data_type.to_string().to_lowercase(),
                    &group.time_window.is_some(),
                ),
                "404-spi-stats-metric-op-not-legal",
            ));
        }
    }
    let sql_part_groups = sql_part_group_infos.iter().map(|group| group.1.clone()).collect::<Vec<String>>().join(",");

    // Package outer select
    // (column name with fun, alias name, show_name, is dimension)
    let mut sql_part_outer_select_infos = vec![];
    for (column_name_with_fun, alias_name, show_name) in sql_part_group_infos {
        sql_part_outer_select_infos.push((column_name_with_fun, alias_name, show_name, true));
    }
    for select in &query_req.select {
        let col_conf = conf_info.get(&format!("{}{}", select.code, select.rel_external_id.clone().unwrap_or_default())).ok_or(funs.err().not_found(
            "metric",
            "query",
            &format!("Missing col_data_type for select code [{code}] does not exist.", code = select.code),
            "500-spi-stats-internal-error",
        ))?;
        let col_data_type = col_conf.mes_data_type.as_ref().ok_or(funs.err().not_found(
            "metric",
            "query",
            &format!("Missing col_data_type for select code [{code}] does not exist.", code = select.code),
            "500-spi-stats-internal-error",
        ))?;
        let column_name_with_fun = col_data_type.to_pg_select(
            &format!(
                "_.{}",
                if let Some(rel_external_id) = &select.rel_external_id {
                    format!("{}{FUNCTION_SUFFIX_FLAG}{}", select.code, rel_external_id)
                } else {
                    select.code.clone()
                }
            ),
            &select.fun,
        );
        let alias_name = format!(
            "{}{FUNCTION_SUFFIX_FLAG}{}",
            if let Some(rel_external_id) = &select.rel_external_id {
                format!("{}{FUNCTION_SUFFIX_FLAG}{}", select.code, rel_external_id)
            } else {
                select.code.clone()
            },
            select.fun.to_string().to_lowercase()
        );
        sql_part_outer_select_infos.push((column_name_with_fun, alias_name, col_conf.show_name.clone(), false));
    }
    let sql_part_outer_selects =
        sql_part_outer_select_infos.iter().map(|(column_name_with_fun, alias_name, _, _)| format!("{column_name_with_fun} AS {alias_name}")).collect::<Vec<String>>().join(",");

    // Package having
    let sql_part_havings = if let Some(havings) = &query_req.having {
        let mut sql_part_havings = vec![];
        for having in havings {
            let col_conf = conf_info.get(&format!("{}{}", having.code, having.rel_external_id.clone().unwrap_or_default())).ok_or(funs.err().not_found(
                "metric",
                "query",
                &format!("Missing config for having code [{code}] does not exist.", code = having.code),
                "500-spi-stats-internal-error",
            ))?;
            if let Some((sql_part, value)) = col_conf
                .mes_data_type
                .as_ref()
                .ok_or(funs.err().not_found(
                    "metric",
                    "query",
                    &format!("Missing mes_data_type for having code [{code}] does not exist.", code = having.code),
                    "500-spi-stats-internal-error",
                ))?
                .to_pg_having(
                    false,
                    &format!(
                        "_.{}",
                        if let Some(rel_external_id) = &having.rel_external_id {
                            format!("{}{FUNCTION_SUFFIX_FLAG}{}", having.code, rel_external_id)
                        } else {
                            having.code.clone()
                        }
                    ),
                    &having.op,
                    params.len() + 1,
                    &having.value,
                    Some(&having.fun),
                )?
            {
                value.iter().for_each(|v| params.push(v.clone()));
                sql_part_havings.push(sql_part);
            } else {
                return Err(funs.err().not_found(
                    "metric",
                    "query",
                    &format!(
                        "The query column=[{}] type=[{}] operation=[{}] fun=[{}] is not legal.",
                        &having.code,
                        col_conf.mes_data_type.as_ref().map(ToString::to_string).unwrap_or("None".into()).to_lowercase(),
                        &having.op.to_sql(),
                        &having.fun.to_string().to_lowercase()
                    ),
                    "404-spi-stats-metric-op-not-legal",
                ));
            }
        }
        format!("HAVING {}", sql_part_havings.join(","))
    } else {
        "".to_string()
    };

    // Package dimension order
    let sql_dimension_orders = if let Some(orders) = &query_req.dimension_order {
        let sql_part_orders = orders
            .iter()
            .map(|order| {
                if order.rel_external_id.is_some() {
                    format!("fact.ext ->>{} {}", order.code, if order.asc { "ASC" } else { "DESC" })
                } else {
                    format!("fact.{} {}", order.code, if order.asc { "ASC" } else { "DESC" })
                }
            })
            .collect::<Vec<String>>();
        format!("ORDER BY {}", sql_part_orders.join(","))
    } else {
        "".to_string()
    };

    // Package metrics or group order
    let sql_orders = if query_req.group_order.is_some() || query_req.metrics_order.is_some() {
        let mut sql_part_orders = Vec::new();
        if let Some(orders) = &query_req.group_order {
            let group_orders = orders
                .iter()
                .map(|order| {
                    format!(
                        "{}{FUNCTION_SUFFIX_FLAG}{} {}",
                        if let Some(rel_external_id) = &order.rel_external_id {
                            format!("{}{FUNCTION_SUFFIX_FLAG}{}", order.code, rel_external_id)
                        } else {
                            order.code.clone()
                        },
                        order.time_window.as_ref().map(|i| i.to_string().to_lowercase()).unwrap_or("".to_string()),
                        if order.asc { "ASC" } else { "DESC" }
                    )
                })
                .collect::<Vec<String>>();
            sql_part_orders.extend(group_orders);
        }
        if let Some(orders) = &query_req.metrics_order {
            let metrics_orders = orders
                .iter()
                .map(|order| {
                    format!(
                        "{}{FUNCTION_SUFFIX_FLAG}{} {}",
                        if let Some(rel_external_id) = &order.rel_external_id {
                            format!("{}{FUNCTION_SUFFIX_FLAG}{}", order.code, rel_external_id)
                        } else {
                            order.code.clone()
                        },
                        order.fun.to_string().to_lowercase(),
                        if order.asc { "ASC" } else { "DESC" }
                    )
                })
                .collect::<Vec<String>>();
            sql_part_orders.extend(metrics_orders);
        }
        format!("ORDER BY {}", sql_part_orders.join(","))
    } else {
        "".to_string()
    };
    // package limit
    let query_limit = if let Some(limit) = &query_req.limit { format!("LIMIT {limit}") } else { "".to_string() };
    let ignore_group_agg = !(!sql_part_groups.is_empty() && query_req.group_agg.unwrap_or(false));
    let own_paths_placeholder = (1..=own_paths_count).map(|idx| format!("${}", idx)).collect::<Vec<String>>().join(", ");
    let create_time_placeholder = format!("${}", own_paths_count + 1);
    let end_time_placeholder = format!("${}", own_paths_count + 2);
    let filter_own_paths = if query_req.own_paths.is_some() {
        format!("fact.own_paths IN ({own_paths_placeholder})")
    } else {
        "fact.own_paths LIKE $1".to_string()
    };
    let final_sql = format!(
        r#"SELECT {sql_part_outer_selects}{}
    FROM (
        SELECT
             {sql_part_inner_selects}{}
             FROM(
                SELECT {}fact.*, 1 as _count
                FROM {fact_inst_table_name} fact
                LEFT JOIN {fact_inst_del_table_name} del ON del.key = fact.key AND del.ct >= {create_time_placeholder} AND del.ct <= {end_time_placeholder}
                WHERE
                    {filter_own_paths}
                    AND del.key IS NULL
                    AND fact.ct >= {create_time_placeholder} AND fact.ct <= {end_time_placeholder}
                ORDER BY {}fact.ct DESC
             ) fact 
             where 1 = 1
            {sql_part_wheres}
            {sql_dimension_orders}
        LIMIT {conf_limit}
    ) _
    {}
    {sql_part_havings}
    {sql_orders}
    {query_limit}"#,
        if ignore_group_agg {
            "".to_string()
        } else {
            ",string_agg(_._key || ' - ' || _._own_paths || ' - ' || to_char(_._ct, 'YYYY-MM-DD HH24:MI:SS'), ',') as s_agg".to_string()
        },
        if ignore_group_agg {
            "".to_string()
        } else {
            ",fact.key as _key, fact.own_paths as _own_paths, fact.ct as _ct".to_string()
        },
        if query_req.ignore_distinct.unwrap_or(false) {
            ""
        } else if mes_distinct {
            "DISTINCT ON (fact.key) fact.key AS _key,"
        } else {
            ""
        },
        if query_req.ignore_distinct.unwrap_or(false) {
            ""
        } else if mes_distinct {
            "_key,"
        } else {
            ""
        },
        if sql_part_groups.is_empty() {
            "".to_string()
        } else {
            #[allow(clippy::collapsible_else_if)]
            if query_req.ignore_group_rollup.unwrap_or(false) {
                format!("GROUP BY {sql_part_groups}")
            } else {
                format!("GROUP BY ROLLUP({sql_part_groups})")
            }
        }
    );

    let result = conn
        .query_all(&final_sql, params)
        .await?
        .iter()
        .map(|record|
        // TODO This method cannot get the data of array type, so the dimension of multiple values, such as labels, cannot be obtained.
        serde_json::Value::from_query_result(record, ""))
        .collect::<Result<Vec<_>, _>>()?;

    let select_dimension_keys =
        sql_part_outer_select_infos.iter().filter(|(_, _, _, is_dimension)| *is_dimension).map(|(_, alias_name, _, _)| alias_name.to_string()).collect::<Vec<String>>();
    let select_measure_keys =
        sql_part_outer_select_infos.iter().filter(|(_, _, _, is_dimension)| !*is_dimension).map(|(_, alias_name, _, _)| alias_name.to_string()).collect::<Vec<String>>();
    let show_names = sql_part_outer_select_infos.into_iter().map(|(_, alias_name, show_name, _)| (alias_name, show_name)).collect::<HashMap<String, String>>();
    let dim_record_agg = package_dim_record_agg(conf_info.clone(), funs, ctx).await?;
    Ok(StatsQueryMetricsResp {
        from: query_req.from.to_string(),
        show_names,
        group: package_groups(dim_record_agg, conf_info, select_dimension_keys, &select_measure_keys, ignore_group_agg, result)
            .map_err(|msg| TardisError::internal_error(&format!("Fail to package groups: {msg}"), "500-spi-stats-internal-error"))?,
    })
}

// todo 下钻 上探
async fn package_dim_record_agg(
    conf_info: HashMap<String, StatsConfInfo>,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<HashMap<String, HashMap<String, serde_json::Value>>> {
    let mut result: HashMap<String, HashMap<String, serde_json::Value>> = HashMap::new();
    for (col_key, conf) in conf_info.clone() {
        if conf.dim_rel_conf_dim_key.is_none() {
            continue;
        }
        let dimension_key = conf.dim_rel_conf_dim_key.unwrap_or_default();
        let dimension_hierarchy = if let Some(stats_con_info) = conf_info.get(&col_key) {
            stats_con_info.dim_hierarchy.clone()
        } else {
            None
        };
        if dimension_hierarchy.unwrap_or(vec![]).len() > 0 {
            let dim: HashMap<String, serde_json::Value> = dim_record_paginate(dimension_key.clone(), None, None, 1, 9999, None, None, funs, ctx)
                .await?
                .records
                .into_iter()
                .map(|dim| (String::from(serde_json::json!(dim.get("key")).as_str().unwrap_or("")), dim))
                .collect();
            result.insert(format!("{}{FUNCTION_SUFFIX_FLAG}", col_key.clone()), dim);
        } else {
            result.insert(format!("{}{FUNCTION_SUFFIX_FLAG}", col_key), HashMap::new());
        }
    }
    Ok(result)
}

fn package_groups(
    dim_record_agg: HashMap<String, HashMap<String, serde_json::Value>>,
    conf_info: HashMap<String, StatsConfInfo>,
    curr_select_dimension_keys: Vec<String>,
    select_measure_keys: &Vec<String>,
    ignore_group_agg: bool,
    result: Vec<serde_json::Value>,
) -> Result<serde_json::Value, String> {
    if curr_select_dimension_keys.is_empty() {
        let first_result = result.first().ok_or("result is empty")?;
        let mut leaf_node = Map::with_capacity(result.len());
        for measure_key in select_measure_keys {
            let val = first_result.get(measure_key).ok_or(format!("failed to get key {measure_key}"))?;
            let val = if measure_key.ends_with(&format!("{FUNCTION_SUFFIX_FLAG}avg")) {
                // Fix `avg` function return type error
                let val = val
                    .as_str()
                    .ok_or(format!("value of field {measure_key} should be a string"))?
                    .parse::<f64>()
                    .map_err(|_| format!("value of field {measure_key} can not be parsed as a valid f64 number"))?;
                serde_json::Value::from(val)
            } else {
                val.clone()
            };
            leaf_node.insert(measure_key.to_string(), val.clone());
        }
        if !ignore_group_agg {
            leaf_node.insert("group".to_string(), first_result.get("group").ok_or("failed to get key group".to_string())?.clone());
        }
        return Ok(serde_json::Value::Object(leaf_node));
    }
    let mut node = Map::with_capacity(0);

    let dimension_key = curr_select_dimension_keys.first().ok_or("curr_select_dimension_keys is empty")?;

    // todo 下钻 上探
    // let dimension_hierarchy = if let Some(stats_con_info) = conf_info.get(dimension_key.split(FUNCTION_SUFFIX_FLAG).next().unwrap_or("")) {
    //     stats_con_info.dim_hierarchy.clone()
    // } else {
    //     None
    // };
    // let dimension_hierarchy_len = dimension_hierarchy.unwrap_or(vec![]).len() as i32;
    let mut groups = HashMap::new();
    let mut order = Vec::new();
    for record in result {
        let key = {
            let key = record.get(dimension_key).unwrap_or(&json!(null));
            match key {
                serde_json::Value::Null => "ROLLUP".to_string(),
                serde_json::Value::String(s) => s.clone(),
                not_null => not_null.to_string(),
            }
        };
        let group = groups.entry(key.clone()).or_insert_with(Vec::new);
        if ignore_group_agg {
            group.push(record.clone());
        } else {
            let mut g_aggs = record.clone();
            if let Some(g_agg) = g_aggs.as_object_mut() {
                g_agg.insert("group".to_owned(), package_groups_agg(record)?);
            }
            group.push(g_aggs.clone());
        }
        if !order.contains(&key) {
            order.push(key.clone());
        }
    }
    for key in order {
        let group = groups.get(&key).expect("groups shouldn't miss the value of key in order");
        let sub = package_groups(
            dim_record_agg.clone(),
            conf_info.clone(),
            curr_select_dimension_keys[1..].to_vec(),
            select_measure_keys,
            ignore_group_agg,
            group.to_vec(),
        )?;
        // todo 下钻 上探
        // println!("dimension_key:[{}],key[{}]", dimension_key, key);
        // if let Some(dim_record_map) = dim_record_agg.get(dimension_key) {
        //     println!("dim_record_map:{:?}", dim_record_map);
        //     if let Some(dim_record) = dim_record_map.get(&key) {
        //         println!("dim_record:{:?} , {}", dim_record, dimension_hierarchy_len);
        //         for i in dimension_hierarchy_len..0 {
        //             let field_key = format!("key{}",i);
        //             if let Some(val) = dim_record.get(field_key.clone()) {
        //                 if !val.as_str().unwrap_or_default().is_empty() {
        //                     let val = dim_record.get(format!("key{i}")).expect("msg").to_string();
        //                     println!("on [{}]",val );
        //                     node.insert(val, sub.clone());
        //                 }
        //             }
        //         }
        //     }
        // }
        node.insert(key, sub);
    }
    Ok(serde_json::Value::Object(node))
}

fn package_groups_agg(record: serde_json::Value) -> Result<serde_json::Value, String> {
    match record.get("s_agg") {
        Some(agg) => {
            if agg.is_null() {
                return Ok(serde_json::Value::Null);
            }
            let mut details = Vec::new();
            let var_agg = agg.as_str().ok_or("field group_agg should be a string")?;
            let vars = var_agg.split(',').collect::<Vec<&str>>();
            for var in vars {
                let fields = var.split(" - ").collect::<Vec<&str>>();
                details.push(json!({
                    "key": fields.first().unwrap_or(&""),
                    "own_paths": fields.get(1).unwrap_or(&""),
                    "ct": fields.get(2).unwrap_or(&""),
                }));
            }
            Ok(serde_json::Value::Array(details))
        }
        None => Ok(serde_json::Value::Null),
    }
}

#[derive(sea_orm::FromQueryResult, Clone)]
struct StatsConfInfo {
    pub col_key: String,
    pub show_name: String,
    pub col_kind: StatsFactColKind,
    pub dim_multi_values: Option<bool>,
    pub mes_data_distinct: Option<bool>,
    pub mes_data_type: Option<StatsDataTypeKind>,
    pub dim_rel_conf_dim_key: Option<String>,
    pub dim_data_type: Option<StatsDataTypeKind>,
    pub dim_hierarchy: Option<Vec<String>>,
    pub rel_external_id: Option<String>,
    pub query_limit: i32,
}
