use bios_basic::spi::{spi_enumeration::SpiQueryOpKind, spi_funs::SpiBsInstExtractor};
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

pub async fn add(add_req: &mut SearchItemAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let mut params = Vec::new();
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

    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = search_pg_initializer::init_table_and_conn(bs_inst, &add_req.tag, ctx, true).await?;
    conn.begin().await?;
    conn.execute_one(
        &format!(
            r#"INSERT INTO {table_name} 
    (key, title, title_tsv, content_tsv, owner, own_paths, create_time, update_time, ext, visit_keys)
VALUES
    ($1, $2, to_tsvector('public.chinese_zh', $3), to_tsvector('public.chinese_zh', $4), $5, $6, $7, $8, $9, {})"#,
            if add_req.visit_keys.is_some() { "$10" } else { "null" },
        ),
        params,
    )
    .await?;
    conn.commit().await?;
    Ok(())
}

pub async fn modify(tag: &str, key: &str, modify_req: &mut SearchItemModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = search_pg_initializer::init_table_and_conn(bs_inst, tag, ctx, true).await?;

    let mut params = Vec::new();
    params.push(Value::from(key));

    let mut sql_sets: Vec<String> = Vec::new();

    if let Some(title) = &modify_req.title {
        sql_sets.push(format!("title = ${}", params.len() + 1));
        sql_sets.push(format!("title_tsv = to_tsvector('public.chinese_zh', ${})", params.len() + 1));
        params.push(Value::from(title));
    };
    if let Some(content) = &modify_req.content {
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
        let mut ext = ext.clone();
        if !modify_req.ext_override.unwrap_or(false) {
            let storage_ext = get_ext(tag, key, &table_name, &conn, funs).await?;
            merge(&mut ext, storage_ext);
        }
        sql_sets.push(format!("ext = ${}", params.len() + 1));
        params.push(Value::from(ext.clone()));
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

pub async fn delete(tag: &str, key: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = search_pg_initializer::init_table_and_conn(bs_inst, tag, ctx, false).await?;
    conn.begin().await?;
    conn.execute_one(&format!("DELETE FROM {table_name} WHERE key = $1"), vec![Value::from(key)]).await?;
    conn.commit().await?;
    Ok(())
}

async fn get_ext(tag: &str, key: &str, table_name: &str, conn: &TardisRelDBlConnection, funs: &TardisFunsInst) -> TardisResult<serde_json::Value> {
    let result = conn
        .query_one(&format!("SELECT ext FROM {table_name} WHERE key = $1"), vec![Value::from(key)])
        .await?
        .ok_or_else(|| funs.err().not_found("item", "get_ext", &format!("search item [{key}] not found in [{tag}]"), "404-spi-search-item-not-exist"))?;
    Ok(result.try_get("", "ext").unwrap())
}

pub async fn search(search_req: &mut SearchItemSearchReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<TardisPage<SearchItemSearchResp>> {
    let select_fragments;
    let mut from_fragments = "".to_string();
    let mut where_fragments: Vec<String> = Vec::new();
    let mut where_visit_keys_fragments: Vec<String> = Vec::new();
    let mut order_fragments: Vec<String> = Vec::new();

    let mut sql_vals: Vec<Value> = vec![];

    if let Some(q) = &search_req.query.q {
        sql_vals.push(Value::from(q.as_str()));
        from_fragments = format!(", to_tsquery('public.chinese_zh', ${}) AS query", sql_vals.len());
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

    for visit_keys in search_req.ctx.to_sql() {
        sql_vals.push(Value::from(visit_keys));
        where_visit_keys_fragments.push(format!("${}::varchar", sql_vals.len()));
    }
    if where_visit_keys_fragments.is_empty() {
        where_fragments.push("visit_keys IS NULL".to_string());
    } else {
        where_fragments.push(format!("(visit_keys IS NULL OR visit_keys @> ARRAY[{}])", where_visit_keys_fragments.join(", ")));
    }

    if let Some(keys) = &search_req.query.keys {
        if !keys.is_empty() {
            where_fragments.push(format!(
                "key LIKE ANY (ARRAY[{}])",
                keys.iter().enumerate().map(|(idx, _)| format!("${}", sql_vals.len() + idx + 1)).collect::<Vec<String>>().join(",")
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
                owners.iter().enumerate().map(|(idx, _)| format!("${}", sql_vals.len() + idx + 1)).collect::<Vec<String>>().join(",")
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
                own_paths.iter().enumerate().map(|(idx, _)| format!("${}", sql_vals.len() + idx + 1)).collect::<Vec<String>>().join(",")
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
    if let Some(ext) = &search_req.query.ext {
        for ext_item in ext {
            if ext_item.op == SpiQueryOpKind::Like {
                sql_vals.push(Value::from(format!("%{}%", ext_item.value)));
            } else {
                sql_vals.push(Value::from(ext_item.value.to_string()));
            }
            where_fragments.push(format!("ext ->> '{}' {} ${}", ext_item.field, ext_item.op.to_sql(), sql_vals.len()));
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

    sql_vals.push(Value::from(search_req.page.size));
    sql_vals.push(Value::from((search_req.page.number - 1) * search_req.page.size as u32));
    let page_fragments = format!("LIMIT ${} OFFSET ${}", sql_vals.len() - 1, sql_vals.len());

    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (conn, table_name) = search_pg_initializer::init_table_and_conn(bs_inst, &search_req.tag, ctx, false).await?;
    let result = conn
        .query_all(
            format!(
                r#"SELECT key, title, owner, own_paths, create_time, update_time, ext, count(*) OVER() AS total{}
FROM {table_name}{}
WHERE 
    {}
    {}
{}"#,
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
            if total_size == 0 {
                total_size = item.try_get("", "total").unwrap();
            }
            SearchItemSearchResp {
                key: item.try_get("", "key").unwrap(),
                title: item.try_get("", "title").unwrap(),
                owner: item.try_get("", "owner").unwrap(),
                own_paths: item.try_get("", "own_paths").unwrap(),
                create_time: item.try_get("", "create_time").unwrap(),
                update_time: item.try_get("", "update_time").unwrap(),
                ext: item.try_get("", "ext").unwrap(),
                rank_title: item.try_get("", "rank_title").unwrap(),
                rank_content: item.try_get("", "rank_content").unwrap(),
            }
        })
        .collect::<Vec<SearchItemSearchResp>>();

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
            let a = a.as_object_mut().unwrap();
            for (k, v) in b {
                merge(a.entry(k).or_insert(serde_json::Value::Null), v);
            }
        }
        (a, b) => *a = b,
    }
}
