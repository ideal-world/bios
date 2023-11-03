use std::collections::HashMap;

use pinyin::{to_pinyin_vec, Pinyin};
use tardis::{
    basic::{dto::TardisContext, error::TardisError, result::TardisResult},
    chrono::Utc,
    db::{
        reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
        sea_orm::{FromQueryResult, Value},
    },
    serde_json::{self, json, Map},
    web::web_resp::TardisPage,
    TardisFuns, TardisFunsInst,
};

use bios_basic::{basic_enumeration::BasicQueryOpKind, dto::BasicQueryCondInfo, helper::db_helper, spi::spi_funs::SpiBsInst};

use crate::dto::search_item_dto::{
    AdvBasicQueryCondInfo, SearchItemAddReq, SearchItemModifyReq, SearchItemSearchQScopeKind, SearchItemSearchReq, SearchItemSearchResp, SearchQueryMetricsReq,
    SearchQueryMetricsResp,
};

use super::search_pg_initializer;

const FUNCTION_SUFFIX_FLAG: &str = "__";
const FUNCTION_EXT_SUFFIX_FLAG: &str = "_ext_";

pub async fn add(add_req: &mut SearchItemAddReq, _funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let mut params = Vec::new();
    params.push(Value::from(add_req.kind.to_string()));
    params.push(Value::from(add_req.key.to_string()));
    params.push(Value::from(add_req.title.as_str()));
    params.push(Value::from(format!(
        "{},{}",
        add_req.title.as_str(),
        to_pinyin_vec(add_req.title.as_str(), Pinyin::plain).join(",")
    )));
    params.push(Value::from(add_req.content.as_str()));
    params.push(Value::from(format!(
        "{},{}",
        add_req.content.as_str(),
        to_pinyin_vec(add_req.content.as_str(), Pinyin::plain).join(",")
    )));
    params.push(Value::from(add_req.owner.as_ref().unwrap_or(&"".to_string()).as_str()));
    params.push(Value::from(add_req.own_paths.as_ref().unwrap_or(&"".to_string()).as_str()));
    params.push(Value::from(if let Some(create_time) = add_req.create_time { create_time } else { Utc::now() }));
    params.push(Value::from(if let Some(update_time) = add_req.update_time { update_time } else { Utc::now() }));
    params.push(Value::from(if let Some(ext) = &add_req.ext {
        ext.clone()
    } else {
        TardisFuns::json.str_to_json("{}")?
    }));
    if let Some(visit_keys) = &add_req.visit_keys {
        params.push(Value::from(visit_keys.to_sql()));
    };

    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = search_pg_initializer::init_table_and_conn(bs_inst, &add_req.tag, ctx, true).await?;
    conn.begin().await?;
    conn.execute_one(
        &format!(
            r#"INSERT INTO {table_name} 
    (kind, key, title, title_tsv,content, content_tsv, owner, own_paths, create_time, update_time, ext, visit_keys)
VALUES
    ($1, $2, $3, to_tsvector('public.chinese_zh', $4), $5, to_tsvector('public.chinese_zh', $6), $7, $8, $9, $10, $11, {})"#,
            if add_req.visit_keys.is_some() { "$11" } else { "null" },
        ),
        params,
    )
    .await?;
    conn.commit().await?;
    Ok(())
}

pub async fn modify(tag: &str, key: &str, modify_req: &mut SearchItemModifyReq, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = search_pg_initializer::init_table_and_conn(bs_inst, tag, ctx, true).await?;

    let mut params = Vec::new();
    params.push(Value::from(key));

    let mut sql_sets: Vec<String> = Vec::new();

    if let Some(kind) = &modify_req.kind {
        sql_sets.push(format!("kind = ${}", params.len() + 1));
        params.push(Value::from(kind));
    };
    if let Some(title) = &modify_req.title {
        sql_sets.push(format!("title = ${}", params.len() + 1));
        params.push(Value::from(title));
        sql_sets.push(format!("title_tsv = to_tsvector('public.chinese_zh', ${})", params.len() + 1));
        params.push(Value::from(format!("{},{}", title, to_pinyin_vec(title, Pinyin::plain).join(","))));
    };
    if let Some(content) = &modify_req.content {
        sql_sets.push(format!("content = ${}", params.len() + 1));
        params.push(Value::from(content));
        sql_sets.push(format!("content_tsv = to_tsvector('public.chinese_zh', ${})", params.len() + 1));
        params.push(Value::from(format!("{},{}", content, to_pinyin_vec(content, Pinyin::plain).join(","))));
    };
    if let Some(owner) = &modify_req.owner {
        sql_sets.push(format!("owner = ${}", params.len() + 1));
        params.push(Value::from(owner));
    };
    if let Some(own_paths) = &modify_req.own_paths {
        sql_sets.push(format!("own_paths = ${}", params.len() + 1));
        params.push(Value::from(own_paths));
    };
    if let Some(create_time) = modify_req.create_time {
        sql_sets.push(format!("create_time = ${}", params.len() + 1));
        params.push(Value::from(create_time));
    };
    if let Some(update_time) = modify_req.update_time {
        sql_sets.push(format!("update_time = ${}", params.len() + 1));
        params.push(Value::from(update_time));
    };
    if let Some(ext) = &modify_req.ext {
        sql_sets.push(format!("ext = ${}", params.len() + 1));
        let ext = ext.clone();
        if !modify_req.ext_override.unwrap_or(false) {
            let mut storage_ext = get_ext(tag, key, &table_name, &conn, funs, inst).await?;
            merge(&mut storage_ext, ext);
            params.push(Value::from(storage_ext.clone()));
        } else {
            params.push(Value::from(ext.clone()));
        }
    };
    if let Some(visit_keys) = &modify_req.visit_keys {
        sql_sets.push(format!("visit_keys = ${}", params.len() + 1));
        params.push(Value::from(visit_keys.to_sql()));
    };

    conn.begin().await?;
    conn.execute_one(
        &format!(
            r#"UPDATE {table_name}
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

pub async fn delete(tag: &str, key: &str, _funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = search_pg_initializer::init_table_and_conn(bs_inst, tag, ctx, false).await?;
    conn.begin().await?;
    conn.execute_one(&format!("DELETE FROM {table_name} WHERE key = $1"), vec![Value::from(key)]).await?;
    conn.commit().await?;
    Ok(())
}
pub async fn delete_by_ownership(tag: &str, own_paths: &str, _funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = search_pg_initializer::init_table_and_conn(bs_inst, tag, ctx, false).await?;
    conn.begin().await?;
    conn.execute_one(&format!("DELETE FROM {table_name} WHERE own_paths = $1"), vec![Value::from(own_paths)]).await?;
    conn.commit().await?;
    Ok(())
}

async fn get_ext(tag: &str, key: &str, table_name: &str, conn: &TardisRelDBlConnection, funs: &TardisFunsInst, _inst: &SpiBsInst) -> TardisResult<serde_json::Value> {
    let result = conn
        .query_one(&format!("SELECT ext FROM {table_name} WHERE key = $1"), vec![Value::from(key)])
        .await?
        .ok_or_else(|| funs.err().not_found("item", "get_ext", &format!("search item [{key}] not found in [{tag}]"), "404-spi-search-item-not-exist"))?;
    Ok(result.try_get("", "ext")?)
}

pub async fn search(search_req: &mut SearchItemSearchReq, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<TardisPage<SearchItemSearchResp>> {
    let select_fragments;
    let mut from_fragments = "".to_string();
    let mut where_fragments: Vec<String> = vec!["1=1".to_string()];
    let mut order_fragments: Vec<String> = Vec::new();

    let mut sql_vals: Vec<Value> = vec![];

    if let Some(q) = &search_req.query.q {
        let q = q
            .chars()
            // Fixed like `syntax error in tsquery: "吴 林"`
            .filter(|c| !c.is_whitespace())
            .map(|c| match c {
                '｜' => '|',
                '＆' => '&',
                '！' => '!',
                _ => c,
            })
            .collect::<String>();
        sql_vals.push(Value::from(q.as_str()));
        from_fragments = format!(", plainto_tsquery('public.chinese_zh', ${}) AS query", sql_vals.len());
        match search_req.query.q_scope.as_ref().unwrap_or(&SearchItemSearchQScopeKind::Title) {
            SearchItemSearchQScopeKind::Title => {
                select_fragments = ", COALESCE(ts_rank(title_tsv, query), 0::float4) AS rank_title, 0::float4 AS rank_content".to_string();
                where_fragments.push("(query @@ title_tsv)".to_string());
            }
            SearchItemSearchQScopeKind::Content => {
                select_fragments = ", 0::float4 AS rank_title, COALESCE(ts_rank(content_tsv, query), 0::float4) AS rank_content".to_string();
                where_fragments.push("(query @@ content_tsv)".to_string());
            }
            SearchItemSearchQScopeKind::TitleContent => {
                select_fragments = ", COALESCE(ts_rank(title_tsv, query), 0::float4) AS rank_title, COALESCE(ts_rank(content_tsv, query), 0::float4) AS rank_content".to_string();
                where_fragments.push("(query @@ title_tsv OR query @@ content_tsv)".to_string());
            }
        }
    } else {
        select_fragments = ", 0::float4 AS rank_title, 0::float4 AS rank_content".to_string();
    }

    // Add visit_keys filter
    let req_ctx = search_req.ctx.to_sql();
    if !req_ctx.is_empty() {
        let mut where_visit_keys_fragments = Vec::new();
        for (scope_key, scope_values) in req_ctx {
            if scope_values.is_empty() {
                continue;
            }
            if scope_values.len() == 1 {
                where_visit_keys_fragments.push(format!("visit_keys -> '{scope_key}' ? ${}", sql_vals.len() + 1));
            } else {
                where_visit_keys_fragments.push(format!(
                    "visit_keys -> '{scope_key}' ?| array[{}]",
                    (0..scope_values.len()).map(|idx| format!("${}", sql_vals.len() + idx + 1)).collect::<Vec<String>>().join(", ")
                ));
            }
            for scope_value in scope_values {
                sql_vals.push(Value::from(scope_value));
            }
        }
        where_fragments.push(format!(
            "(visit_keys IS NULL OR ({}))",
            where_visit_keys_fragments.join(if search_req.ctx.cond_by_or.unwrap_or(false) { " OR " } else { " AND " })
        ));
    }

    if let Some(kinds) = &search_req.query.kinds {
        if !kinds.is_empty() {
            where_fragments.push(format!(
                "kind = ANY (ARRAY[{}])",
                (0..kinds.len()).map(|idx| format!("${}", sql_vals.len() + idx + 1)).collect::<Vec<String>>().join(",")
            ));
            for kind in kinds {
                sql_vals.push(Value::from(kind.to_string()));
            }
        }
    }
    if let Some(keys) = &search_req.query.keys {
        if !keys.is_empty() {
            where_fragments.push(format!(
                "key LIKE ANY (ARRAY[{}])",
                (0..keys.len()).map(|idx| format!("${}", sql_vals.len() + idx + 1)).collect::<Vec<String>>().join(",")
            ));
            for key in keys {
                sql_vals.push(Value::from(format!("{key}%")));
            }
        }
    }
    if let Some(owners) = &search_req.query.owners {
        if !owners.is_empty() {
            where_fragments.push(format!(
                "owner LIKE ANY (ARRAY[{}])",
                (0..owners.len()).map(|idx| format!("${}", sql_vals.len() + idx + 1)).collect::<Vec<String>>().join(",")
            ));
            for owner in owners {
                sql_vals.push(Value::from(format!("{owner}%")));
            }
        }
    }
    if let Some(own_paths) = &search_req.query.own_paths {
        if !own_paths.is_empty() {
            where_fragments.push(format!(
                "own_paths LIKE ANY (ARRAY[{}])",
                (0..own_paths.len()).map(|idx| format!("${}", sql_vals.len() + idx + 1)).collect::<Vec<String>>().join(",")
            ));
            for own_path in own_paths {
                sql_vals.push(Value::from(format!("{own_path}%")));
            }
        }
    }
    if let Some(create_time_start) = search_req.query.create_time_start {
        sql_vals.push(Value::from(create_time_start));
        where_fragments.push(format!("create_time >= ${}", sql_vals.len()));
    }
    if let Some(create_time_end) = search_req.query.create_time_end {
        sql_vals.push(Value::from(create_time_end));
        where_fragments.push(format!("create_time <= ${}", sql_vals.len()));
    }
    if let Some(update_time_start) = search_req.query.update_time_start {
        sql_vals.push(Value::from(update_time_start));
        where_fragments.push(format!("update_time >= ${}", sql_vals.len()));
    }
    if let Some(update_time_end) = search_req.query.update_time_end {
        sql_vals.push(Value::from(update_time_end));
        where_fragments.push(format!("update_time <= ${}", sql_vals.len()));
    }
    let err_not_found = |ext_item: &BasicQueryCondInfo| {
        Err(funs.err().not_found(
            "item",
            "search",
            &format!("The ext field=[{}] value=[{}] operation=[{}] is not legal.", &ext_item.field, ext_item.value, &ext_item.op,),
            "404-spi-search-op-not-legal",
        ))
    };
    if let Some(ext) = &search_req.query.ext {
        for ext_item in ext {
            let value = db_helper::json_to_sea_orm_value(&ext_item.value, ext_item.op == BasicQueryOpKind::Like || ext_item.op == BasicQueryOpKind::NotLike);
            let Some(mut value) = value else { return err_not_found(ext_item) };
            if ext_item.op == BasicQueryOpKind::In {
                let value = value.clone();
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
                    return err_not_found(ext_item);
                }
                let Some(value) = value.pop() else {
                    return Err(funs.err().bad_request("item", "search", "Request item using 'IN' operator show hava a value", "400-spi-item-op-in-without-value"));
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
                } else if value.is_chrono_date_time_utc() {
                    where_fragments.push(format!(
                        "(ext ->> '{}')::timestamp with time zone {} ${}",
                        ext_item.field,
                        ext_item.op.to_sql(),
                        sql_vals.len() + 1
                    ));
                } else {
                    where_fragments.push(format!("ext ->> '{}' {} ${}", ext_item.field, ext_item.op.to_sql(), sql_vals.len() + 1));
                }
                sql_vals.push(value);
            }
        }
    }

    if let Some(sort) = &search_req.sort {
        for sort_item in sort {
            if sort_item.field.to_lowercase() == "key"
                || sort_item.field.to_lowercase() == "title"
                || sort_item.field.to_lowercase() == "owner"
                || sort_item.field.to_lowercase() == "own_paths"
                || sort_item.field.to_lowercase() == "create_time"
                || sort_item.field.to_lowercase() == "update_time"
            {
                order_fragments.push(format!("{} {}", sort_item.field, sort_item.order.to_sql()));
            } else {
                order_fragments.push(format!("ext ->> '{}' {}", sort_item.field, sort_item.order.to_sql()));
            }
        }
    }

    // advanced query
    let mut sql_adv_query = vec![];
    if let Some(adv_query) = &search_req.adv_query {
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
                            sql_and_where.push(format!("(ext ->> '{}' is null or ext ->> '{}' = '')", ext_item.field, ext_item.field));
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
                            sql_and_where.push(format!("({} is null or {} = '')", ext_item.field, ext_item.field));
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
    sql_vals.push(Value::from(search_req.page.size));
    sql_vals.push(Value::from((search_req.page.number - 1) * search_req.page.size as u32));
    let page_fragments = format!("LIMIT ${} OFFSET ${}", sql_vals.len() - 1, sql_vals.len());

    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (conn, table_name) = search_pg_initializer::init_table_and_conn(bs_inst, &search_req.tag, ctx, false).await?;
    let result = conn
        .query_all(
            format!(
                r#"SELECT kind, key, title, owner, own_paths, create_time, update_time, ext{}{}{}
FROM {table_name}{}
WHERE 
    {}
    {}
    {}
{}"#,
                if search_req.page.fetch_total { ", count(*) OVER() AS total" } else { "" },
                if search_req.query.in_q_content.unwrap_or(false) { ", content" } else { "" },
                select_fragments,
                from_fragments,
                where_fragments.join(" AND "),
                if sql_adv_query.is_empty() {
                    "".to_string()
                } else {
                    format!(" AND ( 1=1 {})", sql_adv_query.join(" "))
                },
                if order_fragments.is_empty() {
                    "".to_string()
                } else {
                    format!("ORDER BY {}", order_fragments.join(", "))
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
            if search_req.page.fetch_total && total_size == 0 {
                total_size = item.try_get("", "total")?;
            }
            Ok(SearchItemSearchResp {
                kind: item.try_get("", "kind")?,
                key: item.try_get("", "key")?,
                title: item.try_get("", "title")?,
                content: item.try_get("", "content").unwrap_or_default(),
                owner: item.try_get("", "owner")?,
                own_paths: item.try_get("", "own_paths")?,
                create_time: item.try_get("", "create_time")?,
                update_time: item.try_get("", "update_time")?,
                ext: item.try_get("", "ext")?,
                rank_title: item.try_get("", "rank_title")?,
                rank_content: item.try_get("", "rank_content")?,
            })
        })
        .collect::<TardisResult<Vec<SearchItemSearchResp>>>()?;

    Ok(TardisPage {
        page_size: search_req.page.size as u64,
        page_number: search_req.page.number as u64,
        total_size: total_size as u64,
        records: result,
    })
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

pub async fn query_metrics(query_req: &SearchQueryMetricsReq, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<SearchQueryMetricsResp> {
    let mut params = vec![];
    let conf_limit = query_req.conf_limit.unwrap_or(100);
    let select_fragments;
    let mut from_fragments = "".to_string();
    // Package filter
    let mut sql_part_wheres = vec![];
    if let Some(q) = &query_req.query.q {
        let q = q
            .chars()
            // Fixed like `syntax error in tsquery: "吴 林"`
            .filter(|c| !c.is_whitespace())
            .map(|c| match c {
                '｜' => '|',
                '＆' => '&',
                '！' => '!',
                _ => c,
            })
            .collect::<String>();
        params.push(Value::from(q.as_str()));
        from_fragments = format!(", plainto_tsquery('public.chinese_zh', ${}) AS query", params.len());
        match query_req.query.q_scope.as_ref().unwrap_or(&SearchItemSearchQScopeKind::Title) {
            SearchItemSearchQScopeKind::Title => {
                select_fragments = ", COALESCE(ts_rank(title_tsv, query), 0::float4) AS rank_title, 0::float4 AS rank_content".to_string();
                sql_part_wheres.push("(query @@ title_tsv)".to_string());
            }
            SearchItemSearchQScopeKind::Content => {
                select_fragments = ", 0::float4 AS rank_title, COALESCE(ts_rank(content_tsv, query), 0::float4) AS rank_content".to_string();
                sql_part_wheres.push("(query @@ content_tsv)".to_string());
            }
            SearchItemSearchQScopeKind::TitleContent => {
                select_fragments = ", COALESCE(ts_rank(title_tsv, query), 0::float4) AS rank_title, COALESCE(ts_rank(content_tsv, query), 0::float4) AS rank_content".to_string();
                sql_part_wheres.push("(query @@ title_tsv OR query @@ content_tsv)".to_string());
            }
        }
    } else {
        select_fragments = ", 0::float4 AS rank_title, 0::float4 AS rank_content".to_string();
    }
    if let Some(wheres) = &query_req._where {
        let mut sql_part_or_wheres = vec![];
        for or_wheres in wheres {
            let mut sql_part_and_wheres = vec![];
            for and_where in or_wheres {
                let where_column = if and_where.in_ext.unwrap_or(true) {
                    format!("fact.ext ->> '{}'", &and_where.code)
                } else {
                    format!("fact.{}", &and_where.code)
                };
                if let Some((sql_part, value)) = and_where.data_type.to_pg_where(
                    and_where.multi_values.unwrap_or(false),
                    &where_column,
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
                            and_where.data_type.to_string().to_lowercase(),
                            &and_where.op.to_sql(),
                            &and_where.time_window.is_some(),
                            and_where.multi_values.unwrap_or_default()
                        ),
                        "404-spi-stats-metric-op-not-legal",
                    ));
                }
            }
            sql_part_or_wheres.push(sql_part_and_wheres.join(" AND "));
        }
        if !sql_part_or_wheres.is_empty() {
            sql_part_wheres.push(format!("( {} )", sql_part_or_wheres.join(" OR ")));
        }
    }

    // Add visit_keys filter
    let req_ctx = query_req.ctx.to_sql();
    if !req_ctx.is_empty() {
        let mut where_visit_keys_fragments = Vec::new();
        for (scope_key, scope_values) in req_ctx {
            if scope_values.is_empty() {
                continue;
            }
            if scope_values.len() == 1 {
                where_visit_keys_fragments.push(format!("fact.visit_keys -> '{scope_key}' ? ${}", params.len() + 1));
            } else {
                where_visit_keys_fragments.push(format!(
                    "fact.visit_keys -> '{scope_key}' ?| array[{}]",
                    (0..scope_values.len()).map(|idx| format!("${}", params.len() + idx + 1)).collect::<Vec<String>>().join(", ")
                ));
            }
            for scope_value in scope_values {
                params.push(Value::from(scope_value));
            }
        }
        if !where_visit_keys_fragments.is_empty() {
            sql_part_wheres.push(format!(
                " (fact.visit_keys IS NULL OR ({}))",
                where_visit_keys_fragments.join(if query_req.ctx.cond_by_or.unwrap_or(false) { " OR " } else { " AND " })
            ));
        }
    }

    if let Some(kinds) = &query_req.query.kinds {
        if !kinds.is_empty() {
            sql_part_wheres.push(format!(
                "fact.kind = ANY (ARRAY[{}])",
                (0..kinds.len()).map(|idx| format!("${}", params.len() + idx + 1)).collect::<Vec<String>>().join(",")
            ));
            for kind in kinds {
                params.push(Value::from(kind.to_string()));
            }
        }
    }
    if let Some(keys) = &query_req.query.keys {
        if !keys.is_empty() {
            sql_part_wheres.push(format!(
                "fact.key LIKE ANY (ARRAY[{}])",
                (0..keys.len()).map(|idx| format!("${}", params.len() + idx + 1)).collect::<Vec<String>>().join(",")
            ));
            for key in keys {
                params.push(Value::from(format!("{key}%")));
            }
        }
    }
    if let Some(owners) = &query_req.query.owners {
        if !owners.is_empty() {
            sql_part_wheres.push(format!(
                "fact.owner LIKE ANY (ARRAY[{}])",
                (0..owners.len()).map(|idx| format!("${}", params.len() + idx + 1)).collect::<Vec<String>>().join(",")
            ));
            for owner in owners {
                params.push(Value::from(format!("{owner}%")));
            }
        }
    }
    if let Some(own_paths) = &query_req.query.own_paths {
        if !own_paths.is_empty() {
            sql_part_wheres.push(format!(
                "fact.own_paths LIKE ANY (ARRAY[{}])",
                (0..own_paths.len()).map(|idx| format!("${}", params.len() + idx + 1)).collect::<Vec<String>>().join(",")
            ));
            for own_path in own_paths {
                params.push(Value::from(format!("{own_path}%")));
            }
        }
    }
    if let Some(create_time_start) = query_req.query.create_time_start {
        params.push(Value::from(create_time_start));
        sql_part_wheres.push(format!("fact.create_time >= ${}", params.len()));
    }
    if let Some(create_time_end) = query_req.query.create_time_end {
        params.push(Value::from(create_time_end));
        sql_part_wheres.push(format!("fact.create_time <= ${}", params.len()));
    }
    if let Some(update_time_start) = query_req.query.update_time_start {
        params.push(Value::from(update_time_start));
        sql_part_wheres.push(format!("fact.update_time >= ${}", params.len()));
    }
    if let Some(update_time_end) = query_req.query.update_time_end {
        params.push(Value::from(update_time_end));
        sql_part_wheres.push(format!("fact.update_time <= ${}", params.len()));
    }
    let err_not_found = |ext_item: &BasicQueryCondInfo| {
        Err(funs.err().not_found(
            "item",
            "search",
            &format!("The ext field=[{}] value=[{}] operation=[{}] is not legal.", &ext_item.field, ext_item.value, &ext_item.op,),
            "404-spi-search-op-not-legal",
        ))
    };
    if let Some(ext) = &query_req.query.ext {
        for ext_item in ext {
            let value = db_helper::json_to_sea_orm_value(&ext_item.value, ext_item.op == BasicQueryOpKind::Like || ext_item.op == BasicQueryOpKind::NotLike);
            let Some(mut value) = value else { return err_not_found(ext_item) };
            if ext_item.op == BasicQueryOpKind::In {
                let value = value.clone();
                if value.len() == 1 {
                    sql_part_wheres.push(format!("fact.ext -> '{}' ? ${}", ext_item.field, params.len() + 1));
                } else {
                    sql_part_wheres.push(format!(
                        "fact.ext -> '{}' ?| array[{}]",
                        ext_item.field,
                        (0..value.len()).map(|idx| format!("${}", params.len() + idx + 1)).collect::<Vec<String>>().join(", ")
                    ));
                }
                for val in value {
                    params.push(val);
                }
            } else if ext_item.op == BasicQueryOpKind::NotIn {
                let value = value.clone();
                if value.len() == 1 {
                    sql_part_wheres.push(format!("not (fact.ext -> '{}' ? ${})", ext_item.field, params.len() + 1));
                } else {
                    sql_part_wheres.push(format!(
                        "not (fact.ext -> '{}' ?| array[{}])",
                        ext_item.field,
                        (0..value.len()).map(|idx| format!("${}", params.len() + idx + 1)).collect::<Vec<String>>().join(", ")
                    ));
                }
                for val in value {
                    params.push(val);
                }
            } else if ext_item.op == BasicQueryOpKind::IsNull {
                sql_part_wheres.push(format!("fact.ext ->> '{}' is null", ext_item.field));
            } else if ext_item.op == BasicQueryOpKind::IsNotNull {
                sql_part_wheres.push(format!("fact.ext ->> '{}' is not null", ext_item.field));
            } else if ext_item.op == BasicQueryOpKind::IsNullOrEmpty {
                sql_part_wheres.push(format!("(fact.ext ->> '{}' is null or fact.ext ->> '{}' = '')", ext_item.field, ext_item.field));
            } else {
                if value.len() > 1 {
                    return err_not_found(ext_item);
                }
                let Some(value) = value.pop() else {
                    return Err(funs.err().bad_request("item", "search", "Request item using 'IN' operator show hava a value", "400-spi-item-op-in-without-value"));
                };
                if let Value::Bool(_) = value {
                    sql_part_wheres.push(format!("(fact.ext ->> '{}')::boolean {} ${}", ext_item.field, ext_item.op.to_sql(), params.len() + 1));
                } else if let Value::TinyInt(_) = value {
                    sql_part_wheres.push(format!("(fact.ext ->> '{}')::smallint {} ${}", ext_item.field, ext_item.op.to_sql(), params.len() + 1));
                } else if let Value::SmallInt(_) = value {
                    sql_part_wheres.push(format!("(fact.ext ->> '{}')::smallint {} ${}", ext_item.field, ext_item.op.to_sql(), params.len() + 1));
                } else if let Value::Int(_) = value {
                    sql_part_wheres.push(format!("(fact.ext ->> '{}')::integer {} ${}", ext_item.field, ext_item.op.to_sql(), params.len() + 1));
                } else if let Value::BigInt(_) = value {
                    sql_part_wheres.push(format!("(fact.ext ->> '{}')::bigint {} ${}", ext_item.field, ext_item.op.to_sql(), params.len() + 1));
                } else if let Value::TinyUnsigned(_) = value {
                    sql_part_wheres.push(format!("(fact.ext ->> '{}')::smallint {} ${}", ext_item.field, ext_item.op.to_sql(), params.len() + 1));
                } else if let Value::SmallUnsigned(_) = value {
                    sql_part_wheres.push(format!("(fact.ext ->> '{}')::integer {} ${}", ext_item.field, ext_item.op.to_sql(), params.len() + 1));
                } else if let Value::Unsigned(_) = value {
                    sql_part_wheres.push(format!("(fact.ext ->> '{}')::bigint {} ${}", ext_item.field, ext_item.op.to_sql(), params.len() + 1));
                } else if let Value::BigUnsigned(_) = value {
                    // TODO
                    sql_part_wheres.push(format!("(fact.ext ->> '{}')::bigint {} ${}", ext_item.field, ext_item.op.to_sql(), params.len() + 1));
                } else if let Value::Float(_) = value {
                    sql_part_wheres.push(format!("(fact.ext ->> '{}')::real {} ${}", ext_item.field, ext_item.op.to_sql(), params.len() + 1));
                } else if let Value::Double(_) = value {
                    sql_part_wheres.push(format!(
                        "(fact.ext ->> '{}')::double precision {} ${}",
                        ext_item.field,
                        ext_item.op.to_sql(),
                        params.len() + 1
                    ));
                } else if value.is_chrono_date_time_utc() {
                    sql_part_wheres.push(format!(
                        "(fact.ext ->> '{}')::timestamp with time zone {} ${}",
                        ext_item.field,
                        ext_item.op.to_sql(),
                        params.len() + 1
                    ));
                } else {
                    sql_part_wheres.push(format!("fact.ext ->> '{}' {} ${}", ext_item.field, ext_item.op.to_sql(), params.len() + 1));
                }
                params.push(value);
            }
        }
    }
    let sql_part_wheres = if sql_part_wheres.is_empty() {
        "".to_string()
    } else {
        format!(" AND {}", sql_part_wheres.join(" AND "))
    };

    // advanced query
    let mut sql_adv_query = vec![];
    if let Some(adv_query) = &query_req.adv_query {
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
                                sql_and_where.push(format!("fact.ext -> '{}' ? ${}", ext_item.field, params.len() + 1));
                            } else {
                                sql_and_where.push(format!(
                                    "fact.ext -> '{}' ?| array[{}]",
                                    ext_item.field,
                                    (0..value.len()).map(|idx| format!("${}", params.len() + idx + 1)).collect::<Vec<String>>().join(", ")
                                ));
                            }
                            for val in value {
                                params.push(val);
                            }
                        } else if ext_item.op == BasicQueryOpKind::NotIn {
                            let value = value.clone();
                            if value.len() == 1 {
                                sql_and_where.push(format!("not (fact.ext -> '{}' ? ${})", ext_item.field, params.len() + 1));
                            } else {
                                sql_and_where.push(format!(
                                    "not (fact.ext -> '{}' ?| array[{}])",
                                    ext_item.field,
                                    (0..value.len()).map(|idx| format!("${}", params.len() + idx + 1)).collect::<Vec<String>>().join(", ")
                                ));
                            }
                            for val in value {
                                params.push(val);
                            }
                        } else if ext_item.op == BasicQueryOpKind::IsNull {
                            sql_and_where.push(format!("fact.ext ->> '{}' is null", ext_item.field));
                        } else if ext_item.op == BasicQueryOpKind::IsNotNull {
                            sql_and_where.push(format!("fact.ext ->> '{}' is not null", ext_item.field));
                        } else if ext_item.op == BasicQueryOpKind::IsNullOrEmpty {
                            sql_and_where.push(format!("(fact.ext ->> '{}' is null or ext ->> '{}' = '')", ext_item.field, ext_item.field));
                        } else {
                            if value.len() > 1 {
                                return err_not_found(ext_item);
                            }
                            let Some(value) = value.pop() else {
                                return Err(funs.err().bad_request("item", "search", "Request item using 'IN' operator show hava a value", "400-spi-item-op-in-without-value"));
                            };
                            if let Value::Bool(_) = value {
                                sql_and_where.push(format!("(fact.ext ->> '{}')::boolean {} ${}", ext_item.field, ext_item.op.to_sql(), params.len() + 1));
                            } else if let Value::TinyInt(_) = value {
                                sql_and_where.push(format!("(fact.ext ->> '{}')::smallint {} ${}", ext_item.field, ext_item.op.to_sql(), params.len() + 1));
                            } else if let Value::SmallInt(_) = value {
                                sql_and_where.push(format!("(fact.ext ->> '{}')::smallint {} ${}", ext_item.field, ext_item.op.to_sql(), params.len() + 1));
                            } else if let Value::Int(_) = value {
                                sql_and_where.push(format!("(fact.ext ->> '{}')::integer {} ${}", ext_item.field, ext_item.op.to_sql(), params.len() + 1));
                            } else if let Value::BigInt(_) = value {
                                sql_and_where.push(format!("(fact.ext ->> '{}')::bigint {} ${}", ext_item.field, ext_item.op.to_sql(), params.len() + 1));
                            } else if let Value::TinyUnsigned(_) = value {
                                sql_and_where.push(format!("(fact.ext ->> '{}')::smallint {} ${}", ext_item.field, ext_item.op.to_sql(), params.len() + 1));
                            } else if let Value::SmallUnsigned(_) = value {
                                sql_and_where.push(format!("(fact.ext ->> '{}')::integer {} ${}", ext_item.field, ext_item.op.to_sql(), params.len() + 1));
                            } else if let Value::Unsigned(_) = value {
                                sql_and_where.push(format!("(fact.ext ->> '{}')::bigint {} ${}", ext_item.field, ext_item.op.to_sql(), params.len() + 1));
                            } else if let Value::BigUnsigned(_) = value {
                                // TODO
                                sql_and_where.push(format!("(fact.ext ->> '{}')::bigint {} ${}", ext_item.field, ext_item.op.to_sql(), params.len() + 1));
                            } else if let Value::Float(_) = value {
                                sql_and_where.push(format!("(fact.ext ->> '{}')::real {} ${}", ext_item.field, ext_item.op.to_sql(), params.len() + 1));
                            } else if let Value::Double(_) = value {
                                sql_and_where.push(format!(
                                    "(fact.ext ->> '{}')::double precision {} ${}",
                                    ext_item.field,
                                    ext_item.op.to_sql(),
                                    params.len() + 1
                                ));
                            } else if value.is_chrono_date_time_utc() {
                                sql_and_where.push(format!(
                                    "(fact.ext ->> '{}')::timestamp with time zone {} ${}",
                                    ext_item.field,
                                    ext_item.op.to_sql(),
                                    params.len() + 1
                                ));
                            } else {
                                sql_and_where.push(format!("fact.ext ->> '{}' {} ${}", ext_item.field, ext_item.op.to_sql(), params.len() + 1));
                            }
                            params.push(value);
                        }
                    } else {
                        if ext_item.op == BasicQueryOpKind::In {
                            if !value.is_empty() {
                                sql_and_where.push(format!(
                                    "fact.{} LIKE ANY (ARRAY[{}])",
                                    ext_item.field,
                                    (0..value.len()).map(|idx| format!("${}", params.len() + idx + 1)).collect::<Vec<String>>().join(",")
                                ));
                                for val in value {
                                    params.push(Value::from(format!("{val}%")));
                                }
                            }
                        } else if ext_item.op == BasicQueryOpKind::NotIn {
                            if !value.is_empty() {
                                sql_and_where.push(format!(
                                    "fact.{} NOT LIKE ANY (ARRAY[{}])",
                                    ext_item.field,
                                    (0..value.len()).map(|idx| format!("${}", params.len() + idx + 1)).collect::<Vec<String>>().join(",")
                                ));
                                for val in value {
                                    params.push(Value::from(format!("{val}%")));
                                }
                            }
                        } else if ext_item.op == BasicQueryOpKind::IsNull {
                            sql_and_where.push(format!("fact.{} is null", ext_item.field));
                        } else if ext_item.op == BasicQueryOpKind::IsNotNull {
                            sql_and_where.push(format!("fact.{} is not null", ext_item.field));
                        } else if ext_item.op == BasicQueryOpKind::IsNullOrEmpty {
                            sql_and_where.push(format!("(fact.{} is null or {} = '')", ext_item.field, ext_item.field));
                        } else {
                            if value.len() > 1 {
                                return err_not_found(ext_item);
                            }
                            let Some(value) = value.pop() else {
                                return Err(funs.err().bad_request("item", "search", "Request item using 'IN' operator show hava a value", "400-spi-item-op-in-without-value"));
                            };
                            if let Value::Bool(_) = value {
                                sql_and_where.push(format!("(fact.{}::boolean) {} ${}", ext_item.field, ext_item.op.to_sql(), params.len() + 1));
                            } else if let Value::TinyInt(_) = value {
                                sql_and_where.push(format!("(fact.{}::smallint) {} ${}", ext_item.field, ext_item.op.to_sql(), params.len() + 1));
                            } else if let Value::SmallInt(_) = value {
                                sql_and_where.push(format!("(fact.{}::smallint) {} ${}", ext_item.field, ext_item.op.to_sql(), params.len() + 1));
                            } else if let Value::Int(_) = value {
                                sql_and_where.push(format!("(fact.{}::integer) {} ${}", ext_item.field, ext_item.op.to_sql(), params.len() + 1));
                            } else if let Value::BigInt(_) = value {
                                sql_and_where.push(format!("(fact.{}::bigint) {} ${}", ext_item.field, ext_item.op.to_sql(), params.len() + 1));
                            } else if let Value::TinyUnsigned(_) = value {
                                sql_and_where.push(format!("(fact.{}::smallint) {} ${}", ext_item.field, ext_item.op.to_sql(), params.len() + 1));
                            } else if let Value::SmallUnsigned(_) = value {
                                sql_and_where.push(format!("(fact.{}::integer) {} ${}", ext_item.field, ext_item.op.to_sql(), params.len() + 1));
                            } else if let Value::Unsigned(_) = value {
                                sql_and_where.push(format!("(fact.{}::bigint) {} ${}", ext_item.field, ext_item.op.to_sql(), params.len() + 1));
                            } else if let Value::BigUnsigned(_) = value {
                                // TODO
                                sql_and_where.push(format!("(fact.{}::bigint) {} ${}", ext_item.field, ext_item.op.to_sql(), params.len() + 1));
                            } else if let Value::Float(_) = value {
                                sql_and_where.push(format!("(fact.{}::real) {} ${}", ext_item.field, ext_item.op.to_sql(), params.len() + 1));
                            } else if let Value::Double(_) = value {
                                sql_and_where.push(format!("(fact.{}::double precision) {} ${}", ext_item.field, ext_item.op.to_sql(), params.len() + 1));
                            } else {
                                sql_and_where.push(format!("fact.{} {} ${}", ext_item.field, ext_item.op.to_sql(), params.len() + 1));
                            }
                            params.push(value);
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

    let sql_adv_query = if sql_adv_query.is_empty() {
        "".to_string()
    } else {
        format!(" AND ( 1=1 {})", sql_adv_query.join(" "))
    };

    // Package inner select
    // Add measures
    let mut sql_part_inner_selects = vec![];
    for select in &query_req.select {
        if select.in_ext.unwrap_or(true) {
            sql_part_inner_selects.push(format!("fact.ext ->> '{}' AS {}", &select.code, &select.code));
        } else {
            sql_part_inner_selects.push(format!("fact.{} AS {}", &select.code, &select.code));
        }
    }
    for group in &query_req.group {
        if group.in_ext.unwrap_or(true) {
            if group.multi_values.unwrap_or(false) {
                // sql_part_inner_selects.push(format!("jsonb_array_elements(fact.ext -> '{}') AS {}", &group.code, &group.code));
                sql_part_inner_selects.push(format!("jsonb_array_elements(case when fact.ext-> '{}' is null then '[\"\"]' else case when jsonb_array_length(fact.ext -> '{}') = 0 then '[\"\"]' else fact.ext -> '{}' end end) as {}",
                                                    &group.code, &group.code, &group.code, &group.code));
            } else {
                sql_part_inner_selects.push(format!("fact.ext ->> '{}' AS {}", &group.code, &group.code));
            }
        } else {
            sql_part_inner_selects.push(format!("fact.{} AS {}", &group.code, &group.code));
        }
    }
    let sql_part_inner_selects = sql_part_inner_selects.join(",");

    // Package group
    // (column name with fun, alias name, show name)
    let mut sql_part_group_infos = vec![];
    for group in &query_req.group {
        if let Some(column_name_with_fun) = group.data_type.to_pg_group(&format!("_.{}", &group.code), &group.time_window) {
            let alias_name = format!(
                "{}{}{FUNCTION_SUFFIX_FLAG}{}",
                group.code,
                if group.in_ext.unwrap_or(true) { FUNCTION_EXT_SUFFIX_FLAG } else { "" },
                group.time_window.as_ref().map(|i| i.to_string().to_lowercase()).unwrap_or("".to_string())
            );
            sql_part_group_infos.push((column_name_with_fun, alias_name.clone(), alias_name));
        } else {
            return Err(funs.err().not_found(
                "metric",
                "query",
                &format!(
                    "The group column=[{}] type=[{}] time_window=[{}] is not legal.",
                    &group.code,
                    group.data_type.to_string().to_lowercase(),
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
        sql_part_outer_select_infos.push((format!("COALESCE({},'\"empty\"')", column_name_with_fun), alias_name, show_name, true));
    }
    for select in &query_req.select {
        let select_column = if select.in_ext.unwrap_or(true) {
            format!("_.ext ->> '{}'", &select.code)
        } else {
            format!("_.{}", &select.code)
        };
        let column_name_with_fun = select.data_type.to_pg_select(&select_column, &select.fun);
        let alias_name = format!(
            "{}{}{FUNCTION_SUFFIX_FLAG}{}",
            select.code,
            if select.in_ext.unwrap_or(true) { FUNCTION_EXT_SUFFIX_FLAG } else { "" },
            select.fun.to_string().to_lowercase()
        );
        sql_part_outer_select_infos.push((column_name_with_fun, alias_name.clone(), alias_name, false));
    }
    let sql_part_outer_selects =
        sql_part_outer_select_infos.iter().map(|(column_name_with_fun, alias_name, _, _)| format!("{column_name_with_fun} AS {alias_name}")).collect::<Vec<String>>().join(",");

    // Package having
    let sql_part_havings = if let Some(havings) = &query_req.having {
        let mut sql_part_havings = vec![];
        for having in havings {
            let having_column = if having.in_ext.unwrap_or(true) {
                format!("_.ext ->> '{}'", &having.code)
            } else {
                format!("_.{}", &having.code)
            };
            if let Some((sql_part, value)) = having.data_type.to_pg_having(false, &having_column, &having.op, params.len() + 1, &having.value, Some(&having.fun))? {
                value.iter().for_each(|v| params.push(v.clone()));
                sql_part_havings.push(sql_part);
            } else {
                return Err(funs.err().not_found(
                    "metric",
                    "query",
                    &format!(
                        "The query column=[{}] type=[{}] operation=[{}] fun=[{}] is not legal.",
                        &having.code,
                        having.data_type.to_string().to_lowercase(),
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
                let order_column = if order.in_ext.unwrap_or(true) {
                    format!("fact.ext ->> '{}' {}", order.code, if order.asc { "ASC" } else { "DESC" })
                } else {
                    format!("fact.{} {}", order.code, if order.asc { "ASC" } else { "DESC" })
                };
                return order_column;
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
                        "{}{}{FUNCTION_SUFFIX_FLAG}{} {}",
                        order.code,
                        if order.in_ext.unwrap_or(true) { FUNCTION_EXT_SUFFIX_FLAG } else { "" },
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
                        "{}{}{FUNCTION_SUFFIX_FLAG}{} {}",
                        order.code,
                        if order.in_ext.unwrap_or(true) { FUNCTION_EXT_SUFFIX_FLAG } else { "" },
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

    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (conn, table_name) = search_pg_initializer::init_table_and_conn(bs_inst, &query_req.tag, ctx, false).await?;
    let ignore_group_agg = !(!sql_part_groups.is_empty() && query_req.group_agg.unwrap_or(false));
    let final_sql = format!(
        r#"SELECT {sql_part_outer_selects}{}
    FROM (
        SELECT
             {sql_part_inner_selects}{}
             FROM(
                SELECT {}fact.kind, fact.key, fact.title, fact.owner, fact.own_paths, fact.create_time, fact.update_time, fact.ext, 1 as _count{}{}
                FROM {table_name} fact{}
                WHERE
                1=1
                {sql_part_wheres}
                {sql_adv_query}
                ORDER BY {}fact.create_time DESC
             ) fact
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
            ",string_agg(_._key || ' - ' || _._own_paths || ' - ' || to_char(_._create_time, 'YYYY-MM-DD HH24:MI:SS'), ',') as s_agg".to_string()
        },
        if ignore_group_agg {
            "".to_string()
        } else {
            ",fact.key as _key, fact.own_paths as _own_paths, fact.create_time as _create_time".to_string()
        },
        if query_req.ignore_distinct.unwrap_or(false) {
            ""
        } else {
            "DISTINCT ON (fact.key) fact.key AS _key,"
        },
        if query_req.query.in_q_content.unwrap_or(false) { ", content" } else { "" },
        select_fragments,
        from_fragments,
        if query_req.ignore_distinct.unwrap_or(false) { "" } else { "_key," },
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
    Ok(SearchQueryMetricsResp {
        tag: query_req.tag.to_string(),
        show_names,
        group: package_groups(select_dimension_keys, &select_measure_keys, ignore_group_agg, result)
            .map_err(|msg| TardisError::internal_error(&format!("Fail to package groups: {msg}"), "500-spi-stats-internal-error"))?,
    })
}

fn package_groups(
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
        let sub = package_groups(curr_select_dimension_keys[1..].to_vec(), select_measure_keys, ignore_group_agg, group.to_vec())?;
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
            println!("{}", agg);
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
