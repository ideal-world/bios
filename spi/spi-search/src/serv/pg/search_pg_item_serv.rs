use bios_basic::{basic_enumeration::BasicQueryOpKind, dto::BasicQueryCondInfo, helper::db_helper, spi::spi_funs::SpiBsInst};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    chrono::Utc,
    db::{
        reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
        sea_orm::Value,
    },
    serde_json,
    web::web_resp::TardisPage,
    TardisFuns, TardisFunsInst,
};

use crate::dto::search_item_dto::{SearchItemAddReq, SearchItemModifyReq, SearchItemSearchQScopeKind, SearchItemSearchReq, SearchItemSearchResp};

use super::search_pg_initializer;

pub async fn add(add_req: &mut SearchItemAddReq, _funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let mut params = Vec::new();
    params.push(Value::from(add_req.kind.to_string()));
    params.push(Value::from(add_req.key.to_string()));
    params.push(Value::from(add_req.title.as_str()));
    params.push(Value::from(add_req.title.as_str()));
    params.push(Value::from(add_req.content.as_str()));
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
    ($1, $2, $3, to_tsvector('public.chinese_zh', $4), $5, to_tsvector('public.chinese_zh', $5), $6, $7, $8, $9, $10, {})"#,
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
        sql_sets.push(format!("title_tsv = to_tsvector('public.chinese_zh', ${})", params.len() + 1));
        params.push(Value::from(title));
    };
    if let Some(content) = &modify_req.content {
        sql_sets.push(format!("content = ${}", params.len() + 1));
        sql_sets.push(format!("content_tsv = to_tsvector('public.chinese_zh', ${})", params.len() + 1));
        params.push(Value::from(content));
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
            let value = db_helper::json_to_sea_orm_value(&ext_item.value, ext_item.op == BasicQueryOpKind::Like);
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
            }if ext_item.op == BasicQueryOpKind::NotIn {
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
            if let Some(create_time_start) = group_query.create_time_start {
                sql_vals.push(Value::from(create_time_start));
                sql_and_where.push(format!("create_time >= ${}", sql_vals.len()));
            }
            if let Some(create_time_end) = group_query.create_time_end {
                sql_vals.push(Value::from(create_time_end));
                sql_and_where.push(format!("create_time <= ${}", sql_vals.len()));
            }
            if let Some(update_time_start) = group_query.update_time_start {
                sql_vals.push(Value::from(update_time_start));
                sql_and_where.push(format!("update_time >= ${}", sql_vals.len()));
            }
            if let Some(update_time_end) = group_query.update_time_end {
                sql_vals.push(Value::from(update_time_end));
                sql_and_where.push(format!("update_time <= ${}", sql_vals.len()));
            }
            let err_not_found = |ext_item: &BasicQueryCondInfo| {
                Err(funs.err().not_found(
                    "item",
                    "search",
                    &format!("The ext field=[{}] value=[{}] operation=[{}] is not legal.", &ext_item.field, ext_item.value, &ext_item.op,),
                    "404-spi-search-op-not-legal",
                ))
            };
            if let Some(ext) = &group_query.ext {
                for ext_item in ext {
                    let value = db_helper::json_to_sea_orm_value(&ext_item.value, ext_item.op == BasicQueryOpKind::Like);
                    let Some(mut value) = value else { return err_not_found(ext_item) };
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
                        } else {
                            sql_and_where.push(format!("ext ->> '{}' {} ${}", ext_item.field, ext_item.op.to_sql(), sql_vals.len() + 1));
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
    let sql_adv_query = if sql_adv_query.is_empty() { "".to_string() } else { sql_adv_query.join(" ") };

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
    {sql_adv_query}
    {}
{}"#,
                if search_req.page.fetch_total { ", count(*) OVER() AS total" } else { "" },
                if search_req.query.in_q_content.unwrap_or(false) { ", content" } else { "" },
                select_fragments,
                from_fragments,
                where_fragments.join(" AND "),
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
