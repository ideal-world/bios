use std::{collections::HashMap, str::FromStr, vec};

use bios_sdk_invoke::clients::event_client::{get_topic, mq_error, EventAttributeExt as _, SPI_RPC_TOPIC};
use tardis::{
    basic::{dto::TardisContext, error::TardisError, result::TardisResult},
    chrono::{DateTime, Utc},
    db::{
        reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
        sea_orm::Value,
    },
    futures::TryFutureExt as _,
    serde_json::{self, Value as JsonValue},
    web::web_resp::TardisPage,
    TardisFuns, TardisFunsInst,
};

use bios_basic::{
    dto::BasicQueryCondInfo,
    enumeration::BasicQueryOpKind,
    helper::db_helper,
    spi::{spi_funs::SpiBsInst, spi_initializer::common_pg::get_schema_name_from_ext},
};

use crate::{
    dto::log_item_dto::{AdvBasicQueryCondInfo, LogConfigReq, LogItemAddReq, LogItemFindReq, LogItemFindResp, StatsItemAddReq},
    log_constants::{CONFIG_TABLE_NAME, LOG_REF_FLAG, TABLE_LOG_FLAG_V2},
};

use super::log_pg_initializer;

pub async fn add(add_req: &mut LogItemAddReq, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<String> {
    let id = add_req.idempotent_id.clone().unwrap_or(TardisFuns::field.nanoid());

    let bs_inst = inst.inst::<TardisRelDBClient>();
    let mut insert_content = add_req.content.clone();
    let (mut conn, table_name) = log_pg_initializer::init_table_and_conn(bs_inst, &add_req.tag, ctx, true).await?;
    conn.begin().await?;
    let ref_fields = get_ref_fields_by_table_name(&conn, &get_schema_name_from_ext(&inst.ext).expect("ignore"), &table_name).await?;
    if let Some(key) = add_req.key.as_ref() {
        let get_last_record = conn
            .query_one(
                &format!(
                    r#"
  select ts,key,content from {table_name} where key = $1 order by ts desc limit 1
  "#
                ),
                vec![Value::from(key.to_string())],
            )
            .await?;

        if let Some(last_record) = get_last_record {
            let last_content: JsonValue = last_record.try_get("", "content")?;
            let last_ts: DateTime<Utc> = last_record.try_get("", "ts")?;
            let last_key: String = last_record.try_get("", "key")?;

            insert_content = last_content;
            for ref_field in &ref_fields {
                if let Some(field_value) = insert_content.get_mut(ref_field) {
                    if !is_log_ref(field_value) {
                        *field_value = JsonValue::String(get_ref_filed_value(&last_ts, &last_key));
                    }
                }
            }

            if let (Some(insert_content), Some(add_req_content)) = (insert_content.as_object_mut(), add_req.content.as_object()) {
                for (k, v) in add_req_content {
                    insert_content.insert(k.to_string(), v.clone());
                }
            }
        }
    }

    let mut params = vec![
        Value::from(id.clone()),
        Value::from(add_req.kind.as_ref().unwrap_or(&"".into()).to_string()),
        Value::from(add_req.key.as_ref().unwrap_or(&"".into()).to_string()),
        Value::from(add_req.tag.clone()),
        Value::from(add_req.op.as_ref().unwrap_or(&"".to_string()).as_str()),
        Value::from(insert_content),
        Value::from(add_req.owner.as_ref().unwrap_or(&"".to_string()).as_str()),
        Value::from(add_req.owner_name.as_ref().unwrap_or(&"".to_string()).as_str()),
        Value::from(add_req.own_paths.as_ref().unwrap_or(&"".to_string()).as_str()),
        Value::from(add_req.push),
        Value::from(if let Some(ext) = &add_req.ext {
            ext.clone()
        } else {
            TardisFuns::json.str_to_json("{}")?
        }),
        Value::from(add_req.rel_key.as_ref().unwrap_or(&"".into()).to_string()),
        Value::from(add_req.msg.as_ref().unwrap_or(&"".into()).as_str()),
    ];
    if let Some(ts) = add_req.ts {
        params.push(Value::from(ts));
    }
    conn.execute_one(
        &format!(
            r#"INSERT INTO {table_name}
  (idempotent_id, kind, key, tag, op, content, owner, owner_name, own_paths, push, ext, rel_key, msg{})
VALUES
  ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13{})
"#,
            if add_req.ts.is_some() { ", ts" } else { "" },
            if add_req.ts.is_some() { ", $14" } else { "" },
        ),
        params,
    )
    .await?;
    conn.commit().await?;
    //if push is true, then push to EDA
    if add_req.push {
        push_to_eda(&add_req, &ref_fields, funs, ctx).await?;
    }
    Ok(id)
}

fn get_ref_filed_value(ref_log_record_ts: &DateTime<Utc>, ref_log_record_key: &str) -> String {
    let ref_log_record_ts = ref_log_record_ts.to_string();
    format!("{LOG_REF_FLAG}@{ref_log_record_ts}#{ref_log_record_key}")
}

/// check if the value is referenced
/// true if the value is referenced
fn is_log_ref(value: &JsonValue) -> bool {
    if let Some(value_str) = value.as_str() {
        if value_str.starts_with(LOG_REF_FLAG) {
            return true;
        }
    }
    false
}

fn parse_ref_ts_key(ref_key: &str) -> TardisResult<(DateTime<Utc>, String)> {
    let split_vec: Vec<&str> = ref_key.split("@").collect();
    if split_vec.len() != 2 {
        return Err(TardisError::format_error(&format!("ref_key:{ref_key} format error"), ""));
    }
    let split_vec: Vec<&str> = split_vec[1].split("#").collect();
    if split_vec.len() != 2 {
        return Err(TardisError::format_error(&format!("ref_key:{ref_key} format error"), ""));
    }
    Ok((
        DateTime::from_str(split_vec[0]).map_err(|e| TardisError::wrap(&format!("parse ts:{} error:{e}", split_vec[0]), ""))?,
        split_vec[1].to_string(),
    ))
}

pub async fn find(find_req: &mut LogItemFindReq, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<TardisPage<LogItemFindResp>> {
    let mut where_fragments: Vec<String> = Vec::new();
    let mut sql_vals: Vec<Value> = vec![];

    if let Some(kinds) = &find_req.kinds {
        let place_holder = kinds
            .iter()
            .map(|kind| {
                sql_vals.push(Value::from(kind.to_string()));
                format!("${}", sql_vals.len())
            })
            .collect::<Vec<String>>()
            .join(",");
        where_fragments.push(format!("kind IN ({place_holder})"));
    }
    if let Some(owners) = &find_req.owners {
        let place_holder = owners
            .iter()
            .map(|owner| {
                sql_vals.push(Value::from(owner.to_string()));
                format!("${}", sql_vals.len())
            })
            .collect::<Vec<String>>()
            .join(",");
        where_fragments.push(format!("owner IN ({place_holder})"));
    }
    if let Some(keys) = &find_req.keys {
        let place_holder = keys
            .iter()
            .map(|key| {
                sql_vals.push(Value::from(key.to_string()));
                format!("${}", sql_vals.len())
            })
            .collect::<Vec<String>>()
            .join(",");
        where_fragments.push(format!("key IN ({place_holder})"));
    }
    if let Some(ops) = &find_req.ops {
        let place_holder = ops
            .iter()
            .map(|op| {
                sql_vals.push(Value::from(op.as_str()));
                format!("${}", sql_vals.len())
            })
            .collect::<Vec<String>>()
            .join(",");
        where_fragments.push(format!("op IN ({place_holder})"));
    }
    if let Some(rel_keys) = &find_req.rel_keys {
        let place_holder = rel_keys
            .iter()
            .map(|rel_key| {
                sql_vals.push(Value::from(rel_key.to_string()));
                format!("${}", sql_vals.len())
            })
            .collect::<Vec<String>>()
            .join(",");
        where_fragments.push(format!("rel_key IN ({place_holder})"));
    }
    if let Some(own_paths) = &find_req.own_paths {
        sql_vals.push(Value::from(format!("{}%", own_paths)));
        where_fragments.push(format!("own_paths like ${}", sql_vals.len()));
    }
    if let Some(ts_start) = find_req.ts_start {
        sql_vals.push(Value::from(ts_start));
        where_fragments.push(format!("ts >= ${}", sql_vals.len()));
    }
    if let Some(ts_end) = find_req.ts_end {
        sql_vals.push(Value::from(ts_end));
        where_fragments.push(format!("ts <= ${}", sql_vals.len()));
    }
    let err_notfound = |ext: &BasicQueryCondInfo| {
        Err(funs.err().not_found(
            "item",
            "log",
            &format!("The ext field=[{}] value=[{}] operation=[{}] is not legal.", &ext.field, ext.value, &ext.op,),
            "404-spi-log-op-not-legal",
        ))
    };
    let err_op_in_without_value = || Err(funs.err().bad_request("item", "log", "Request item using 'IN' operator show hava a value", "400-spi-item-op-in-without-value"));
    if let Some(ext) = &find_req.ext {
        for ext_item in ext {
            let value = db_helper::json_to_sea_orm_value(&ext_item.value, &ext_item.op);
            let Some(mut value) = value else {
                return err_notfound(ext_item);
            };
            if ext_item.op == BasicQueryOpKind::In {
                if value.len() == 1 {
                    where_fragments.push(format!("ext -> '{}' ? ${}", ext_item.field, sql_vals.len() + 1));
                } else {
                    where_fragments.push(format!(
                        "ext -> '{}' ?| array[{}]",
                        ext_item.field,
                        (0..value.len()).map(|idx| format!("${}", sql_vals.len() + idx + 1)).collect::<Vec<String>>().join(", ")
                    ));
                }
                for val in value {
                    sql_vals.push(val);
                }
            } else if ext_item.op == BasicQueryOpKind::NotIn {
                let value = value.clone();
                if value.len() == 1 {
                    where_fragments.push(format!("not (ext -> '{}' ? ${})", ext_item.field, sql_vals.len() + 1));
                } else {
                    where_fragments.push(format!(
                        "not (ext -> '{}' ?| array[{}])",
                        ext_item.field,
                        (0..value.len()).map(|idx| format!("${}", sql_vals.len() + idx + 1)).collect::<Vec<String>>().join(", ")
                    ));
                }
                for val in value {
                    sql_vals.push(val);
                }
            } else if ext_item.op == BasicQueryOpKind::IsNull {
                where_fragments.push(format!("ext ->> '{}' is null", ext_item.field));
            } else if ext_item.op == BasicQueryOpKind::IsNotNull {
                where_fragments.push(format!("(ext ->> '{}' is not null or ext ->> '{}' != '')", ext_item.field, ext_item.field));
            } else if ext_item.op == BasicQueryOpKind::IsNullOrEmpty {
                where_fragments.push(format!("(ext ->> '{}' is null or ext ->> '{}' = '')", ext_item.field, ext_item.field));
            } else {
                if value.len() > 1 {
                    return err_notfound(ext_item);
                }
                let Some(value) = value.pop() else {
                    return err_op_in_without_value();
                };
                if let Value::Bool(_) = value {
                    where_fragments.push(format!("(ext ->> '{}')::boolean {} ${}", ext_item.field, ext_item.op.to_sql(), sql_vals.len() + 1));
                } else if let Value::TinyInt(_) = value {
                    where_fragments.push(format!("(ext ->> '{}')::smallint {} ${}", ext_item.field, ext_item.op.to_sql(), sql_vals.len() + 1));
                } else if let Value::SmallInt(_) = value {
                    where_fragments.push(format!("(ext ->> '{}')::smallint {} ${}", ext_item.field, ext_item.op.to_sql(), sql_vals.len() + 1));
                } else if let Value::Int(_) = value {
                    where_fragments.push(format!("(ext ->> '{}')::integer {} ${}", ext_item.field, ext_item.op.to_sql(), sql_vals.len() + 1));
                } else if let Value::BigInt(_) = value {
                    where_fragments.push(format!("(ext ->> '{}')::bigint {} ${}", ext_item.field, ext_item.op.to_sql(), sql_vals.len() + 1));
                } else if let Value::TinyUnsigned(_) = value {
                    where_fragments.push(format!("(ext ->> '{}')::smallint {} ${}", ext_item.field, ext_item.op.to_sql(), sql_vals.len() + 1));
                } else if let Value::SmallUnsigned(_) = value {
                    where_fragments.push(format!("(ext ->> '{}')::integer {} ${}", ext_item.field, ext_item.op.to_sql(), sql_vals.len() + 1));
                } else if let Value::Unsigned(_) = value {
                    where_fragments.push(format!("(ext ->> '{}')::bigint {} ${}", ext_item.field, ext_item.op.to_sql(), sql_vals.len() + 1));
                } else if let Value::BigUnsigned(_) = value {
                    // TODO
                    where_fragments.push(format!("(ext ->> '{}')::bigint {} ${}", ext_item.field, ext_item.op.to_sql(), sql_vals.len() + 1));
                } else if let Value::Float(_) = value {
                    where_fragments.push(format!("(ext ->> '{}')::real {} ${}", ext_item.field, ext_item.op.to_sql(), sql_vals.len() + 1));
                } else if let Value::Double(_) = value {
                    where_fragments.push(format!("(ext ->> '{}')::double precision {} ${}", ext_item.field, ext_item.op.to_sql(), sql_vals.len() + 1));
                } else {
                    where_fragments.push(format!("ext ->> '{}' {} ${}", ext_item.field, ext_item.op.to_sql(), sql_vals.len() + 1));
                }
                sql_vals.push(value);
            }
        }
    }
    if let Some(ext_or) = &find_req.ext_or {
        let mut or_fragments = vec![];
        for ext_or_item in ext_or {
            let value = db_helper::json_to_sea_orm_value(&ext_or_item.value, &ext_or_item.op);

            let Some(mut value) = value else {
                return err_notfound(ext_or_item);
            };
            if ext_or_item.op == BasicQueryOpKind::In {
                if value.len() == 1 {
                    or_fragments.push(format!("ext -> '{}' ? ${}", ext_or_item.field, sql_vals.len() + 1));
                } else {
                    or_fragments.push(format!(
                        "ext -> '{}' ?| array[{}]",
                        ext_or_item.field,
                        (0..value.len()).map(|idx| format!("${}", sql_vals.len() + idx + 1)).collect::<Vec<String>>().join(", ")
                    ));
                }
                for val in value {
                    sql_vals.push(val);
                }
            } else {
                if value.len() > 1 {
                    return err_notfound(ext_or_item);
                }
                let Some(value) = value.pop() else {
                    return err_op_in_without_value();
                };
                if let Value::Bool(_) = value {
                    or_fragments.push(format!("(ext ->> '{}')::boolean {} ${}", ext_or_item.field, ext_or_item.op.to_sql(), sql_vals.len() + 1));
                } else if let Value::TinyInt(_) = value {
                    or_fragments.push(format!("(ext ->> '{}')::smallint {} ${}", ext_or_item.field, ext_or_item.op.to_sql(), sql_vals.len() + 1));
                } else if let Value::SmallInt(_) = value {
                    or_fragments.push(format!("(ext ->> '{}')::smallint {} ${}", ext_or_item.field, ext_or_item.op.to_sql(), sql_vals.len() + 1));
                } else if let Value::Int(_) = value {
                    or_fragments.push(format!("(ext ->> '{}')::integer {} ${}", ext_or_item.field, ext_or_item.op.to_sql(), sql_vals.len() + 1));
                } else if let Value::BigInt(_) = value {
                    or_fragments.push(format!("(ext ->> '{}')::bigint {} ${}", ext_or_item.field, ext_or_item.op.to_sql(), sql_vals.len() + 1));
                } else if let Value::TinyUnsigned(_) = value {
                    or_fragments.push(format!("(ext ->> '{}')::smallint {} ${}", ext_or_item.field, ext_or_item.op.to_sql(), sql_vals.len() + 1));
                } else if let Value::SmallUnsigned(_) = value {
                    or_fragments.push(format!("(ext ->> '{}')::integer {} ${}", ext_or_item.field, ext_or_item.op.to_sql(), sql_vals.len() + 1));
                } else if let Value::Unsigned(_) = value {
                    or_fragments.push(format!("(ext ->> '{}')::bigint {} ${}", ext_or_item.field, ext_or_item.op.to_sql(), sql_vals.len() + 1));
                } else if let Value::BigUnsigned(_) = value {
                    // TODO
                    or_fragments.push(format!("(ext ->> '{}')::bigint {} ${}", ext_or_item.field, ext_or_item.op.to_sql(), sql_vals.len() + 1));
                } else if let Value::Float(_) = value {
                    or_fragments.push(format!("(ext ->> '{}')::real {} ${}", ext_or_item.field, ext_or_item.op.to_sql(), sql_vals.len() + 1));
                } else if let Value::Double(_) = value {
                    or_fragments.push(format!(
                        "(ext ->> '{}')::double precision {} ${}",
                        ext_or_item.field,
                        ext_or_item.op.to_sql(),
                        sql_vals.len() + 1
                    ));
                } else {
                    or_fragments.push(format!("ext ->> '{}' {} ${}", ext_or_item.field, ext_or_item.op.to_sql(), sql_vals.len() + 1));
                }
                sql_vals.push(value);
            }
        }
        where_fragments.push(format!(" ( {} ) ", or_fragments.join(" OR ")));
    }

    // advanced query
    let mut sql_adv_query = vec![];
    if let Some(adv_query) = &find_req.adv_query {
        for group_query in adv_query {
            let mut sql_and_where = vec![];
            let err_not_found = |ext_item: &AdvBasicQueryCondInfo| {
                Err(funs.err().not_found(
                    "item",
                    "search",
                    &format!("The ext field=[{}] value=[{}] operation=[{}] is not legal.", &ext_item.field, ext_item.value, &ext_item.op,),
                    "404-spi-search-op-not-legal",
                ))
            };
            if let Some(ext) = &group_query.ext {
                for ext_item in ext {
                    let value = db_helper::json_to_sea_orm_value(&ext_item.value, &ext_item.op);
                    let Some(mut value) = value else { return err_not_found(ext_item) };
                    if ext_item.in_ext.unwrap_or(true) {
                        if ext_item.op == BasicQueryOpKind::In {
                            if value.len() == 1 {
                                sql_and_where.push(format!("ext -> '{}' ? ${}", ext_item.field, sql_vals.len() + 1));
                            } else {
                                sql_and_where.push(format!(
                                    "ext -> '{}' ?| array[{}]",
                                    ext_item.field,
                                    (0..value.len()).map(|idx| format!("${}", sql_vals.len() + idx + 1)).collect::<Vec<String>>().join(", ")
                                ));
                            }
                            for val in value {
                                sql_vals.push(val);
                            }
                        } else if ext_item.op == BasicQueryOpKind::NotIn {
                            let value = value.clone();
                            if value.len() == 1 {
                                sql_and_where.push(format!("not (ext -> '{}' ? ${})", ext_item.field, sql_vals.len() + 1));
                            } else {
                                sql_and_where.push(format!(
                                    "not (ext -> '{}' ?| array[{}])",
                                    ext_item.field,
                                    (0..value.len()).map(|idx| format!("${}", sql_vals.len() + idx + 1)).collect::<Vec<String>>().join(", ")
                                ));
                            }
                            for val in value {
                                sql_vals.push(val);
                            }
                        } else if ext_item.op == BasicQueryOpKind::IsNull {
                            sql_and_where.push(format!("ext ->> '{}' is null", ext_item.field));
                        } else if ext_item.op == BasicQueryOpKind::IsNotNull {
                            sql_and_where.push(format!("ext ->> '{}' is not null", ext_item.field));
                        } else if ext_item.op == BasicQueryOpKind::IsNullOrEmpty {
                            sql_and_where.push(format!("(ext ->> '{}' is null or ext ->> '{}' = '' or (jsonb_typeof(ext -> '{}') = 'array' and (jsonb_array_length(ext-> '{}') is null or jsonb_array_length(ext-> '{}') = 0)))", ext_item.field, ext_item.field, ext_item.field, ext_item.field, ext_item.field));
                        } else {
                            if value.len() > 1 {
                                return err_not_found(ext_item);
                            }
                            let Some(value) = value.pop() else {
                                return Err(funs.err().bad_request("item", "search", "Request item using 'IN' operator show hava a value", "400-spi-item-op-in-without-value"));
                            };
                            if let Value::Bool(_) = value {
                                sql_and_where.push(format!("(ext ->> '{}')::boolean {} ${}", ext_item.field, ext_item.op.to_sql(), sql_vals.len() + 1));
                            } else if let Value::TinyInt(_) = value {
                                sql_and_where.push(format!("(ext ->> '{}')::smallint {} ${}", ext_item.field, ext_item.op.to_sql(), sql_vals.len() + 1));
                            } else if let Value::SmallInt(_) = value {
                                sql_and_where.push(format!("(ext ->> '{}')::smallint {} ${}", ext_item.field, ext_item.op.to_sql(), sql_vals.len() + 1));
                            } else if let Value::Int(_) = value {
                                sql_and_where.push(format!("(ext ->> '{}')::integer {} ${}", ext_item.field, ext_item.op.to_sql(), sql_vals.len() + 1));
                            } else if let Value::BigInt(_) = value {
                                sql_and_where.push(format!("(ext ->> '{}')::bigint {} ${}", ext_item.field, ext_item.op.to_sql(), sql_vals.len() + 1));
                            } else if let Value::TinyUnsigned(_) = value {
                                sql_and_where.push(format!("(ext ->> '{}')::smallint {} ${}", ext_item.field, ext_item.op.to_sql(), sql_vals.len() + 1));
                            } else if let Value::SmallUnsigned(_) = value {
                                sql_and_where.push(format!("(ext ->> '{}')::integer {} ${}", ext_item.field, ext_item.op.to_sql(), sql_vals.len() + 1));
                            } else if let Value::Unsigned(_) = value {
                                sql_and_where.push(format!("(ext ->> '{}')::bigint {} ${}", ext_item.field, ext_item.op.to_sql(), sql_vals.len() + 1));
                            } else if let Value::BigUnsigned(_) = value {
                                // TODO
                                sql_and_where.push(format!("(ext ->> '{}')::bigint {} ${}", ext_item.field, ext_item.op.to_sql(), sql_vals.len() + 1));
                            } else if let Value::Float(_) = value {
                                sql_and_where.push(format!("(ext ->> '{}')::real {} ${}", ext_item.field, ext_item.op.to_sql(), sql_vals.len() + 1));
                            } else if let Value::Double(_) = value {
                                sql_and_where.push(format!("(ext ->> '{}')::double precision {} ${}", ext_item.field, ext_item.op.to_sql(), sql_vals.len() + 1));
                            } else if value.is_chrono_date_time_utc() {
                                sql_and_where.push(format!(
                                    "(ext ->> '{}')::timestamp with time zone {} ${}",
                                    ext_item.field,
                                    ext_item.op.to_sql(),
                                    sql_vals.len() + 1
                                ));
                            } else {
                                sql_and_where.push(format!("ext ->> '{}' {} ${}", ext_item.field, ext_item.op.to_sql(), sql_vals.len() + 1));
                            }
                            sql_vals.push(value);
                        }
                    } else if ext_item.op == BasicQueryOpKind::In {
                        if !value.is_empty() {
                            sql_and_where.push(format!(
                                "{} IN ({})",
                                ext_item.field,
                                (0..value.len()).map(|idx| format!("${}", sql_vals.len() + idx + 1)).collect::<Vec<String>>().join(",")
                            ));
                            for val in value {
                                sql_vals.push(val);
                            }
                        }
                    } else if ext_item.op == BasicQueryOpKind::NotIn {
                        if !value.is_empty() {
                            sql_and_where.push(format!(
                                "{} NOT IN ({})",
                                ext_item.field,
                                (0..value.len()).map(|idx| format!("${}", sql_vals.len() + idx + 1)).collect::<Vec<String>>().join(",")
                            ));
                            for val in value {
                                sql_vals.push(val);
                            }
                        }
                    } else if ext_item.op == BasicQueryOpKind::IsNull {
                        sql_and_where.push(format!("{} is null", ext_item.field));
                    } else if ext_item.op == BasicQueryOpKind::IsNotNull {
                        sql_and_where.push(format!("({} is not null or {} != '' )", ext_item.field, ext_item.field));
                    } else if ext_item.op == BasicQueryOpKind::IsNullOrEmpty {
                        sql_and_where.push(format!("({} is null or {} = '' )", ext_item.field, ext_item.field));
                    } else {
                        if value.len() > 1 {
                            return err_not_found(ext_item);
                        }
                        let Some(value) = value.pop() else {
                            return Err(funs.err().bad_request("item", "search", "Request item using 'IN' operator show hava a value", "400-spi-item-op-in-without-value"));
                        };
                        if let Value::Bool(_) = value {
                            sql_and_where.push(format!("({}::boolean) {} ${}", ext_item.field, ext_item.op.to_sql(), sql_vals.len() + 1));
                        } else if let Value::TinyInt(_) = value {
                            sql_and_where.push(format!("({}::smallint) {} ${}", ext_item.field, ext_item.op.to_sql(), sql_vals.len() + 1));
                        } else if let Value::SmallInt(_) = value {
                            sql_and_where.push(format!("({}::smallint) {} ${}", ext_item.field, ext_item.op.to_sql(), sql_vals.len() + 1));
                        } else if let Value::Int(_) = value {
                            sql_and_where.push(format!("({}::integer) {} ${}", ext_item.field, ext_item.op.to_sql(), sql_vals.len() + 1));
                        } else if let Value::BigInt(_) = value {
                            sql_and_where.push(format!("({}::bigint) {} ${}", ext_item.field, ext_item.op.to_sql(), sql_vals.len() + 1));
                        } else if let Value::TinyUnsigned(_) = value {
                            sql_and_where.push(format!("({}::smallint) {} ${}", ext_item.field, ext_item.op.to_sql(), sql_vals.len() + 1));
                        } else if let Value::SmallUnsigned(_) = value {
                            sql_and_where.push(format!("({}::integer) {} ${}", ext_item.field, ext_item.op.to_sql(), sql_vals.len() + 1));
                        } else if let Value::Unsigned(_) = value {
                            sql_and_where.push(format!("({}::bigint) {} ${}", ext_item.field, ext_item.op.to_sql(), sql_vals.len() + 1));
                        } else if let Value::BigUnsigned(_) = value {
                            // TODO
                            sql_and_where.push(format!("({}::bigint) {} ${}", ext_item.field, ext_item.op.to_sql(), sql_vals.len() + 1));
                        } else if let Value::Float(_) = value {
                            sql_and_where.push(format!("({}::real) {} ${}", ext_item.field, ext_item.op.to_sql(), sql_vals.len() + 1));
                        } else if let Value::Double(_) = value {
                            sql_and_where.push(format!("({}::double precision) {} ${}", ext_item.field, ext_item.op.to_sql(), sql_vals.len() + 1));
                        } else {
                            sql_and_where.push(format!("{} {} ${}", ext_item.field, ext_item.op.to_sql(), sql_vals.len() + 1));
                        }
                        sql_vals.push(value);
                    }
                }
            }
            if !sql_and_where.is_empty() {
                sql_adv_query.push(format!(
                    " {} ( {} )",
                    if group_query.group_by_or.unwrap_or(false) { "OR" } else { "AND" },
                    sql_and_where.join(" AND ")
                ));
            }
        }
    }
    if where_fragments.is_empty() {
        where_fragments.push("1 = 1".to_string());
    }

    sql_vals.push(Value::from(find_req.page_size));
    sql_vals.push(Value::from((find_req.page_number - 1) * find_req.page_size as u32));
    let page_fragments = format!("LIMIT ${} OFFSET ${}", sql_vals.len() - 1, sql_vals.len());

    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (conn, table_name) = log_pg_initializer::init_table_and_conn(bs_inst, &find_req.tag, ctx, false).await?;
    let result = conn
        .query_all(
            format!(
                r#"SELECT ts, idempotent_id, key, op, content, kind, ext, owner, owner_name, own_paths, rel_key, msg, count(*) OVER() AS total
FROM {table_name}
WHERE
  {}
  {}
ORDER BY ts DESC
{}"#,
                where_fragments.join(" AND "),
                if sql_adv_query.is_empty() {
                    "".to_string()
                } else {
                    format!(" AND ( 1=1 {})", sql_adv_query.join(" "))
                },
                page_fragments
            )
            .as_str(),
            sql_vals,
        )
        .await?;

    let mut total_size: i64 = 0;

    let mut result = result
        .into_iter()
        .map(|item| {
            if total_size == 0 {
                total_size = item.try_get("", "total")?;
            }
            Ok(LogItemFindResp {
                ts: item.try_get("", "ts")?,
                id: item.try_get("", "idempotent_id")?,
                key: item.try_get("", "key")?,
                op: item.try_get("", "op")?,
                ext: item.try_get("", "ext")?,
                content: item.try_get("", "content")?,
                rel_key: item.try_get("", "rel_key")?,
                kind: item.try_get("", "kind")?,
                owner: item.try_get("", "owner")?,
                own_paths: item.try_get("", "own_paths")?,
                msg: item.try_get("", "msg")?,
                owner_name: item.try_get("", "owner_name")?,
            })
        })
        .collect::<TardisResult<Vec<_>>>()?;

    // Stores the mapping relationship between ref_value and true_value
    let mut cache_value = HashMap::<String, JsonValue>::new();

    for log in &mut result {
        if let Some(json_map) = log.content.as_object_mut() {
            for (k, v) in json_map {
                if is_log_ref(v) {
                    if let Some(v_str) = v.as_str() {
                        let ref_string = v_str.to_string();
                        if let Some(true_content) = cache_value.get(&ref_string) {
                            if let Some(true_value) = true_content.get(k) {
                                *v = true_value.clone();
                            }
                        } else {
                            let (ts, key) = parse_ref_ts_key(v_str)?;
                            if let Some(query_result) =
                                conn.query_one(&format!("select content from {table_name} where ts=$1 and key=$2"), vec![Value::from(ts), Value::from(key)]).await?
                            {
                                let query_content: JsonValue = query_result.try_get("", "content")?;
                                if let Some(query_true_value) = query_content.get(k) {
                                    *v = query_true_value.clone();
                                }
                                cache_value.insert(ref_string, query_content);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(TardisPage {
        page_size: find_req.page_size as u64,
        page_number: find_req.page_number as u64,
        total_size: total_size as u64,
        records: result,
    })
}

pub async fn modify_ext(tag: &str, key: &str, ext: &mut JsonValue, _funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (conn, table_name) = log_pg_initializer::init_table_and_conn(bs_inst, tag, ctx, false).await?;

    let ext_str: String = serde_json::to_string(ext).map_err(|e| TardisError::internal_error(&format!("Fail to parse ext: {e}"), "500-spi-log-internal-error"))?;
    conn.execute_one(&format!("update {table_name} set ext=ext||$1 where key=$2"), vec![Value::from(ext_str), Value::from(key)]).await?;
    Ok(())
}

pub async fn add_config(req: &LogConfigReq, _funs: &TardisFunsInst, _ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let table_full_name = bios_basic::spi::spi_initializer::common_pg::get_table_full_name(&inst.ext, TABLE_LOG_FLAG_V2.to_string(), req.tag.clone());
    let schema_name = get_schema_name_from_ext(&inst.ext).expect("ignore");
    let bs_inst = inst.inst::<TardisRelDBClient>();
    if bs_inst
        .0
        .conn()
        .query_one(
            &format!("select table_name,ref_field from {schema_name}.{CONFIG_TABLE_NAME} where table_name = $1 and ref_field = $2"),
            vec![Value::from(table_full_name.clone()), Value::from(req.ref_field.clone())],
        )
        .await?
        .is_some()
    {
        Ok(())
    } else {
        //新增记录
        bs_inst
            .0
            .conn()
            .execute_one(
                &format!("insert into {schema_name}.{CONFIG_TABLE_NAME}(table_name,ref_field) VALUES ($1,$2)"),
                vec![Value::from(table_full_name), Value::from(req.ref_field.clone())],
            )
            .await?;
        Ok(())
    }
}

pub async fn delete_config(config: &mut LogConfigReq, _funs: &TardisFunsInst, _ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let table_full_name = bios_basic::spi::spi_initializer::common_pg::get_table_full_name(&inst.ext, TABLE_LOG_FLAG_V2.to_string(), config.tag.clone());
    let schema_name = get_schema_name_from_ext(&inst.ext).expect("ignore");
    let bs_inst = inst.inst::<TardisRelDBClient>();
    bs_inst
        .0
        .conn()
        .execute_one(
            &format!("delete from {schema_name}.{CONFIG_TABLE_NAME} where table_name = $1 and ref_field = $2"),
            vec![Value::from(table_full_name), Value::from(config.ref_field.clone())],
        )
        .await?;
    Ok(())
}

async fn get_ref_fields_by_table_name(conn: &TardisRelDBlConnection, schema_name: &str, table_full_name: &str) -> TardisResult<Vec<String>> {
    let query_results = conn
        .query_all(
            &format!("select ref_field from {schema_name}.{CONFIG_TABLE_NAME} where table_name = $1"),
            vec![Value::from(table_full_name)],
        )
        .await?;

    let mut ref_fields = Vec::new();
    for row in query_results {
        let ref_field: String = row.try_get("", "ref_field")?;
        ref_fields.push(ref_field);
    }

    Ok(ref_fields)
}

async fn push_to_eda(req: &LogItemAddReq, ref_fields: &Vec<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    if let Some(topic) = get_topic(&SPI_RPC_TOPIC) {
        let mut req_clone = req.clone();
        for ref_field in ref_fields {
            if let Some(content) = req_clone.content.as_object_mut() {
                content.remove(ref_field);
            }
        }
        let stats_add: StatsItemAddReq = req_clone.into();
        topic.send_event(stats_add.inject_context(funs, ctx).json()).map_err(mq_error).await?;
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use tardis::{chrono::Utc, serde_json::Value};

    use crate::serv::pgv2::log_pg_item_serv::{is_log_ref, parse_ref_ts_key};

    use super::get_ref_filed_value;

    #[test]
    fn test_ref_value() {
        let ts = Utc::now();
        let key = "test-key".to_owned();
        let ref_value = get_ref_filed_value(&ts, &key);

        assert!(is_log_ref(&Value::String(ref_value.clone())));
        assert!(!is_log_ref(&Value::String(key.to_string())));
        assert_eq!(parse_ref_ts_key(&ref_value).unwrap(), (ts, key));
    }
}
