use std::collections::HashMap;

use bios_basic::{
    helper::db_helper::{json_to_sea_orm_value_pure, sea_orm_value_to_json},
    process::task_processor::TaskProcessor,
    rbum::{
        dto::{
            rbum_cert_dto::{RbumCertAddReq, RbumCertModifyReq},
            rbum_filer_dto::{RbumBasicFilterReq, RbumCertFilterReq},
        },
        rbum_enumeration::{RbumCertRelKind, RbumCertStatusKind},
        serv::{rbum_cert_serv::RbumCertServ, rbum_crud_serv::RbumCrudOperation},
    },
    spi::{
        spi_constants::SPI_PG_KIND_CODE,
        spi_funs::{SpiBsInst, SpiBsInstExtractor},
        spi_initializer::common_pg::{self},
    },
};
use serde_json::Map;
use tardis::{
    basic::{dto::TardisContext, error::TardisError, field::TrimString, result::TardisResult},
    chrono::{DateTime, Utc},
    config::config_dto::{CompatibleType, DBModuleConfig},
    db::{
        reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
        sea_orm::{FromQueryResult, Value},
    },
    log,
    regex::{self, Regex},
    TardisFunsInst,
};

use crate::{
    dto::{
        stats_conf_dto::{
            StatsConfFactColInfoResp, StatsSyncDbConfigAddReq, StatsSyncDbConfigExt, StatsSyncDbConfigInfoResp, StatsSyncDbConfigInfoWithSkResp, StatsSyncDbConfigModifyReq,
        },
        stats_record_dto::StatsFactRecordLoadReq,
    },
    stats_config::StatsConfig,
    stats_constants::DOMAIN_CODE,
    stats_enumeration::{StatsDataTypeKind, StatsFactColKind},
    stats_initializer,
};

use super::{stats_pg_conf_fact_col_serv, stats_pg_conf_fact_serv, stats_pg_record_serv};

pub(crate) async fn db_config_add(add_req: StatsSyncDbConfigAddReq, funs: &TardisFunsInst, ctx: &TardisContext, _inst: &SpiBsInst) -> TardisResult<String> {
    // 使用rel_rbum_id kind supplier 来作为unique key
    let mut rbum_cert_add_req = RbumCertAddReq {
        ak: TrimString(add_req.db_user),
        sk: Some(TrimString(add_req.db_password)),
        conn_uri: Some(add_req.db_url),
        rel_rbum_id: "".to_string(),
        kind: Some(SPI_PG_KIND_CODE.to_string()),
        supplier: Some(DOMAIN_CODE.to_string()),
        ext: serde_json::to_string(&StatsSyncDbConfigExt {
            max_connections: add_req.max_connections,
            min_connections: add_req.min_connections,
        })
        .ok(),
        sk_invisible: None,
        ignore_check_sk: false,
        start_time: None,
        end_time: None,
        status: RbumCertStatusKind::Enabled,
        vcode: None,
        rel_rbum_cert_conf_id: None,
        rel_rbum_kind: RbumCertRelKind::Item,
        is_outside: true,
    };
    let rbum_cert = RbumCertServ::add_rbum(&mut rbum_cert_add_req, funs, ctx).await?;
    Ok(rbum_cert)
}

pub(crate) async fn db_config_modify(modify_req: StatsSyncDbConfigModifyReq, funs: &TardisFunsInst, ctx: &TardisContext, _inst: &SpiBsInst) -> TardisResult<()> {
    if RbumCertServ::find_one_rbum(
        &RbumCertFilterReq {
            basic: RbumBasicFilterReq {
                ids: Some(vec![modify_req.id.clone()]),
                ..Default::default()
            },
            ..Default::default()
        },
        funs,
        ctx,
    )
    .await?
    .is_some()
    {
        let mut rbum_cert_modify_req = RbumCertModifyReq {
            ak: modify_req.db_user.map(TrimString),
            sk: modify_req.db_password.map(TrimString),
            conn_uri: modify_req.db_url,
            sk_invisible: None,
            ignore_check_sk: false,
            ext: None,
            start_time: None,
            end_time: None,
            status: None,
        };
        RbumCertServ::modify_rbum(&modify_req.id, &mut rbum_cert_modify_req, funs, ctx).await?;
    } else {
        return Err(funs.err().not_found(&RbumCertServ::get_obj_name(), "modify", "rbum cert not found", "404-rbum-cert-not-found"));
    }
    Ok(())
}

pub(crate) async fn db_config_list(funs: &TardisFunsInst, ctx: &TardisContext, _inst: &SpiBsInst) -> TardisResult<Vec<StatsSyncDbConfigInfoResp>> {
    let rbum_cert_list = RbumCertServ::find_detail_rbums(
        &RbumCertFilterReq {
            kind: Some(SPI_PG_KIND_CODE.to_string()),
            suppliers: Some(vec![DOMAIN_CODE.to_string()]),
            ..Default::default()
        },
        None,
        None,
        funs,
        ctx,
    )
    .await?;

    return Ok(rbum_cert_list
        .iter()
        .map(|rbum_cert| {
            let ext = serde_json::from_str::<StatsSyncDbConfigExt>(&rbum_cert.ext).ok();
            StatsSyncDbConfigInfoResp {
                id: rbum_cert.id.clone(),
                db_url: rbum_cert.conn_uri.clone(),
                db_user: rbum_cert.ak.clone(),
                max_connections: ext.clone().and_then(|ext| ext.max_connections),
                min_connections: ext.clone().and_then(|ext| ext.min_connections),
            }
        })
        .collect());
}

async fn get_db_conn_by_cert_id(cert_id: &str, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<TardisRelDBlConnection> {
    let db_config = get_db_config(cert_id, funs, ctx, inst).await?;
    let data_source_conn = TardisRelDBClient::init(&DBModuleConfig {
        url: format!("postgres://{}:{}@{}", db_config.db_user, db_config.db_password, db_config.db_url),
        max_connections: db_config.max_connections.unwrap_or(20),
        min_connections: db_config.min_connections.unwrap_or(5),
        connect_timeout_sec: None,
        idle_timeout_sec: None,
        compatible_type: CompatibleType::default(),
    })
    .await?
    .conn();
    Ok(data_source_conn)
}

async fn get_db_config(cert_id: &str, funs: &TardisFunsInst, ctx: &TardisContext, _inst: &SpiBsInst) -> TardisResult<StatsSyncDbConfigInfoWithSkResp> {
    if let Some(rbum_cert) = RbumCertServ::find_one_detail_rbum(
        &RbumCertFilterReq {
            basic: RbumBasicFilterReq {
                ids: Some(vec![cert_id.to_string()]),
                ..Default::default()
            },
            kind: Some(SPI_PG_KIND_CODE.to_string()),
            suppliers: Some(vec![DOMAIN_CODE.to_string()]),
            ..Default::default()
        },
        funs,
        ctx,
    )
    .await?
    {
        let db_password = RbumCertServ::show_sk(cert_id, &RbumCertFilterReq::default(), funs, ctx).await?;
        let ext = serde_json::from_str::<StatsSyncDbConfigExt>(&rbum_cert.ext).ok();
        let max_connections = ext.clone().and_then(|ext| ext.max_connections);
        let min_connections = ext.clone().and_then(|ext| ext.min_connections);
        Ok(StatsSyncDbConfigInfoWithSkResp {
            id: cert_id.to_string(),
            db_url: rbum_cert.conn_uri.clone(),
            db_user: rbum_cert.ak.clone(),
            db_password,
            max_connections,
            min_connections,
        })
    } else {
        return Err(funs.err().not_found(&RbumCertServ::get_obj_name(), "find", "rbum cert not found", "404-rbum-cert-not-found"));
    }
}

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
        move |_task_id| async move {
            let funs = stats_initializer::get_tardis_inst();
            let inst = funs.init(None, &task_ctx, true, stats_initializer::init_fun).await?;
            let db_source_conn = get_db_conn_by_cert_id(&cert_id, &funs, &task_ctx, inst.as_ref()).await?;
            let db_source_list = db_source_conn.query_all(&sync_sql, vec![]).await?;
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
                stats_pg_record_serv::fact_record_load(&fact_conf_key, &fact_record_key, add_req, &funs, &task_ctx, inst.as_ref()).await?;
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
        move |_task_id| async move {
            let funs = stats_initializer::get_tardis_inst();
            let inst = funs.init(None, &task_ctx, true, stats_initializer::init_fun).await?;
            let mut page_number = 1;
            let page_size = 10;
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
                            stats_pg_record_serv::fact_record_load(&fact_conf_key, fact_record_key, add_req, &funs, &task_ctx, inst.as_ref()).await?;
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
        "".to_string(),
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
    inst: &SpiBsInst,
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
    let data_source_conn = get_db_conn_by_cert_id(&cert_id, funs, ctx, inst).await?;
    let (sql, params) = process_sql(&sql, &fact_record)?;
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

fn process_sql(sql: &str, fact_record: &HashMap<String, Value>) -> TardisResult<(String, Vec<Value>)> {
    let mut values = Vec::new();
    let mut placeholder_index = 1;
    // 正则匹配 `${key}` 形式的占位符
    let re = Regex::new(r"\$\{(\w+)\}").unwrap();
    let mut is_err = false;
    let mut err_msg = Err(TardisError::bad_request("The rel_sql is not a valid sql.", "400-spi-stats-fact-col-conf-rel-sql-not-valid"));
    // 替换 `${key}` 为 `$1`、`$2` 等
    let processed_sql = re.replace_all(sql, |caps: &regex::Captures| {
        // 提取键名并存入值列表
        let key = caps[1].to_string();
        // 从 record 中获取对应的 Value 值
        if let Some(value) = fact_record.get(&key) {
            values.push(value.clone()); // 将值推送到 Vec
        } else {
            is_err = true;
            err_msg = Err(TardisError::bad_request(
                &format!("The key [{}] not found in fact record", key),
                "400-spi-stats-fact-col-conf-key-not-found-in-fact-record",
            ));
        }
        let result = format!("${}", placeholder_index);
        placeholder_index += 1;
        result
    });
    if is_err {
        return err_msg;
    }
    // 返回替换后的 SQL 和提取的值列表
    Ok((processed_sql.to_string(), values))
}

/// validate fact and fact col sql
pub(crate) fn validate_select_sql(sql: &str) -> bool {
    if sql.is_empty() {
        return true;
    }
    let re = Regex::new(r"(?i)^\s*select\b").expect("should compile regex");
    re.is_match(&sql)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use tardis::{
        chrono::{DateTime, Utc},
        db::sea_orm::Value,
    };

    use crate::serv::pg::stats_pg_sync_serv::{process_sql, validate_select_sql};

    #[test]
    fn test_validate_select_sql() {
        let sql = "SELECT * FROM users";
        assert_eq!(validate_select_sql(sql), true);
        let sql = " select name FROM users";
        assert_eq!(validate_select_sql(sql), true);
        let sql = "INSERT INTO users (name) VALUES ('John')";
        assert_eq!(validate_select_sql(sql), false);
        let sql = "UPDATE users SET name = 'John'";
        assert_eq!(validate_select_sql(sql), false);
    }

    #[test]
    fn test_generate_sql_and_params() {
        let sql = "select id from table where id = ${id} and name = ${name} and age = ${age} and ct = ${ct}";
        let mut fact_record = HashMap::new();
        fact_record.insert("ct".to_string(), Value::from(DateTime::<Utc>::from_timestamp(1715260800, 0)));
        fact_record.insert("id".to_string(), Value::from("1"));
        fact_record.insert("age".to_string(), Value::from(18));
        fact_record.insert("name".to_string(), Value::from("name1"));
        let (sql, params) = process_sql(sql, &fact_record).unwrap();
        assert_eq!(sql, "select id from table where id = $1 and name = $2 and age = $3 and ct = $4");
        assert_eq!(
            params,
            vec![
                Value::from("1"),
                Value::from("name1"),
                Value::from(18),
                Value::from(DateTime::<Utc>::from_timestamp(1715260800, 0))
            ]
        );
    }
}
