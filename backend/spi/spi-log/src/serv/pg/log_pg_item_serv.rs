use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    db::{reldb_client::TardisRelDBClient, sea_orm::Value},
    web::web_resp::TardisPage,
    TardisFuns, TardisFunsInst,
};

use bios_basic::{dto::BasicQueryCondInfo, enumeration::BasicQueryOpKind, helper::db_helper, spi::spi_funs::SpiBsInst};

use crate::dto::log_item_dto::{AdvBasicQueryCondInfo, LogItemAddReq, LogItemFindReq, LogItemFindResp};

use super::log_pg_initializer;

pub async fn add(add_req: &mut LogItemAddReq, _funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<String> {
    let id = add_req.id.clone().unwrap_or(TardisFuns::field.nanoid());
    let mut params = vec![
        Value::from(id.clone()),
        Value::from(add_req.kind.as_ref().unwrap_or(&"".into()).to_string()),
        Value::from(add_req.key.as_ref().unwrap_or(&"".into()).to_string()),
        Value::from(add_req.op.as_ref().unwrap_or(&"".to_string()).as_str()),
        Value::from(add_req.content.as_str()),
        Value::from(add_req.owner.as_ref().unwrap_or(&"".to_string()).as_str()),
        Value::from(add_req.own_paths.as_ref().unwrap_or(&"".to_string()).as_str()),
        Value::from(if let Some(ext) = &add_req.ext {
            ext.clone()
        } else {
            TardisFuns::json.str_to_json("{}")?
        }),
        Value::from(add_req.rel_key.as_ref().unwrap_or(&"".into()).to_string()),
    ];
    if let Some(ts) = add_req.ts {
        params.push(Value::from(ts));
    }

    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = log_pg_initializer::init_table_and_conn(bs_inst, &add_req.tag, ctx, true).await?;
    conn.begin().await?;
    conn.execute_one(
        &format!(
            r#"INSERT INTO {table_name} 
    (id, kind, key, op, content, owner, own_paths, ext, rel_key{})
VALUES
    ($1, $2, $3, $4, $5, $6, $7, $8, $9{})
	"#,
            if add_req.ts.is_some() { ", ts" } else { "" },
            if add_req.ts.is_some() { ", $10" } else { "" },
        ),
        params,
    )
    .await?;
    conn.commit().await?;
    Ok(id)
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
            let value = db_helper::json_to_sea_orm_value(&ext_item.value, ext_item.op == BasicQueryOpKind::Like);
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
                where_fragments.push(format!("ext ->> '{}' is not null", ext_item.field));
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
            let value = db_helper::json_to_sea_orm_value(&ext_or_item.value, ext_or_item.op == BasicQueryOpKind::Like);

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
                    let value = db_helper::json_to_sea_orm_value(&ext_item.value, ext_item.op == BasicQueryOpKind::Like || ext_item.op == BasicQueryOpKind::NotLike);
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
                    } else {
                        if ext_item.op == BasicQueryOpKind::In {
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
                            sql_and_where.push(format!("{} is not null", ext_item.field));
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
                r#"SELECT ts, id, key, op, content, kind, ext, owner, own_paths, rel_key, count(*) OVER() AS total
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

    let result = result
        .into_iter()
        .map(|item| {
            if total_size == 0 {
                total_size = item.try_get("", "total")?;
            }
            Ok(LogItemFindResp {
                ts: item.try_get("", "ts")?,
                id: item.try_get("", "id")?,
                key: item.try_get("", "key")?,
                op: item.try_get("", "op")?,
                ext: item.try_get("", "ext")?,
                content: item.try_get("", "content")?,
                rel_key: item.try_get("", "rel_key")?,
                kind: item.try_get("", "kind")?,
                owner: item.try_get("", "owner")?,
                own_paths: item.try_get("", "own_paths")?,
            })
        })
        .collect::<TardisResult<Vec<_>>>()?;

    Ok(TardisPage {
        page_size: find_req.page_size as u64,
        page_number: find_req.page_number as u64,
        total_size: total_size as u64,
        records: result,
    })
}
