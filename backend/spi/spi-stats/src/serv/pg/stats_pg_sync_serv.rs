use std::collections::HashMap;

use bios_basic::{
    helper::db_helper::{json_to_sea_orm_value_pure, sea_orm_value_to_json},
    process::task_processor::TaskProcessor,
    spi::{
        spi_funs::{SpiBsInst, SpiBsInstExtractor},
        spi_initializer::common_pg::{self},
    },
};
use serde_json::{json, Map};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    chrono::{DateTime, Utc},
    db::{
        reldb_client::TardisRelDBClient,
        sea_orm::{FromQueryResult, Value},
    },
    TardisFunsInst,
};

use crate::{
    dto::{stats_conf_dto::StatsConfFactColInfoResp, stats_record_dto::StatsFactRecordLoadReq},
    serv::{stats_cert_serv, stats_valid_serv},
    stats_config::StatsConfig,
    stats_enumeration::{StatsDataTypeKind, StatsFactColKind},
    stats_initializer,
};

use super::{stats_pg_conf_fact_col_serv, stats_pg_conf_fact_serv, stats_pg_record_serv};

pub(crate) async fn fact_record_sync(fact_conf_key: &str, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (conn, _) = common_pg::init_conn(bs_inst).await?;

    let Some(fact_conf) = stats_pg_conf_fact_serv::get(fact_conf_key, &conn, ctx).await? else {
        return Err(funs.err().not_found("starsys_stats_conf_fact", "find", "fact conf not found", "404-fact-conf-not-found"));
    };
    let Some(cert_id) = fact_conf.rel_cert_id else {
        return Err(funs.err().bad_request("starsys_stats_conf_fact", "sync", "The rel_cert_id is required", "404-fact-conf-not-found"));
    };
    let Some(sync_sql) = fact_conf.sync_sql else {
        return Err(funs.err().bad_request("starsys_stats_conf_fact", "sync", "The sync_sql is required", "404-fact-conf-not-found"));
    };
    if cert_id.is_empty() || sync_sql.is_empty() {
        return Err(funs.err().bad_request("starsys_stats_conf_fact", "sync", "The rel_cert_id and sync_sql is required", "404-fact-conf-not-found"));
    }
    let task_ctx = ctx.clone();
    let fact_conf_key = fact_conf_key.to_string();
    TaskProcessor::execute_task_with_ctx(
        &funs.conf::<StatsConfig>().cache_key_async_task_status,
        move |task_id| async move {
            let funs = stats_initializer::get_tardis_inst();
            let inst = funs.init(None, &task_ctx, true, stats_initializer::init_fun).await?;
            let db_source_conn = stats_cert_serv::get_db_conn_by_cert_id(&cert_id, &funs, &task_ctx).await?;
            let db_source_list = db_source_conn.query_all(&sync_sql, vec![]).await?;
            let mut success = 0;
            let mut error = 0;
            let mut error_list = vec![];
            let total = db_source_list.len();
            for db_source_record in db_source_list {
                let fact_record_key = db_source_record.try_get::<String>("", "key")?;
                let add_req = StatsFactRecordLoadReq {
                    own_paths: db_source_record.try_get::<Option<String>>("", "own_paths")?.unwrap_or_default(),
                    ct: db_source_record.try_get::<Option<DateTime<Utc>>>("", "ct")?.unwrap_or_default(),
                    idempotent_id: db_source_record.try_get::<Option<String>>("", "idempotent_id")?,
                    ignore_updates: Some(false),
                    data: serde_json::Value::from_query_result(&db_source_record, "")?,
                    ext: None,
                };
                let load_resp = stats_pg_record_serv::fact_record_load(&fact_conf_key, &fact_record_key, add_req, &funs, &task_ctx, inst.as_ref()).await;
                if load_resp.is_ok() {
                    success += 1;
                } else {
                    error += 1;
                    error_list.push(json!({"key":fact_record_key,"error":load_resp.unwrap_err().to_string()}));
                }
                let _ = TaskProcessor::set_process_data(
                    &funs.conf::<StatsConfig>().cache_key_async_task_status,
                    task_id,
                    json!({"success":success,"error":error,"total":total,"error_list":error_list}),
                    &funs.cache(),
                )
                .await;
            }
            Ok(())
        },
        &funs.cache(),
        "".to_string(),
        None,
        ctx,
    )
    .await?;
    Ok(())
}

pub(crate) async fn fact_col_record_sync(fact_conf_key: &str, fact_col_conf_key: &str, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let Some(fact_col_conf) = stats_pg_conf_fact_col_serv::find_by_fact_key_and_col_conf_key(fact_conf_key, fact_col_conf_key, funs, ctx, inst).await? else {
        return Err(funs.err().not_found("starsys_stats_conf_fact_col", "sync", "fact col conf not found", "404-fact-col-conf-not-found"));
    };
    if fact_col_conf.rel_sql.is_none() {
        return Err(funs.err().bad_request("starsys_stats_conf_fact_col", "sync", "The rel_sql is required", "404-fact-col-conf-not-found"));
    }
    let task_ctx = ctx.clone();
    let fact_conf_key = fact_conf_key.to_string();
    let fact_col_conf_key = fact_col_conf_key.to_string();
    TaskProcessor::execute_task_with_ctx(
        &funs.conf::<StatsConfig>().cache_key_async_task_status,
        move |task_id| async move {
            let funs = stats_initializer::get_tardis_inst();
            let inst = funs.init(None, &task_ctx, true, stats_initializer::init_fun).await?;
            let mut page_number = 1;
            let page_size = 10;
            let mut success = 0;
            let mut error = 0;
            let mut error_list = vec![];
            loop {
                let fact_record_pages =
                    stats_pg_record_serv::get_fact_record_paginated(&fact_conf_key, None, page_number, page_size, Some(true), &funs, &task_ctx, inst.as_ref()).await?;
                for fact_record in &fact_record_pages.records {
                    let fact_record_key = fact_record.get("key").and_then(|v| v.as_str()).unwrap_or("");
                    if let Some(idempotent_id) = fact_record.get("idempotent_id") {
                        if idempotent_id.is_null() || idempotent_id.as_str().unwrap_or_default().is_empty() {
                            continue;
                        }
                        let fact_record_map = fact_record.as_object().unwrap().iter().filter_map(|(k, v)| json_to_sea_orm_value_pure(v).map(|val| (k.clone(), val))).collect();
                        if let Some(col_result) = fact_col_record_result(fact_col_conf.clone(), fact_record_map, &funs, &task_ctx, inst.as_ref()).await? {
                            let add_req = StatsFactRecordLoadReq {
                                own_paths: fact_record.get("own_paths").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                                ct: Utc::now(),
                                idempotent_id: Some(idempotent_id.as_str().unwrap_or_default().to_string()),
                                ignore_updates: Some(false),
                                data: {
                                    let mut map = Map::new();
                                    map.insert(fact_col_conf_key.clone(), sea_orm_value_to_json(col_result.clone()).unwrap_or_default());
                                    serde_json::Value::Object(map)
                                },
                                ext: None,
                            };
                            let load_resp = stats_pg_record_serv::fact_record_load(&fact_conf_key, fact_record_key, add_req, &funs, &task_ctx, inst.as_ref()).await;
                            if load_resp.is_ok() {
                                success += 1;
                            } else {
                                error += 1;
                                error_list.push(json!({"key":fact_record_key,"error":load_resp.unwrap_err().to_string()}));
                            }
                            let _ = TaskProcessor::set_process_data(
                                &funs.conf::<StatsConfig>().cache_key_async_task_status,
                                task_id,
                                json!({"success":success,"error":error,"total":fact_record_pages.total_size,"error_list":error_list}),
                                &funs.cache(),
                            )
                            .await;
                        }
                    }
                }
                if fact_record_pages.records.len() < page_size.try_into().unwrap() || fact_record_pages.total_size <= (page_size as u64 * (page_number - 1) as u64) {
                    break;
                }
                page_number += 1;
            }
            Ok(())
        },
        &funs.cache(),
        "spi-stats".to_string(),
        None,
        ctx,
    )
    .await?;
    Ok(())
}

pub(crate) async fn fact_col_record_result(
    fact_col: StatsConfFactColInfoResp,
    fact_record: HashMap<String, Value>,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
    _inst: &SpiBsInst,
) -> TardisResult<Option<Value>> {
    let Some(cert_id) = fact_col.rel_cert_id else {
        return Ok(None);
    };
    let Some(sql) = fact_col.rel_sql else {
        return Ok(None);
    };
    if cert_id.is_empty() || sql.is_empty() {
        return Ok(None);
    }
    let data_source_conn = stats_cert_serv::get_db_conn_by_cert_id(&cert_id, funs, ctx).await?;
    let (sql, params) = stats_valid_serv::process_sql(&sql, &fact_record)?;
    if let Some(rel_record) = data_source_conn.query_one(&sql, params).await? {
        if let Some(first_column) = rel_record.column_names().get(0) {
            let result = match fact_col.kind {
                StatsFactColKind::Dimension | StatsFactColKind::Ext => {
                    if fact_col.dim_multi_values.unwrap_or(false) {
                        fact_col.dim_data_type.clone().unwrap_or(StatsDataTypeKind::String).result_to_sea_orm_value_array(&rel_record, first_column)?
                    } else {
                        fact_col.dim_data_type.clone().unwrap_or(StatsDataTypeKind::String).result_to_sea_orm_value(&rel_record, first_column)?
                    }
                }
                StatsFactColKind::Measure => fact_col.mes_data_type.clone().unwrap_or(StatsDataTypeKind::Int).result_to_sea_orm_value(&rel_record, first_column)?,
            };
            return Ok(Some(result));
        }
    }
    Ok(None)
}
