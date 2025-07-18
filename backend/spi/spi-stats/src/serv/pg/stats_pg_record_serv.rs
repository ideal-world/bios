use std::collections::{HashMap, HashSet};

use bios_basic::spi::{
    spi_funs::SpiBsInst,
    spi_initializer::common_pg::{self, package_table_name},
};
use itertools::Itertools;
use serde_json::Map;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    chrono::{DateTime, Utc},
    db::{
        reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
        sea_orm::{FromQueryResult, Value},
    },
    log::info,
    serde_json,
    web::{poem_openapi::types::Type, web_resp::TardisPage},
    TardisFuns, TardisFunsInst,
};

use crate::{
    dto::{
        stats_conf_dto::StatsConfFactColInfoResp,
        stats_record_dto::{StatsDimRecordAddReq, StatsFactRecordLoadReq, StatsFactRecordsLoadReq},
    },
    serv::pg::stats_pg_sync_serv,
    stats_enumeration::StatsFactColKind,
};

use super::{stats_pg_conf_dim_serv, stats_pg_conf_fact_col_serv, stats_pg_conf_fact_serv};

pub(crate) async fn get_fact_record_latest(
    fact_conf_key: &str,
    fact_record_keys: impl IntoIterator<Item = &str>,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<Vec<serde_json::Value>> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (conn, _) = common_pg::init_conn(bs_inst).await?;
    let (placeholder, params) = fact_record_keys.into_iter().fold((String::new(), Vec::new()), |(mut placeholder, mut values), key| {
        if !placeholder.is_empty() {
            placeholder.push(',')
        }
        values.push(Value::from(key));
        placeholder.push_str(&format!("${}", values.len()));
        (placeholder, values)
    });
    if !stats_pg_conf_fact_serv::online(fact_conf_key, &conn, ctx).await? {
        return Err(funs.err().conflict("fact_record", "load", "The fact config not online.", "409-spi-stats-fact-conf-not-online"));
    }
    let table_name = package_table_name(&format!("stats_inst_fact_{fact_conf_key}"), ctx);
    let result = conn
        .query_all(
            &format!(
                r#"SELECT * FROM (
    SELECT *, ROW_NUMBER() OVER (PARTITION BY "key" ORDER BY ct DESC) AS rn, count(*) OVER() AS total
    FROM {table_name}
    WHERE "key" IN ({placeholder})
) t WHERE rn=1
"#,
            ),
            params,
        )
        .await?;
    let values = result
        .iter()
        .map(|result| {
            let result = serde_json::Value::from_query_result_optional(result, "")?;
            let mut value = result.unwrap_or_default();
            value.as_object_mut().map(|obj| obj.remove("rn"));
            Ok(value)
        })
        .collect::<TardisResult<Vec<serde_json::Value>>>()?;
    Ok(values)
}

pub(crate) async fn get_fact_record_paginated(
    fact_conf_key: &str,
    fact_record_key: Option<String>,
    page_number: u32,
    page_size: u32,
    desc_by_create: Option<bool>,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<TardisPage<serde_json::Value>> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (conn, _) = common_pg::init_conn(bs_inst).await?;
    if !stats_pg_conf_fact_serv::online(fact_conf_key, &conn, ctx).await? {
        return Err(funs.err().conflict("fact_record", "load", "The fact config not online.", "409-spi-stats-fact-conf-not-online"));
    }
    let table_name = package_table_name(&format!("stats_inst_fact_{fact_conf_key}"), ctx);
    let mut sql_order = String::new();
    if let Some(desc_by_create) = desc_by_create {
        sql_order = format!("ORDER BY ct {}", if desc_by_create { "DESC" } else { "ASC" });
    }
    let mut params = vec![Value::from(page_size), Value::from((page_number - 1) * page_size)];
    let mut where_fragments = vec!["1 = 1".to_string()];
    if let Some(fact_record_key) = fact_record_key {
        where_fragments.push(format!("key = ${}", params.len() + 1));
        params.push(Value::from(fact_record_key));
    }
    let result = conn
        .query_all(
            &format!(
                r#"SELECT *, count(*) OVER() AS total
FROM {table_name}
WHERE 
    {}
{sql_order}
LIMIT $1 OFFSET $2
"#,
                where_fragments.join(" AND ")
            ),
            params,
        )
        .await?;
    let total: i64;
    if let Some(first) = result.first() {
        total = first.try_get("", "total")?;
    } else {
        total = 0;
    }
    let records = result
        .iter()
        .map(|item| serde_json::Value::from_query_result_optional(item, "").map(|x| x.unwrap_or(serde_json::Value::Null)))
        .collect::<Result<Vec<serde_json::Value>, _>>()?;
    Ok(TardisPage {
        page_size: page_size.into(),
        page_number: page_size.into(),
        total_size: total as u64,
        records,
    })
}

pub(crate) async fn fact_record_load(
    fact_conf_key: &str,
    fact_record_key: &str,
    add_req: StatsFactRecordLoadReq,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, _) = common_pg::init_conn(bs_inst).await?;
    conn.begin().await?;
    if !stats_pg_conf_fact_serv::online(fact_conf_key, &conn, ctx).await? {
        return Err(funs.err().conflict("fact_record", "load", "The fact config not online.", "409-spi-stats-fact-conf-not-online"));
    }

    let fact_col_conf_set = stats_pg_conf_fact_col_serv::find_by_fact_conf_key(fact_conf_key, funs, ctx, inst).await?;

    let mut fields_values = HashMap::<String, Value>::new();
    fields_values.insert("key".to_string(), Value::from(fact_record_key));
    fields_values.insert("own_paths".to_string(), Value::from(add_req.own_paths));
    fields_values.insert("ct".to_string(), Value::from(add_req.ct));
    fields_values.insert("idempotent_id".to_string(), Value::from(add_req.idempotent_id.clone().unwrap_or_default()));
    let req_data = add_req.data.as_object().ok_or_else(|| {
        funs.err().bad_request(
            "fact_record",
            "load",
            "
        Data should be an map
    ",
            "400-spi-stats-invalid-request",
        )
    })?;

    // 如果存在幂等id 且已经存在对应数据,则丢弃或者进行修改
    // If there is an idempotent id and the corresponding data already exists, discard or modify it
    if let Some(idempotent_id) = add_req.idempotent_id.clone() {
        let idempotent_data_resp = fact_get_idempotent_record_raw(fact_conf_key, &idempotent_id, &conn, ctx).await?;
        if idempotent_data_resp.is_some() {
            if !add_req.ignore_updates.unwrap_or(true) {
                return self::fact_records_modify(fact_conf_key, &idempotent_id, req_data.clone(), fact_col_conf_set, conn, funs, ctx, inst).await;
            }
            return Ok(());
        }
    }
    let latest_data_resp = fact_get_latest_record_raw(fact_conf_key, fact_record_key, fact_col_conf_set.clone(), &conn, ctx).await?;
    if let Some(latest_data) = latest_data_resp.as_ref() {
        let mut storage_ext = latest_data.try_get("", "ext")?;
        merge(&mut storage_ext, add_req.ext.unwrap_or(TardisFuns::json.str_to_json("{}")?));
        fields_values.insert("ext".to_string(), Value::from(storage_ext));
    } else {
        fields_values.insert("ext".to_string(), add_req.ext.unwrap_or(TardisFuns::json.str_to_json("{}")?).into());
    }
    // Because Dimension and Measure may share the same field
    // Existing fields are not stored in duplicate
    for (req_fact_col_key, req_fact_col_value) in req_data {
        // 查找一下是否命中了rel_field字段
        let fact_col_conf = fact_col_conf_set.iter().find(|c| &c.key == req_fact_col_key || c.rel_field.as_ref() == Some(req_fact_col_key));
        if fact_col_conf.is_none()
            || fields_values.contains_key(&fact_col_conf.unwrap().key)
            || req_fact_col_value.is_null()
            || req_fact_col_value.is_none()
            || req_fact_col_value.is_empty()
        {
            continue;
        }
        let fact_col_conf = fact_col_conf.unwrap();
        let req_fact_col_key = fact_col_conf.key.as_str();
        if fact_col_conf.kind == StatsFactColKind::Dimension {
            let Some(key) = fact_col_conf.dim_rel_conf_dim_key.as_ref() else {
                return Err(funs.err().not_found("fact_record", "load", "Fail to get conf_dim_key", "400-spi-stats-fail-to-get-dim-config-key"));
            };
            let Some(dim_conf) = stats_pg_conf_dim_serv::get(key, None, None, &conn, ctx, inst).await? else {
                return Err(funs.err().not_found(
                    "fact_record",
                    "load",
                    &format!("Fail to get dim_conf by key [{key}]"),
                    "400-spi˚-stats-fail-to-get-dim-config-key",
                ));
            };
            if fact_col_conf.dim_multi_values.unwrap_or(false) {
                fields_values.insert(req_fact_col_key.to_string(), dim_conf.data_type.json_to_sea_orm_value_array(req_fact_col_value, false)?);
            } else {
                fields_values.insert(req_fact_col_key.to_string(), dim_conf.data_type.json_to_sea_orm_value(req_fact_col_value, false)?);
            }
        } else if fact_col_conf.kind == StatsFactColKind::Measure {
            let Some(mes_data_type) = fact_col_conf.mes_data_type.as_ref() else {
                return Err(funs.err().bad_request(
                    "fact_record",
                    "load",
                    "Col_conf.mes_data_type shouldn't be empty while fact_col_conf.kind is Measure",
                    "400-spi-stats-invalid-request",
                ));
            };
            fields_values.insert(req_fact_col_key.to_string(), mes_data_type.json_to_sea_orm_value(req_fact_col_value, false)?);
        } else {
            let Some(req_fact_col_value) = req_fact_col_value.as_str() else {
                return Err(funs.err().bad_request(
                    "fact_record",
                    "load",
                    &format!("For the key [{req_fact_col_key}], value: [{req_fact_col_value}] is not a string"),
                    "400-spi-stats-invalid-request",
                ));
            };
            fields_values.insert(req_fact_col_key.to_string(), req_fact_col_value.into());
        }
    }
    info!("fact_col_conf_set={0},req_data={1}", fact_col_conf_set.len(), req_data.len());
    if let Some(latest_data) = latest_data_resp {
        for fact_col_conf in fact_col_conf_set {
            if fields_values.contains_key(&fact_col_conf.key) {
                continue;
            }
            let fact_col_result = if fact_col_conf.rel_sql.is_some() {
                stats_pg_sync_serv::fact_col_record_result(fact_col_conf.clone(), fields_values.clone(), funs, ctx, inst).await?
            } else {
                None
            };
            if fact_col_conf.kind == StatsFactColKind::Dimension {
                let Some(dim_rel_conf_dim_key) = &fact_col_conf.dim_rel_conf_dim_key else {
                    return Err(funs.err().internal_error("fact_record", "load", "dim_rel_conf_dim_key unexpectedly being empty", "500-spi-stats-internal-error"));
                };
                let Some(dim_conf) = stats_pg_conf_dim_serv::get(dim_rel_conf_dim_key, None, None, &conn, ctx, inst).await? else {
                    return Err(funs.err().internal_error(
                        "fact_record",
                        "load",
                        &format!("key [{dim_rel_conf_dim_key}] missing corresponding config "),
                        "500-spi-stats-internal-error",
                    ));
                };
                if fact_col_conf.dim_multi_values.unwrap_or(false) {
                    if dim_conf.data_type.result_to_sea_orm_value_array(&latest_data, &fact_col_conf.key).is_err() {
                        fields_values.insert(
                            fact_col_conf.key.to_string(),
                            fact_col_result.unwrap_or(dim_conf.data_type.result_to_sea_orm_value_array_default()?),
                        );
                    } else {
                        fields_values.insert(
                            fact_col_conf.key.to_string(),
                            fact_col_result.unwrap_or(dim_conf.data_type.result_to_sea_orm_value_array(&latest_data, &fact_col_conf.key)?),
                        );
                    }
                } else {
                    if dim_conf.data_type.result_to_sea_orm_value(&latest_data, &fact_col_conf.key).is_err() {
                        fields_values.insert(
                            fact_col_conf.key.to_string(),
                            fact_col_result.unwrap_or(dim_conf.data_type.result_to_sea_orm_value_default()?),
                        );
                    } else {
                        fields_values.insert(
                            fact_col_conf.key.to_string(),
                            fact_col_result.unwrap_or(dim_conf.data_type.result_to_sea_orm_value(&latest_data, &fact_col_conf.key)?),
                        );
                    }
                }
            } else if fact_col_conf.kind == StatsFactColKind::Measure {
                let Some(mes_data_type) = fact_col_conf.mes_data_type.as_ref() else {
                    return Err(funs.err().bad_request(
                        "fact_record",
                        "load",
                        "Col_conf.mes_data_type shouldn't be empty while fact_col_conf.kind is Measure",
                        "400-spi-stats-invalid-request",
                    ));
                };
                if mes_data_type.result_to_sea_orm_value(&latest_data, &fact_col_conf.key).is_err() {
                    fields_values.insert(fact_col_conf.key.to_string(), fact_col_result.unwrap_or(mes_data_type.result_to_sea_orm_value_default()?));
                } else {
                    fields_values.insert(
                        fact_col_conf.key.to_string(),
                        fact_col_result.unwrap_or(mes_data_type.result_to_sea_orm_value(&latest_data, &fact_col_conf.key)?),
                    );
                }
            } else {
                fields_values.insert(
                    fact_col_conf.key.to_string(),
                    fact_col_result.unwrap_or(Value::from(latest_data.try_get::<String>("", &fact_col_conf.key)?)),
                );
            }
        }
    } else {
        for fact_col_conf in fact_col_conf_set {
            if fields_values.contains_key(&fact_col_conf.key) {
                continue;
            }
            let fact_col_result = if fact_col_conf.rel_sql.is_some() {
                stats_pg_sync_serv::fact_col_record_result(fact_col_conf.clone(), fields_values.clone(), funs, ctx, inst).await?
            } else {
                None
            };
            if fact_col_conf.kind == StatsFactColKind::Dimension {
                let Some(dim_rel_conf_dim_key) = &fact_col_conf.dim_rel_conf_dim_key else {
                    return Err(funs.err().internal_error("fact_record", "load", "dim_rel_conf_dim_key unexpectedly being empty", "500-spi-stats-internal-error"));
                };
                let Some(dim_conf) = stats_pg_conf_dim_serv::get(dim_rel_conf_dim_key, None, None, &conn, ctx, inst).await? else {
                    return Err(funs.err().internal_error(
                        "fact_record",
                        "load",
                        &format!("key [{dim_rel_conf_dim_key}] missing corresponding config "),
                        "500-spi-stats-internal-error",
                    ));
                };
                if fact_col_conf.dim_multi_values.unwrap_or(false) {
                    fields_values.insert(
                        fact_col_conf.key.to_string(),
                        fact_col_result.unwrap_or(dim_conf.data_type.result_to_sea_orm_value_array_default()?),
                    );
                } else {
                    fields_values.insert(
                        fact_col_conf.key.to_string(),
                        fact_col_result.unwrap_or(dim_conf.data_type.result_to_sea_orm_value_default()?),
                    );
                }
            } else if fact_col_conf.kind == StatsFactColKind::Measure {
                let Some(mes_data_type) = fact_col_conf.mes_data_type.as_ref() else {
                    return Err(funs.err().bad_request(
                        "fact_record",
                        "load",
                        "Col_conf.mes_data_type shouldn't be empty while fact_col_conf.kind is Measure",
                        "400-spi-stats-invalid-request",
                    ));
                };
                fields_values.insert(fact_col_conf.key.to_string(), fact_col_result.unwrap_or(mes_data_type.result_to_sea_orm_value_default()?));
            }
        }
    }
    let field_keys = fields_values.keys().collect::<Vec<&String>>();
    let field_values = fields_values.values().collect::<Vec<&Value>>();
    let table_name = package_table_name(&format!("stats_inst_fact_{fact_conf_key}"), ctx);
    conn.execute_one(
        &format!(
            r#"INSERT INTO {table_name}
({})
VALUES
({})
"#,
            field_keys.into_iter().join(","),
            field_values.iter().enumerate().map(|(i, _)| format!("${}", i + 1)).collect::<Vec<String>>().join(",")
        ),
        field_values.into_iter().cloned().collect::<Vec<Value>>(),
    )
    .await?;
    conn.commit().await?;
    Ok(())
}

pub(crate) async fn fact_records_load(
    fact_conf_key: &str,
    add_req_set: Vec<StatsFactRecordsLoadReq>,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, _) = common_pg::init_conn(bs_inst).await?;
    conn.begin().await?;
    if !stats_pg_conf_fact_serv::online(fact_conf_key, &conn, ctx).await? {
        return Err(funs.err().conflict("fact_record", "load_set", "The fact config not online.", "409-spi-stats-fact-conf-not-online"));
    }

    let fact_col_conf_set = stats_pg_conf_fact_col_serv::find_by_fact_conf_key(fact_conf_key, funs, ctx, inst).await?;

    let mut has_fields_init = false;
    let mut fields = vec!["key".to_string(), "own_paths".to_string(), "ext".to_string(), "ct".to_string(), "idempotent_id".to_string()];
    let mut value_sets = vec![];

    for add_req in add_req_set {
        let Some(req_data) = add_req.data.as_object() else {
            return Err(funs.err().bad_request(
                "fact_record",
                "load_set",
                &format!("add_req.data should be a map value, got {data}", data = add_req.data),
                "400-spi-stats-invalid-request",
            ));
        };
        // 如果存在幂等id 且已经存在对应数据,则丢弃数据
        if let Some(idempotent_id) = add_req.idempotent_id.clone() {
            let idempotent_data_resp = fact_get_idempotent_record_raw(fact_conf_key, &idempotent_id, &conn, ctx).await?;
            if idempotent_data_resp.is_some() {
                if !add_req.ignore_updates.unwrap_or(true) {
                    return self::fact_records_modify(fact_conf_key, &idempotent_id, req_data.clone(), fact_col_conf_set, conn, funs, ctx, inst).await;
                }
                continue;
            }
        }
        let mut values = vec![
            Value::from(&add_req.key),
            Value::from(add_req.own_paths),
            Value::from(if let Some(ext) = add_req.ext {
                ext.clone()
            } else {
                TardisFuns::json.str_to_json("{}")?
            }),
            Value::from(add_req.ct),
            Value::from(add_req.idempotent_id.unwrap_or(TardisFuns::field.nanoid())),
        ];
        // Because Dimension and Measure may share the same field
        // Existing fields are not stored in duplicate
        let mut exist_fields = HashSet::new();
        for fact_col_conf in &fact_col_conf_set {
            if exist_fields.contains(&fact_col_conf.key) {
                continue;
            }
            exist_fields.insert(fact_col_conf.key.clone());
            if !has_fields_init {
                fields.push(fact_col_conf.key.to_string());
            }

            let req_fact_col_value = req_data.get(&fact_col_conf.key).ok_or_else(|| {
                funs.err().bad_request(
                    "fact_record",
                    "load_set",
                    &format!(
                        "The fact instance record [{}][{}] is missing a required column [{}].",
                        fact_conf_key, add_req.key, fact_col_conf.key
                    ),
                    "400-spi-stats-fact-inst-record-missing-column",
                )
            })?;
            if fact_col_conf.kind == StatsFactColKind::Dimension {
                let Some(key) = fact_col_conf.dim_rel_conf_dim_key.as_ref() else {
                    return Err(funs.err().not_found("fact_record", "load_set", "Fail to get conf_dim_key", "400-spi-stats-fail-to-get-dim-config-key"));
                };
                let Some(dim_conf) = stats_pg_conf_dim_serv::get(key, None, None, &conn, ctx, inst).await? else {
                    return Err(funs.err().not_found(
                        "fact_record",
                        "load_set",
                        &format!("Fail to get dim_conf by key [{key}]"),
                        "400-spi-stats-fail-to-get-dim-config-key",
                    ));
                };
                // TODO check value enum when stable_ds =true
                if fact_col_conf.dim_multi_values.unwrap_or(false) {
                    values.push(dim_conf.data_type.json_to_sea_orm_value_array(req_fact_col_value, false)?);
                } else {
                    values.push(dim_conf.data_type.json_to_sea_orm_value(req_fact_col_value, false)?);
                }
            } else if fact_col_conf.kind == StatsFactColKind::Measure {
                let Some(mes_data_type) = fact_col_conf.mes_data_type.as_ref() else {
                    return Err(funs.err().bad_request(
                        "fact_record",
                        "load_set",
                        "Col_conf.mes_data_type shouldn't be empty while fact_col_conf.kind is Measure",
                        "400-spi-stats-invalid-request",
                    ));
                };
                values.push(mes_data_type.json_to_sea_orm_value(req_fact_col_value, false)?);
            } else {
                let Some(req_fact_col_value) = req_fact_col_value.as_str() else {
                    return Err(funs.err().bad_request(
                        "fact_record",
                        "load_set",
                        &format!("Value: [{req_fact_col_value}] is not a string"),
                        "400-spi-stats-invalid-request",
                    ));
                };
                values.push(req_fact_col_value.into());
            }
            // TODO check data type
        }
        value_sets.push(values);
        has_fields_init = true;
    }

    let table_name = package_table_name(&format!("stats_inst_fact_{fact_conf_key}"), ctx);
    let fields_placeholders = fields.iter().enumerate().map(|(i, _)| format!("${}", i + 1)).collect::<Vec<String>>().join(",");
    let fields = fields.join(",");
    for values in value_sets {
        conn.execute_one(
            &format!(
                r#"INSERT INTO {table_name}
    ({fields})
    VALUES
    ({fields_placeholders})
    "#,
            ),
            values,
        )
        .await?;
    }
    conn.commit().await?;
    Ok(())
}

async fn fact_records_modify(
    fact_conf_key: &str,
    idempotent_id: &str,
    req_data: Map<String, serde_json::Value>,
    fact_col_conf_set: Vec<StatsConfFactColInfoResp>,
    conn: TardisRelDBlConnection,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<()> {
    let mut sql_sets = vec![];
    let mut params = vec![Value::from(idempotent_id.to_string())];
    let mut fields = vec![];
    for (req_fact_col_key, req_fact_col_value) in req_data {
        let fact_col_conf = fact_col_conf_set.iter().find(|c| c.key == req_fact_col_key || c.rel_field.as_ref() == Some(&req_fact_col_key));
        if fact_col_conf.is_none() || fields.contains(&fact_col_conf.unwrap().key) || req_fact_col_value.is_null() || req_fact_col_value.is_none() || req_fact_col_value.is_empty()
        {
            continue;
        }
        let fact_col_conf = fact_col_conf.unwrap();
        fields.push(fact_col_conf.key.clone());
        if fact_col_conf.kind == StatsFactColKind::Dimension {
            let Some(key) = fact_col_conf.dim_rel_conf_dim_key.as_ref() else {
                return Err(funs.err().not_found("fact_record", "load", "Fail to get conf_dim_key", "400-spi-stats-fail-to-get-dim-config-key"));
            };
            let Some(dim_conf) = stats_pg_conf_dim_serv::get(key, None, None, &conn, ctx, inst).await? else {
                return Err(funs.err().not_found(
                    "fact_record",
                    "load",
                    &format!("Fail to get dim_conf by key [{key}]"),
                    "400-spi˚-stats-fail-to-get-dim-config-key",
                ));
            };
            // TODO check value enum when stable_ds = true
            sql_sets.push(format!("{} = ${}", fact_col_conf.key, params.len() + 1));
            if fact_col_conf.dim_multi_values.unwrap_or(false) {
                params.push(dim_conf.data_type.json_to_sea_orm_value_array(&req_fact_col_value, false)?);
            } else {
                params.push(dim_conf.data_type.json_to_sea_orm_value(&req_fact_col_value, false)?);
            }
        } else if fact_col_conf.kind == StatsFactColKind::Measure {
            let Some(mes_data_type) = fact_col_conf.mes_data_type.as_ref() else {
                return Err(funs.err().bad_request(
                    "fact_record",
                    "load",
                    "Col_conf.mes_data_type shouldn't be empty while fact_col_conf.kind is Measure",
                    "400-spi-stats-invalid-request",
                ));
            };
            sql_sets.push(format!("{} = ${}", fact_col_conf.key, params.len() + 1));
            params.push(mes_data_type.json_to_sea_orm_value(&req_fact_col_value, false)?);
        } else {
            let Some(req_fact_col_value) = req_fact_col_value.as_str() else {
                return Err(funs.err().bad_request(
                    "fact_record",
                    "load",
                    &format!("For the key [{req_fact_col_key}], value: [{req_fact_col_value}] is not a string"),
                    "400-spi-stats-invalid-request",
                ));
            };
            sql_sets.push(format!("{} = ${}", fact_col_conf.key, params.len() + 1));
            params.push(req_fact_col_value.into());
        }
    }
    if sql_sets.is_empty() {
        return Err(funs.err().bad_request("fact_record", "load", "The fact column no data", "400-spi-stats-invalid-request"));
    }
    let table_name = package_table_name(&format!("stats_inst_fact_{fact_conf_key}"), ctx);
    conn.execute_one(
        &format!(
            r#"UPDATE {table_name}
SET {}
WHERE idempotent_id = $1"#,
            sql_sets.join(",")
        ),
        params,
    )
    .await?;
    conn.commit().await?;
    Ok(())
}

pub(crate) async fn fact_record_delete(fact_conf_key: &str, fact_record_key: &str, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, _) = common_pg::init_conn(bs_inst).await?;
    conn.begin().await?;
    if !stats_pg_conf_fact_serv::online(fact_conf_key, &conn, ctx).await? {
        return Err(funs.err().conflict("fact_record", "delete", "The fact config not online.", "409-spi-stats-fact-conf-not-online"));
    }

    let table_name = package_table_name(&format!("stats_inst_fact_{fact_conf_key}_del"), ctx);
    conn.execute_one(
        &format!(
            r#"INSERT INTO {table_name}
(key)
VALUES
($1)
"#,
        ),
        vec![Value::from(fact_record_key)],
    )
    .await?;
    conn.commit().await?;
    Ok(())
}

pub(crate) async fn fact_records_delete(fact_conf_key: &str, fact_record_delete_keys: &[String], funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, _) = common_pg::init_conn(bs_inst).await?;
    conn.begin().await?;
    if !stats_pg_conf_fact_serv::online(fact_conf_key, &conn, ctx).await? {
        return Err(funs.err().conflict("fact_record", "delete_set", "The fact config not online.", "409-spi-stats-fact-conf-not-online"));
    }

    let table_name = package_table_name(&format!("stats_inst_fact_{fact_conf_key}_del"), ctx);
    for delete_key in fact_record_delete_keys {
        conn.execute_one(
            &format!(
                r#"INSERT INTO {table_name}
    (key)
    VALUES
    ($1)
    "#,
            ),
            vec![Value::from(delete_key)],
        )
        .await?;
    }
    conn.commit().await?;
    Ok(())
}

pub(crate) async fn fact_records_delete_by_ownership(fact_conf_key: &str, own_paths: &str, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, _) = common_pg::init_conn(bs_inst).await?;
    conn.begin().await?;
    if !stats_pg_conf_fact_serv::online(fact_conf_key, &conn, ctx).await? {
        return Err(funs.err().conflict("fact_record", "delete_set", "The fact config not online.", "409-spi-stats-fact-conf-not-online"));
    }

    let table_name = package_table_name(&format!("stats_inst_fact_{fact_conf_key}"), ctx);
    conn.execute_one(
        &format!(
            r#"DELETE FROM {table_name}
            WHERE own_paths = $1
    "#,
        ),
        vec![Value::from(own_paths)],
    )
    .await?;
    conn.commit().await?;
    Ok(())
}

pub(crate) async fn fact_records_delete_by_dim_key(
    fact_conf_key: &str,
    dim_conf_key: &str,
    dim_record_key: Option<serde_json::Value>,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, _) = common_pg::init_conn(bs_inst).await?;
    conn.begin().await?;
    if !stats_pg_conf_fact_serv::online(fact_conf_key, &conn, ctx).await? {
        return Err(funs.err().conflict("fact_record", "delete_set", "The fact config not online.", "409-spi-stats-fact-conf-not-online"));
    }
    let fact_record_delete_keys = self::find_fact_record_key(fact_conf_key.to_owned(), dim_conf_key.to_owned(), dim_record_key, &conn, funs, ctx, inst).await?;
    if fact_record_delete_keys.is_empty() {
        return Ok(());
    }
    let table_name = package_table_name(&format!("stats_inst_fact_{fact_conf_key}_del"), ctx);
    for delete_key in fact_record_delete_keys {
        conn.execute_one(
            &format!(
                r#"INSERT INTO {table_name}
    (key)
    VALUES
    ($1)
    "#,
            ),
            vec![Value::from(delete_key)],
        )
        .await?;
    }
    conn.commit().await?;
    Ok(())
}

async fn find_fact_record_key(
    fact_conf_key: String,
    dim_conf_key: String,
    dim_record_key: Option<serde_json::Value>,
    conn: &TardisRelDBlConnection,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<Vec<String>> {
    let dim_conf = stats_pg_conf_dim_serv::get(&dim_conf_key, None, None, conn, ctx, inst)
        .await?
        .ok_or_else(|| funs.err().not_found("fact_record", "find", "The dimension config does not exist.", "404-spi-stats-dim-conf-not-exist"))?;
    let fact_conf_col_key = stats_pg_conf_fact_col_serv::find_by_fact_conf_key(&fact_conf_key, funs, ctx, inst)
        .await?
        .into_iter()
        .find_or_first(|r| r.dim_rel_conf_dim_key.clone().unwrap_or("".to_string()) == dim_conf_key)
        .ok_or_else(|| funs.err().not_found("fact_record", "delete_set", "The fact config does not exist.", "404-spi-stats-fact-conf-not-exist"))?
        .key;
    let table_name = package_table_name(&format!("stats_inst_fact_{fact_conf_key}"), ctx);
    let mut sql_where = vec!["1 = 1".to_string()];
    let mut params: Vec<Value> = vec![];
    if let Some(dim_record_key) = &dim_record_key {
        sql_where.push(format!("{fact_conf_col_key} = $1"));
        params.push(dim_conf.data_type.json_to_sea_orm_value(dim_record_key, false)?);
    }
    let result = conn
        .query_all(
            &format!(
                r#"SELECT DISTINCT KEY
FROM {table_name}
WHERE 
    {}
"#,
                sql_where.join(" AND "),
            ),
            params,
        )
        .await?;
    let mut final_result = vec![];
    for item in result {
        let values = item.try_get_by("key")?;
        final_result.push(values);
    }
    Ok(final_result)
}

pub(crate) async fn fact_records_clean(fact_conf_key: &str, before_ct: Option<DateTime<Utc>>, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, _) = common_pg::init_conn(bs_inst).await?;
    conn.begin().await?;
    if !stats_pg_conf_fact_serv::online(fact_conf_key, &conn, ctx).await? {
        return Err(funs.err().conflict("fact_record", "clean", "The fact config not online.", "409-spi-stats-fact-conf-not-online"));
    }

    let table_name = package_table_name(&format!("stats_inst_fact_{fact_conf_key}"), ctx);
    if let Some(before_ct) = before_ct {
        conn.execute_one(&format!("DELETE FROM {table_name} WHERE ct <= $1"), vec![Value::from(before_ct)]).await?;
    } else {
        conn.execute_one(&format!("DELETE FROM {table_name}"), vec![]).await?;
    }
    conn.commit().await?;
    Ok(())
}

pub(crate) async fn dim_record_add(dim_conf_key: String, add_req: StatsDimRecordAddReq, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, _) = common_pg::init_conn(bs_inst).await?;
    conn.begin().await?;
    if !stats_pg_conf_dim_serv::online(&dim_conf_key, &conn, ctx).await? {
        return Err(funs.err().conflict("dim_record", "add", "The dimension config not online.", "409-spi-stats-dim-conf-not-online"));
    }
    let dim_conf = stats_pg_conf_dim_serv::get(&dim_conf_key, None, None, &conn, ctx, inst).await?.expect("Fail to get dim_conf");
    if !dim_conf.stable_ds {
        return Err(funs.err().bad_request(
            "dim_record",
            "add",
            &format!("The dimension config [{}] stable_ds is false, so adding dimension records is not supported.", &dim_conf_key),
            "400-spi-stats-dim-conf-stable-ds-false",
        ));
    }
    if dim_conf.hierarchy.is_empty() && add_req.parent_key.is_some() {
        return Err(funs.err().bad_request(
            "dim_record",
            "add",
            &format!("The dimension config [{}] not allow hierarchy.", &dim_conf_key),
            "400-spi-stats-dim-conf-not-hierarchy",
        ));
    }

    let table_name = package_table_name(&format!("stats_inst_dim_{}", dim_conf.key), ctx);
    let dim_record_key_value = dim_conf.data_type.json_to_sea_orm_value(&add_req.key, false)?;
    if conn.count_by_sql(&format!("SELECT 1 FROM {table_name} WHERE key = $1"), vec![dim_record_key_value.clone()]).await? != 0 {
        return Err(funs.err().conflict(
            "dim_record",
            "add",
            "The dimension instance record already exists, please delete it and then add it.",
            "409-spi-stats-dim-inst-record-exist",
        ));
    }
    let mut sql_fields = vec![];
    let mut params = vec![dim_record_key_value.clone(), Value::from(add_req.show_name.clone())];

    if let Some(parent_key) = add_req.parent_key {
        let parent_record = dim_record_get(&dim_conf_key, parent_key.clone(), &conn, funs, ctx, inst).await?.ok_or_else(|| {
            funs.err().not_found(
                "dim_record",
                "add",
                &format!("The parent dimension instance record [{parent_key}] not exists."),
                "404-spi-stats-dim-inst-record-not-exist",
            )
        })?;
        let parent_hierarchy = parent_record.get("hierarchy").and_then(|x| x.as_u64()).expect("parent_hierarchy missing field hierarchy");
        if (parent_hierarchy + 1) as usize >= dim_conf.hierarchy.len() {
            return Err(funs.err().conflict(
                "dim_record",
                "add",
                "The dimension instance record hierarchy is too deep.",
                "409-spi-stats-dim-inst-record-hierarchy-too-deep",
            ));
        }
        params.push(Value::from(parent_hierarchy + 1));
        sql_fields.push("hierarchy".to_string());
        params.push(dim_record_key_value);
        sql_fields.push(format!("key{}", parent_hierarchy + 1));
        for i in 0..parent_hierarchy + 1 {
            if let Some(record) = parent_record.get(format!("key{i}")).and_then(serde_json::Value::as_str) {
                params.push(record.into());
                sql_fields.push(format!("key{i}"));
            }
        }
    } else if dim_conf.hierarchy.len() > 1 {
        params.push(Value::from(0));
        sql_fields.push("hierarchy".to_string());
        params.push(dim_record_key_value);
        sql_fields.push("key0".to_string());
    }
    conn.execute_one(
        &format!(
            r#"INSERT INTO {table_name}
(key, show_name {})
VALUES
($1, $2 {})
"#,
            if sql_fields.is_empty() { "".to_string() } else { format!(",{}", sql_fields.join(",")) },
            if sql_fields.is_empty() {
                "".to_string()
            } else {
                format!(",{}", sql_fields.iter().enumerate().map(|(i, _)| format!("${}", i + 3)).collect::<Vec<String>>().join(","))
            }
        ),
        params,
    )
    .await?;
    conn.commit().await?;
    Ok(())
}

pub(in crate::serv::pg) async fn dim_record_get(
    dim_conf_key: &str,
    dim_record_key: serde_json::Value,
    conn: &TardisRelDBlConnection,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<Option<serde_json::Value>> {
    dim_do_record_paginate(dim_conf_key.to_string(), Some(dim_record_key), None, 1, 1, None, None, conn, funs, ctx, inst).await.map(|page| page.records.into_iter().next())
}

pub(crate) async fn dim_record_paginate(
    dim_conf_key: String,
    dim_record_key: Option<serde_json::Value>,
    show_name: Option<String>,
    page_number: u32,
    page_size: u32,
    desc_by_create: Option<bool>,
    desc_by_update: Option<bool>,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<TardisPage<serde_json::Value>> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (conn, _) = common_pg::init_conn(bs_inst).await?;
    dim_do_record_paginate(
        dim_conf_key,
        dim_record_key,
        show_name,
        page_number,
        page_size,
        desc_by_create,
        desc_by_update,
        &conn,
        funs,
        ctx,
        inst,
    )
    .await
}

async fn dim_do_record_paginate(
    dim_conf_key: String,
    dim_record_key: Option<serde_json::Value>,
    show_name: Option<String>,
    page_number: u32,
    page_size: u32,
    desc_by_create: Option<bool>,
    desc_by_update: Option<bool>,
    conn: &TardisRelDBlConnection,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<TardisPage<serde_json::Value>> {
    let dim_conf = stats_pg_conf_dim_serv::get(&dim_conf_key, None, None, conn, ctx, inst)
        .await?
        .ok_or_else(|| funs.err().not_found("dim_record", "find", "The dimension config does not exist.", "404-spi-stats-dim-conf-not-exist"))?;

    let table_name = package_table_name(&format!("stats_inst_dim_{dim_conf_key}"), ctx);
    let mut sql_where = vec!["1 = 1".to_string()];
    let mut sql_order = vec![];
    let mut params: Vec<Value> = vec![Value::from(page_size), Value::from((page_number - 1) * page_size)];
    if let Some(dim_record_key) = &dim_record_key {
        sql_where.push(format!("key = ${}", params.len() + 1));
        params.push(dim_conf.data_type.json_to_sea_orm_value(dim_record_key, false)?);
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
                r#"SELECT *, count(*) OVER() AS total
FROM {table_name}
WHERE 
    {}
    {}
LIMIT $1 OFFSET $2
"#,
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
        let values = serde_json::Value::from_query_result_optional(&item, "")?.expect("Fail to get value from query result in dim_do_record_paginate");
        final_result.push(values);
    }
    Ok(TardisPage {
        page_size: page_size as u64,
        page_number: page_number as u64,
        total_size: total_size as u64,
        records: final_result,
    })
}

pub(crate) async fn dim_record_delete(dim_conf_key: String, dim_record_key: serde_json::Value, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, _) = common_pg::init_conn(bs_inst).await?;
    conn.begin().await?;
    if !stats_pg_conf_dim_serv::online(&dim_conf_key, &conn, ctx).await? {
        return Err(funs.err().conflict("dim_record", "delete", "The dimension config not online.", "409-spi-stats-dim-conf-not-online"));
    }
    let dim_conf = stats_pg_conf_dim_serv::get(&dim_conf_key, None, None, &conn, ctx, inst).await?.expect("Fail to get dim_conf");

    let table_name = package_table_name(&format!("stats_inst_dim_{}", dim_conf.key), ctx);
    let values = vec![dim_conf.data_type.json_to_sea_orm_value(&dim_record_key, false)?];

    conn.execute_one(
        &format!(
            r#"UPDATE {table_name}
SET et = now()
WHERE key = $1
"#,
        ),
        values,
    )
    .await?;
    conn.commit().await?;
    Ok(())
}

pub(crate) async fn dim_record_real_delete(
    dim_conf_key: String,
    dim_record_key: serde_json::Value,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, _) = common_pg::init_conn(bs_inst).await?;
    conn.begin().await?;
    if !stats_pg_conf_dim_serv::online(&dim_conf_key, &conn, ctx).await? {
        return Err(funs.err().conflict("dim_record", "delete", "The dimension config not online.", "409-spi-stats-dim-conf-not-online"));
    }
    let dim_conf = stats_pg_conf_dim_serv::get(&dim_conf_key, None, None, &conn, ctx, inst).await?.expect("Fail to get dim_conf");

    let table_name = package_table_name(&format!("stats_inst_dim_{}", dim_conf.key), ctx);
    let values = vec![dim_conf.data_type.json_to_sea_orm_value(&dim_record_key, false)?];

    conn.execute_one(&format!(r#"delete {table_name} WHERE key = $1 "#,), values).await?;
    conn.commit().await?;
    Ok(())
}

async fn fact_get_latest_record_raw(
    fact_conf_key: &str,
    dim_record_key: &str,
    fact_col_conf_set: Vec<StatsConfFactColInfoResp>,
    conn: &TardisRelDBlConnection,
    ctx: &TardisContext,
) -> TardisResult<Option<tardis::db::sea_orm::QueryResult>> {
    let table_name = package_table_name(&format!("stats_inst_fact_{fact_conf_key}"), ctx);
    let mut field_keys = vec!["key".to_string(), "own_paths".to_string(), "ext".to_string(), "ct".to_string(), "idempotent_id".to_string()];
    fact_col_conf_set.iter().for_each(|c| {
        field_keys.push(c.key.clone());
    });
    // let result = conn.query_one(&format!("SELECT * FROM {table_name} WHERE key = $1 ORDER BY ct DESC"), vec![Value::from(dim_record_key)]).await?;
    let result = conn
        .query_one(
            &format!("SELECT {} FROM {table_name} WHERE key = '{dim_record_key}' ORDER BY ct DESC", field_keys.join(",")),
            vec![],
        )
        .await?;
    Ok(result)
}

async fn fact_get_idempotent_record_raw(
    fact_conf_key: &str,
    idempotent_id: &str,
    conn: &TardisRelDBlConnection,
    ctx: &TardisContext,
) -> TardisResult<Option<tardis::db::sea_orm::QueryResult>> {
    let table_name = package_table_name(&format!("stats_inst_fact_{fact_conf_key}"), ctx);
    let result = conn
        .query_one(
            &format!("SELECT idempotent_id FROM {table_name} WHERE idempotent_id = $1 ORDER BY ct DESC"),
            vec![Value::from(idempotent_id)],
        )
        .await?;
    Ok(result)
}

fn merge(a: &mut serde_json::Value, b: serde_json::Value) {
    match (a, b) {
        (a @ &mut serde_json::Value::Object(_), serde_json::Value::Object(b)) => {
            if let Some(a) = a.as_object_mut() {
                for (k, v) in b {
                    merge(a.entry(k).or_insert(serde_json::Value::Null), v);
                }
            }
        }
        (a, b) => *a = b,
    }
}
