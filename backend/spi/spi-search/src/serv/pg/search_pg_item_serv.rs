use std::{collections::HashMap, vec};

use itertools::Itertools;
use pinyin::{to_pinyin_vec, Pinyin};
use tardis::{
    basic::{dto::TardisContext, error::TardisError, result::TardisResult}, chrono::{Duration, Utc}, db::{
        reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
        sea_orm::{FromQueryResult, Value},
    }, futures::future::join_all, regex::Regex, serde_json::{self, json, Map}, web::web_resp::TardisPage, TardisFuns, TardisFunsInst
};

use bios_basic::{dto::BasicQueryCondInfo, enumeration::BasicQueryOpKind, helper::db_helper, spi::spi_funs::SpiBsInst};

use crate::{
    dto::search_item_dto::{
        AdvSearchItemQueryReq, GroupSearchItemSearchReq, GroupSearchItemSearchResp, SearchExportAggResp, SearchExportDataReq, SearchExportDataResp, SearchImportDataReq,
        SearchItemAddReq, SearchItemModifyReq, SearchItemQueryReq, SearchItemSearchCtxReq, SearchItemSearchPageReq, SearchItemSearchQScopeKind, SearchItemSearchReq,
        SearchItemSearchResp, SearchItemSearchSortReq, SearchQueryMetricsReq, SearchQueryMetricsResp, SearchWordCombinationsRuleWay,
    },
    search_config::SearchConfig,
};

use super::search_pg_initializer;

const FUNCTION_SUFFIX_FLAG: &str = "__";
const FUNCTION_EXT_SUFFIX_FLAG: &str = "_ext_";
const INNER_FIELD: [&str; 7] = ["key", "title", "content", "owner", "own_paths", "create_time", "update_time"];

pub async fn add(add_req: &mut SearchItemAddReq, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let mut params = Vec::new();
    params.push(Value::from(add_req.kind.to_string()));
    params.push(Value::from(add_req.key.to_string()));
    params.push(Value::from(add_req.title.as_str()));
    params.push(Value::from(title_tsv(add_req.title.as_str(), funs).await?));

    params.push(Value::from(add_req.content.as_str()));
    params.push(Value::from(add_req.content.as_str()));
    // params.push(Value::from(format!(
    //     "{},{}",
    //     add_req.content.as_str(),
    //     to_pinyin_vec(add_req.content.as_str(), Pinyin::plain).join(",")
    // )));
    params.push(Value::from(add_req.data_source.as_ref().unwrap_or(&"".to_string()).as_str()));
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
    let word_combinations_way = if add_req.title.chars().count() > funs.conf::<SearchConfig>().split_strategy_rule_config.specify_word_length.unwrap_or(30) {
        get_tokenizer()
    } else {
        "simple".to_string()
    };
    conn.execute_one(
        &format!(
            r#"INSERT INTO {table_name} 
    (kind, key, title, title_tsv, content, content_tsv, data_source, owner, own_paths, create_time, update_time, ext, visit_keys)
VALUES
    ($1, $2, $3, to_tsvector('{word_combinations_way}', $4), $5, to_tsvector('{}', $6), $7, $8, $9, $10, $11, $12, {})"#,
            get_tokenizer(),
            if add_req.visit_keys.is_some() { "$13" } else { "null" },
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
        let word_combinations_way = if title.chars().count() > funs.conf::<SearchConfig>().split_strategy_rule_config.specify_word_length.unwrap_or(30) {
            get_tokenizer()
        } else {
            "simple".to_string()
        };
        sql_sets.push(format!("title_tsv = to_tsvector('{word_combinations_way}', ${})", params.len() + 1));
        params.push(Value::from(title_tsv(title, funs).await?));
    };
    if let Some(content) = &modify_req.content {
        sql_sets.push(format!("content = ${}", params.len() + 1));
        sql_sets.push(format!("content_tsv = to_tsvector('{}', ${})", get_tokenizer(), params.len() + 1));
        params.push(Value::from(content));
        // params.push(Value::from(format!("{},{}", content, to_pinyin_vec(content, Pinyin::plain).join(","))));
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

async fn title_tsv(title: &str, funs: &TardisFunsInst) -> TardisResult<String> {
    let pinyin_vec = to_pinyin_vec(title, Pinyin::plain);
    let content = title.split(' ').last().unwrap_or_default();
    let title_tsv = if title.chars().count() > funs.conf::<SearchConfig>().split_strategy_rule_config.specify_word_length.unwrap_or(30) {
        format!(
            "{} {} {} {} {}",
            title,
            pinyin_vec.clone().into_iter().map(|pinyin| pinyin.chars().next().unwrap_or_default()).join(""),
            generate_word_combinations(title, SearchWordCombinationsRuleWay::Number).join(" "),
            generate_word_combinations(title, SearchWordCombinationsRuleWay::SpecSymbols(vec!["-".to_string(), "_".to_string()])).join(" "),
            generate_word_combinations(&pinyin_vec.join(" "), SearchWordCombinationsRuleWay::SpecSymbols(vec![" ".to_string()])).join(" "),
        )
    } else {
        format!(
            "{} {} {} {} {} {} {} {}",
            title,
            pinyin_vec.clone().into_iter().map(|pinyin| pinyin.chars().next().unwrap_or_default()).join(""),
            generate_word_combinations(title, SearchWordCombinationsRuleWay::Number).join(" "),
            generate_word_combinations(content, SearchWordCombinationsRuleWay::SpecLength(1)).join(" "),
            generate_word_combinations(content, SearchWordCombinationsRuleWay::SpecLength(2)).join(" "),
            generate_word_combinations(content, SearchWordCombinationsRuleWay::SpecLength(3)).join(" "),
            generate_word_combinations(title, SearchWordCombinationsRuleWay::SpecSymbols(vec!["-".to_string(), "_".to_string()])).join(" "),
            generate_word_combinations(&pinyin_vec.join(" "), SearchWordCombinationsRuleWay::SpecSymbols(vec![" ".to_string()])).join(" "),
        )
    };
    Ok(title_tsv)
}

fn generate_word_combinations_with_length(original_str: &str, split_len: usize) -> Vec<String> {
    let mut combinations = Vec::new();
    let original_chars = original_str.chars().map(|c| c.to_string()).collect::<Vec<_>>();
    if original_chars.len() > split_len {
        for i in 0..original_chars.len() - split_len + 1 {
            let word = original_chars[i..=(i + split_len - 1)].join("");
            combinations.push(word);
        }
    }
    combinations
}

fn generate_word_combinations_with_symbol(original_str: &str, symbols: Vec<String>) -> Vec<String> {
    let mut combinations = Vec::new();
    for symbol in symbols {
        let splited_words = original_str.split(&symbol).collect_vec();
        if splited_words.len() == 1 {
            continue;
        }
        for i in 0..splited_words.len() {
            for j in i..splited_words.len() {
                let word = splited_words[i..=j].join(&symbol);
                combinations.push(word);
            }
        }
    }
    combinations.into_iter().map(|word| word.to_string()).collect_vec()
}

fn generate_word_combinations_with_digit(chars: &str) -> Vec<String> {
    if let Ok(re) = Regex::new(r"(\d+)") {
        re.find_iter(chars).map( |m| m.as_str().to_string()).collect_vec()
    } else {
        vec![]
    }
}


fn generate_word_combinations(chars: &str, way: SearchWordCombinationsRuleWay) -> Vec<String> {
    match way {
        // 按数字分词
        SearchWordCombinationsRuleWay::Number => {
            generate_word_combinations_with_digit(chars)
        },
        // 指定长度分词
        SearchWordCombinationsRuleWay::SpecLength(len) => {
            generate_word_combinations_with_length(chars, len)
        },
        // 指定分隔符分词
        SearchWordCombinationsRuleWay::SpecSymbols(allowed_symbols) => {
            generate_word_combinations_with_symbol(chars, allowed_symbols)
        },
    }
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
    let mut where_fragments: Vec<String> = vec!["1=1".to_string()];
    let mut sql_vals: Vec<Value> = vec![];
    let table_alias_name = "search_item";
    // query
    let (select_fragments, from_fragments) = package_query(table_alias_name, search_req.query.clone(), &mut sql_vals, &mut where_fragments, funs)?;

    // Add visit_keys filter
    package_visit_filter(table_alias_name, search_req.ctx.clone(), &mut sql_vals, &mut where_fragments)?;

    let order_fragments = package_order(table_alias_name, search_req.sort.clone())?;

    // advanced query
    let sql_adv_query = package_adv_query(table_alias_name, search_req.adv_query.clone(), &mut sql_vals, funs)?;

    // page
    let page_fragments = package_page(search_req.page.clone(), &mut sql_vals)?;

    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (conn, table_name) = search_pg_initializer::init_table_and_conn(bs_inst, &search_req.tag, ctx, false).await?;
    let result = conn
        .query_all(
            format!(
                r#"SELECT kind, key, title, data_source, owner, own_paths, create_time, update_time, ext{}{}{}
FROM {table_name} {table_alias_name}{}
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
                    format!(
                        " {} ( 1=1 {})",
                        if search_req.adv_by_or.unwrap_or(false) { " OR " } else { " AND " },
                        sql_adv_query.join(" ")
                    )
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
                data_source: item.try_get("", "data_source").unwrap_or_default(),
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

pub async fn group_search(search_req: &mut GroupSearchItemSearchReq, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<Vec<GroupSearchItemSearchResp>> {
    let mut where_fragments: Vec<String> = vec!["1=1".to_string()];
    let mut sql_vals: Vec<Value> = vec![];
    let table_alias_name = "search_item";
    let group_column = if INNER_FIELD.contains(&search_req.group_column.clone().as_str()) {
        search_req.group_column.clone()
    } else {
        format!("ext->>'{}'", search_req.group_column)
    };

    // query
    let (_, from_fragments) = package_query(table_alias_name, search_req.query.clone(), &mut sql_vals, &mut where_fragments, funs)?;

    // Add visit_keys filter
    package_visit_filter(table_alias_name, search_req.ctx.clone(), &mut sql_vals, &mut where_fragments)?;

    // advanced query
    let sql_adv_query = package_adv_query(table_alias_name, search_req.adv_query.clone(), &mut sql_vals, funs)?;

    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (conn, table_name) = search_pg_initializer::init_table_and_conn(bs_inst, &search_req.tag, ctx, false).await?;
    let result = conn
        .query_all(
            format!(
                r#"SELECT {group_column} as group_column, count(*) as count
FROM {table_name} {table_alias_name}{}
WHERE 
    {}
    {}
    group by {group_column}
    "#,
                from_fragments,
                where_fragments.join(" AND "),
                if sql_adv_query.is_empty() {
                    "".to_string()
                } else {
                    format!(" AND ( 1=1 {})", sql_adv_query.join(" "))
                },
            )
            .as_str(),
            sql_vals,
        )
        .await?;

    let result = result
        .into_iter()
        .map(|item| {
            Ok(GroupSearchItemSearchResp {
                group_column: item.try_get("", "group_column")?,
                count: item.try_get("", "count")?,
            })
        })
        .collect::<TardisResult<Vec<GroupSearchItemSearchResp>>>()?;
    Ok(result)
}

fn package_query(
    table_alias_name: &str,
    query: SearchItemQueryReq,
    sql_vals: &mut Vec<Value>,
    where_fragments: &mut Vec<String>,
    funs: &TardisFunsInst,
) -> TardisResult<(String, String)> {
    let select_fragments;
    let mut from_fragments = "".to_string();
    if let Some(q) = &query.q {
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
        from_fragments = format!(
            ", plainto_tsquery('{}', ${}) AS query1, plainto_tsquery('simple', ${}) AS query2",
            get_tokenizer(),
            sql_vals.len(),
            sql_vals.len()
        );

        let rank_title = format!(
            "GREATEST(COALESCE(ts_rank({}.title_tsv, query1), 0 :: float4), COALESCE(ts_rank({}.title_tsv, query2), 0 :: float4)) AS rank_title",
            table_alias_name, table_alias_name,
        );
        let rank_content = format!(
            "GREATEST(COALESCE(ts_rank({}.content_tsv, query1), 0 :: float4), COALESCE(ts_rank({}.content_tsv, query2), 0 :: float4)) AS rank_content",
            table_alias_name, table_alias_name
        );
        match query.q_scope.as_ref().unwrap_or(&SearchItemSearchQScopeKind::Title) {
            SearchItemSearchQScopeKind::Title => {
                select_fragments = format!(", {}, 0::float4 AS rank_content", rank_title);
                where_fragments.push(format!("(query1 @@ {}.title_tsv OR query2 @@ {}.title_tsv)", table_alias_name, table_alias_name));
                // sql_vals.push(Value::from(format!("%{q}%")));
                // where_fragments.push(format!("(query @@ title_tsv OR title LIKE ${})", sql_vals.len()));
            }
            SearchItemSearchQScopeKind::Content => {
                select_fragments = format!(", 0::float4 AS rank_title, {}", rank_content);
                where_fragments.push(format!("(query1 @@ {}.content_tsv OR query2 @@ {}.content_tsv)", table_alias_name, table_alias_name));
                // sql_vals.push(Value::from(format!("%{q}%")));
                // where_fragments.push(format!("(query @@ content_tsv OR content LIKE ${})", sql_vals.len()));
            }
            SearchItemSearchQScopeKind::TitleContent => {
                select_fragments = format!(", {}, {}", rank_title, rank_content);
                where_fragments.push(format!(
                    "((query1 @@ {}.title_tsv OR query2 @@ {}.title_tsv) OR (query1 @@ {}.content_tsv OR query2 @@ {}.content_tsv))",
                    table_alias_name, table_alias_name, table_alias_name, table_alias_name
                ));
                // sql_vals.push(Value::from(format!("%{q}%")));
                // where_fragments.push(format!(
                //     "(query @@ title_tsv OR query @@ content_tsv OR title LIKE ${} OR content LIKE ${})",
                //     sql_vals.len(),
                //     sql_vals.len()
                // ));
            }
        }
    } else {
        select_fragments = ", 0::float4 AS rank_title, 0::float4 AS rank_content".to_string();
    }

    if let Some(kinds) = query.kinds {
        if !kinds.is_empty() {
            where_fragments.push(format!(
                "{}.kind = ANY (ARRAY[{}])",
                table_alias_name,
                (0..kinds.len()).map(|idx| format!("${}", sql_vals.len() + idx + 1)).collect::<Vec<String>>().join(",")
            ));
            for kind in kinds {
                sql_vals.push(Value::from(kind.to_string()));
            }
        }
    }
    if let Some(keys) = query.keys {
        if !keys.is_empty() {
            where_fragments.push(format!(
                "{}.key LIKE ANY (ARRAY[{}])",
                table_alias_name,
                (0..keys.len()).map(|idx| format!("${}", sql_vals.len() + idx + 1)).collect::<Vec<String>>().join(",")
            ));
            for key in keys {
                sql_vals.push(Value::from(format!("{key}%")));
            }
        }
    }
    if let Some(owners) = query.owners {
        if !owners.is_empty() {
            where_fragments.push(format!(
                "{}.owner LIKE ANY (ARRAY[{}])",
                table_alias_name,
                (0..owners.len()).map(|idx| format!("${}", sql_vals.len() + idx + 1)).collect::<Vec<String>>().join(",")
            ));
            for owner in owners {
                sql_vals.push(Value::from(format!("{owner}%")));
            }
        }
    }
    if let Some(own_paths) = query.own_paths {
        if !own_paths.is_empty() {
            where_fragments.push(format!(
                "{}.own_paths in ({})",
                table_alias_name,
                (0..own_paths.len()).map(|idx| format!("${}", sql_vals.len() + idx + 1)).collect::<Vec<String>>().join(",")
            ));
            for own_path in own_paths {
                sql_vals.push(Value::from(own_path.to_string()));
            }
        }
    }
    if let Some(own_paths) = query.rlike_own_paths {
        if !own_paths.is_empty() {
            where_fragments.push(format!(
                "{}.own_paths LIKE ANY (ARRAY[{}])",
                table_alias_name,
                (0..own_paths.len()).map(|idx| format!("${}", sql_vals.len() + idx + 1)).collect::<Vec<String>>().join(",")
            ));
            for own_path in own_paths {
                sql_vals.push(Value::from(format!("{own_path}%")));
            }
        }
    }
    if let Some(create_time_start) = query.create_time_start {
        sql_vals.push(Value::from(create_time_start));
        where_fragments.push(format!("{}.create_time >= ${}", table_alias_name, sql_vals.len()));
    }
    if let Some(create_time_end) = query.create_time_end {
        sql_vals.push(Value::from(create_time_end));
        where_fragments.push(format!("{}.create_time <= ${}", table_alias_name, sql_vals.len()));
    }
    if let Some(update_time_start) = query.update_time_start {
        sql_vals.push(Value::from(update_time_start));
        where_fragments.push(format!("{}.update_time >= ${}", table_alias_name, sql_vals.len()));
    }
    if let Some(update_time_end) = query.update_time_end {
        sql_vals.push(Value::from(update_time_end));
        where_fragments.push(format!("{}.update_time <= ${}", table_alias_name, sql_vals.len()));
    }
    package_ext(table_alias_name, query.ext.clone(), sql_vals, where_fragments, funs)?;
    Ok((select_fragments, from_fragments))
}

fn package_adv_query(table_alias_name: &str, adv_query: Option<Vec<AdvSearchItemQueryReq>>, sql_vals: &mut Vec<Value>, funs: &TardisFunsInst) -> TardisResult<Vec<String>> {
    // advanced query
    let mut sql_adv_query = vec![];
    if let Some(adv_query) = &adv_query {
        for group_query in adv_query {
            let mut sql_and_where = vec![];
            package_ext(table_alias_name, group_query.ext.clone(), sql_vals, &mut sql_and_where, funs)?;
            if !sql_and_where.is_empty() {
                sql_adv_query.push(format!(
                    " {} ( {} )",
                    if group_query.group_by_or.unwrap_or(false) { "OR" } else { "AND" },
                    sql_and_where.join(if group_query.ext_by_or.unwrap_or(false) { " OR " } else { " AND " })
                ));
            }
        }
    }
    Ok(sql_adv_query)
}

fn package_order(table_alias_name: &str, sort: Option<Vec<SearchItemSearchSortReq>>) -> TardisResult<Vec<String>> {
    let mut order_fragments: Vec<String> = Vec::new();
    if let Some(sort) = &sort {
        for sort_item in sort {
            if sort_item.field.to_lowercase() == "key"
                || sort_item.field.to_lowercase() == "title"
                || sort_item.field.to_lowercase() == "content"
                || sort_item.field.to_lowercase() == "owner"
                || sort_item.field.to_lowercase() == "own_paths"
                || sort_item.field.to_lowercase() == "create_time"
                || sort_item.field.to_lowercase() == "update_time"
            {
                order_fragments.push(format!("{}.{} {}", table_alias_name, sort_item.field, sort_item.order.to_sql()));
            } else {
                order_fragments.push(format!("{}.ext -> '{}' {}", table_alias_name, sort_item.field, sort_item.order.to_sql()));
            }
        }
    }
    Ok(order_fragments)
}

fn package_visit_filter(table_alias_name: &str, ctx: SearchItemSearchCtxReq, sql_vals: &mut Vec<Value>, where_fragments: &mut Vec<String>) -> TardisResult<()> {
    // Add visit_keys filter
    let req_ctx = ctx.to_sql();
    if !req_ctx.is_empty() {
        let mut where_visit_keys_fragments = Vec::new();
        for (scope_key, scope_values) in req_ctx {
            if scope_values.is_empty() {
                continue;
            }
            if scope_values.len() == 1 {
                where_visit_keys_fragments.push(format!("{}.visit_keys -> '{scope_key}' ? ${}", table_alias_name, sql_vals.len() + 1));
            } else {
                where_visit_keys_fragments.push(format!(
                    "{}.visit_keys -> '{scope_key}' ?| array[{}]",
                    table_alias_name,
                    (0..scope_values.len()).map(|idx| format!("${}", sql_vals.len() + idx + 1)).collect::<Vec<String>>().join(", ")
                ));
            }
            for scope_value in scope_values {
                sql_vals.push(Value::from(scope_value));
            }
        }
        where_fragments.push(format!(
            "({}.visit_keys IS NULL OR ({}))",
            table_alias_name,
            where_visit_keys_fragments.join(if ctx.cond_by_or.unwrap_or(false) { " OR " } else { " AND " })
        ));
    }
    Ok(())
}

fn package_page(page: SearchItemSearchPageReq, sql_vals: &mut Vec<Value>) -> TardisResult<String> {
    sql_vals.push(Value::from(page.size));
    sql_vals.push(Value::from((page.number - 1) * page.size as u32));
    Ok(format!("LIMIT ${} OFFSET ${}", sql_vals.len() - 1, sql_vals.len()))
}

fn package_ext(
    table_alias_name: &str,
    ext: Option<Vec<BasicQueryCondInfo>>,
    sql_vals: &mut Vec<Value>,
    sql_and_where: &mut Vec<String>,
    funs: &TardisFunsInst,
) -> TardisResult<()> {
    let err_not_found = |ext_item: &BasicQueryCondInfo| {
        Err(funs.err().not_found(
            "item",
            "search",
            &format!("The ext field=[{}] value=[{}] operation=[{}] is not legal.", &ext_item.field, ext_item.value, &ext_item.op,),
            "404-spi-search-op-not-legal",
        ))
    };
    if let Some(ext) = &ext {
        for ext_item in ext {
            let value = db_helper::json_to_sea_orm_value(&ext_item.value, &ext_item.op);
            let Some(mut value) = value else { return err_not_found(&ext_item.clone()) };
            if !INNER_FIELD.contains(&ext_item.field.clone().as_str()) {
                if ext_item.op == BasicQueryOpKind::In {
                    if value.len() == 1 {
                        sql_and_where.push(format!("{}.ext -> '{}' ? ${}", table_alias_name, ext_item.field, sql_vals.len() + 1));
                    } else {
                        sql_and_where.push(format!(
                            "{}.ext -> '{}' ?| array[{}]",
                            table_alias_name,
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
                        sql_and_where.push(format!("not ({}.ext -> '{}' ? ${})", table_alias_name, ext_item.field, sql_vals.len() + 1));
                    } else {
                        sql_and_where.push(format!(
                            "not ({}.ext -> '{}' ?| array[{}])",
                            table_alias_name,
                            ext_item.field,
                            (0..value.len()).map(|idx| format!("${}", sql_vals.len() + idx + 1)).collect::<Vec<String>>().join(", ")
                        ));
                    }
                    for val in value {
                        sql_vals.push(val);
                    }
                } else if ext_item.op == BasicQueryOpKind::IsNull {
                    sql_and_where.push(format!("{}.ext ->> '{}' is null", table_alias_name, ext_item.field));
                } else if ext_item.op == BasicQueryOpKind::IsNotNull {
                    sql_and_where.push(format!(
                        "({}.ext ->> '{}' is not null and {}.ext ->> '{}' != '' and {}.ext ->> '{}' != '[]')",
                        table_alias_name, ext_item.field, table_alias_name, ext_item.field, table_alias_name, ext_item.field
                    ));
                } else if ext_item.op == BasicQueryOpKind::IsNullOrEmpty {
                    sql_and_where.push(format!(
                        "({}.ext ->> '{}' is null or {}.ext ->> '{}' = '' or {}.ext ->> '{}' = '[]')",
                        table_alias_name, ext_item.field, table_alias_name, ext_item.field, table_alias_name, ext_item.field
                    ));
                } else if ext_item.op == BasicQueryOpKind::Len {
                    if let Some(first_value) = value.pop() {
                        sql_and_where.push(format!("(length(ext->>'{}') = ${})", ext_item.field, sql_vals.len() + 1));
                        sql_vals.push(first_value);
                    } else {
                        return err_not_found(ext_item);
                    };
                } else {
                    if value.len() > 1 {
                        return err_not_found(&ext_item.clone());
                    }
                    let Some(value) = value.pop() else {
                        return Err(funs.err().bad_request("item", "search", "Request item using 'IN' operator show hava a value", "400-spi-item-op-in-without-value"));
                    };
                    if let Value::Bool(_) = value {
                        sql_and_where.push(format!(
                            "({}.ext ->> '{}')::boolean {} ${}",
                            table_alias_name,
                            ext_item.field,
                            ext_item.op.to_sql(),
                            sql_vals.len() + 1
                        ));
                    } else if let Value::TinyInt(_) = value {
                        sql_and_where.push(format!(
                            "({}.ext ->> '{}')::smallint {} ${}",
                            table_alias_name,
                            ext_item.field,
                            ext_item.op.to_sql(),
                            sql_vals.len() + 1
                        ));
                    } else if let Value::SmallInt(_) = value {
                        sql_and_where.push(format!(
                            "({}.ext ->> '{}')::smallint {} ${}",
                            table_alias_name,
                            ext_item.field,
                            ext_item.op.to_sql(),
                            sql_vals.len() + 1
                        ));
                    } else if let Value::Int(_) = value {
                        sql_and_where.push(format!(
                            "({}.ext ->> '{}')::integer {} ${}",
                            table_alias_name,
                            ext_item.field,
                            ext_item.op.to_sql(),
                            sql_vals.len() + 1
                        ));
                    } else if let Value::BigInt(_) = value {
                        sql_and_where.push(format!(
                            "({}.ext ->> '{}')::bigint {} ${}",
                            table_alias_name,
                            ext_item.field,
                            ext_item.op.to_sql(),
                            sql_vals.len() + 1
                        ));
                    } else if let Value::TinyUnsigned(_) = value {
                        sql_and_where.push(format!(
                            "({}.ext ->> '{}')::smallint {} ${}",
                            table_alias_name,
                            ext_item.field,
                            ext_item.op.to_sql(),
                            sql_vals.len() + 1
                        ));
                    } else if let Value::SmallUnsigned(_) = value {
                        sql_and_where.push(format!(
                            "({}.ext ->> '{}')::integer {} ${}",
                            table_alias_name,
                            ext_item.field,
                            ext_item.op.to_sql(),
                            sql_vals.len() + 1
                        ));
                    } else if let Value::Unsigned(_) = value {
                        sql_and_where.push(format!(
                            "({}.ext ->> '{}')::bigint {} ${}",
                            table_alias_name,
                            ext_item.field,
                            ext_item.op.to_sql(),
                            sql_vals.len() + 1
                        ));
                    } else if let Value::BigUnsigned(_) = value {
                        // TODO
                        sql_and_where.push(format!(
                            "({}.ext ->> '{}')::bigint {} ${}",
                            table_alias_name,
                            ext_item.field,
                            ext_item.op.to_sql(),
                            sql_vals.len() + 1
                        ));
                    } else if let Value::Float(_) = value {
                        sql_and_where.push(format!(
                            "({}.ext ->> '{}')::real {} ${}",
                            table_alias_name,
                            ext_item.field,
                            ext_item.op.to_sql(),
                            sql_vals.len() + 1
                        ));
                    } else if let Value::Double(_) = value {
                        sql_and_where.push(format!(
                            "({}.ext ->> '{}')::double precision {} ${}",
                            table_alias_name,
                            ext_item.field,
                            ext_item.op.to_sql(),
                            sql_vals.len() + 1
                        ));
                    } else if value.is_chrono_date_time_utc() {
                        sql_and_where.push(format!(
                            "({}.ext ->> '{}')::timestamp with time zone {} ${}",
                            table_alias_name,
                            ext_item.field,
                            ext_item.op.to_sql(),
                            sql_vals.len() + 1
                        ));
                    } else {
                        sql_and_where.push(format!(
                            "{}.ext ->> '{}' {} ${}",
                            table_alias_name,
                            ext_item.field,
                            ext_item.op.to_sql(),
                            sql_vals.len() + 1
                        ));
                    }
                    sql_vals.push(value);
                }
            } else if ext_item.op == BasicQueryOpKind::In {
                if !value.is_empty() {
                    sql_and_where.push(format!(
                        "{}.{} IN ({})",
                        table_alias_name,
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
                        "{}.{} NOT IN ({})",
                        table_alias_name,
                        ext_item.field,
                        (0..value.len()).map(|idx| format!("${}", sql_vals.len() + idx + 1)).collect::<Vec<String>>().join(",")
                    ));
                    for val in value {
                        sql_vals.push(val);
                    }
                }
            } else if ext_item.op == BasicQueryOpKind::IsNull {
                sql_and_where.push(format!("{}.{} is null", table_alias_name, ext_item.field));
            } else if ext_item.op == BasicQueryOpKind::IsNotNull {
                sql_and_where.push(format!("({}.{} is not null)", table_alias_name, ext_item.field));
            } else if ext_item.op == BasicQueryOpKind::IsNullOrEmpty {
                sql_and_where.push(format!(
                    "({}.{} is null or {}.{}::text = '' )",
                    table_alias_name, ext_item.field, table_alias_name, ext_item.field
                ));
            } else {
                if value.len() > 1 {
                    return err_not_found(&ext_item.clone());
                }
                let Some(value) = value.pop() else {
                    return Err(funs.err().bad_request("item", "search", "Request item using 'IN' operator show have a value", "400-spi-item-op-in-without-value"));
                };
                if let Value::Bool(_) = value {
                    sql_and_where.push(format!(
                        "({}.{}::boolean) {} ${}",
                        table_alias_name,
                        ext_item.field,
                        ext_item.op.to_sql(),
                        sql_vals.len() + 1
                    ));
                } else if let Value::TinyInt(_) = value {
                    sql_and_where.push(format!(
                        "({}.{}::smallint) {} ${}",
                        table_alias_name,
                        ext_item.field,
                        ext_item.op.to_sql(),
                        sql_vals.len() + 1
                    ));
                } else if let Value::SmallInt(_) = value {
                    sql_and_where.push(format!(
                        "({}.{}::smallint) {} ${}",
                        table_alias_name,
                        ext_item.field,
                        ext_item.op.to_sql(),
                        sql_vals.len() + 1
                    ));
                } else if let Value::Int(_) = value {
                    sql_and_where.push(format!(
                        "({}.{}::integer) {} ${}",
                        table_alias_name,
                        ext_item.field,
                        ext_item.op.to_sql(),
                        sql_vals.len() + 1
                    ));
                } else if let Value::BigInt(_) = value {
                    sql_and_where.push(format!(
                        "({}.{}::bigint) {} ${}",
                        table_alias_name,
                        ext_item.field,
                        ext_item.op.to_sql(),
                        sql_vals.len() + 1
                    ));
                } else if let Value::TinyUnsigned(_) = value {
                    sql_and_where.push(format!(
                        "({}.{}::smallint) {} ${}",
                        table_alias_name,
                        ext_item.field,
                        ext_item.op.to_sql(),
                        sql_vals.len() + 1
                    ));
                } else if let Value::SmallUnsigned(_) = value {
                    sql_and_where.push(format!(
                        "({}.{}::integer) {} ${}",
                        table_alias_name,
                        ext_item.field,
                        ext_item.op.to_sql(),
                        sql_vals.len() + 1
                    ));
                } else if let Value::Unsigned(_) = value {
                    sql_and_where.push(format!(
                        "({}.{}::bigint) {} ${}",
                        table_alias_name,
                        ext_item.field,
                        ext_item.op.to_sql(),
                        sql_vals.len() + 1
                    ));
                } else if let Value::BigUnsigned(_) = value {
                    // TODO
                    sql_and_where.push(format!(
                        "({}.{}::bigint) {} ${}",
                        table_alias_name,
                        ext_item.field,
                        ext_item.op.to_sql(),
                        sql_vals.len() + 1
                    ));
                } else if let Value::Float(_) = value {
                    sql_and_where.push(format!("({}.{}::real) {} ${}", table_alias_name, ext_item.field, ext_item.op.to_sql(), sql_vals.len() + 1));
                } else if let Value::Double(_) = value {
                    sql_and_where.push(format!(
                        "({}.{}::double precision) {} ${}",
                        table_alias_name,
                        ext_item.field,
                        ext_item.op.to_sql(),
                        sql_vals.len() + 1
                    ));
                } else {
                    sql_and_where.push(format!("{}.{} {} ${}", table_alias_name, ext_item.field, ext_item.op.to_sql(), sql_vals.len() + 1));
                }
                sql_vals.push(value);
            }
        }
    }
    Ok(())
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
    let table_alias_name = "fact";
    let conf_limit = query_req.conf_limit.unwrap_or(100);
    // Package filter
    let mut sql_part_wheres = vec![];

    let (select_fragments, from_fragments) = package_query(table_alias_name, query_req.query.clone(), &mut params, &mut sql_part_wheres, funs)?;

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
    package_visit_filter(table_alias_name, query_req.ctx.clone(), &mut params, &mut sql_part_wheres)?;

    // advanced query
    let sql_adv_query = package_adv_query(table_alias_name, query_req.adv_query.clone(), &mut params, funs)?;

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

            sql_part_group_infos.push((
                format!(
                    "case when _.{} IS NULL {} THEN '\"empty\"' else {} end",
                    &group.code,
                    if INNER_FIELD.contains(&group.code.clone().as_str()) || group.time_window.is_none() {
                        "".to_string()
                    } else {
                        format!("OR _.{} = ''", &group.code.clone())
                    },
                    column_name_with_fun
                ),
                alias_name.clone(),
                alias_name,
            ));
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
        // let select_column = if select.in_ext.unwrap_or(true) {
        //     format!("_.ext ->> '{}'", &select.code)
        // } else {
        //     format!("_.{}", &select.code)
        // };
        let select_column = format!("_.{}", &select.code);
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
                if order.in_ext.unwrap_or(true) {
                    format!("fact.ext ->> '{}' {}", order.code, if order.asc { "ASC" } else { "DESC" })
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

    let sql_part_wheres = if sql_part_wheres.is_empty() {
        "".to_string()
    } else {
        format!(" AND {}", sql_part_wheres.join(" AND "))
    };
    let sql_adv_query = if sql_adv_query.is_empty() {
        "".to_string()
    } else {
        format!(" AND ( 1=1 {})", sql_adv_query.join(" "))
    };
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (conn, table_name) = search_pg_initializer::init_table_and_conn(bs_inst, &query_req.tag, ctx, false).await?;
    let ignore_group_agg = sql_part_groups.is_empty() || !query_req.group_agg.unwrap_or(false);
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
            let val = first_result.get(measure_key).ok_or_else(|| format!("failed to get key {measure_key}"))?;
            let val = if measure_key.ends_with(&format!("{FUNCTION_SUFFIX_FLAG}avg")) {
                // Fix `avg` function return type error
                let val = val
                    .as_str()
                    .ok_or_else(|| format!("value of field {measure_key} should be a string"))?
                    .parse::<f64>()
                    .map_err(|_| format!("value of field {measure_key} can not be parsed as a valid f64 number"))?;
                serde_json::Value::from(val)
            } else {
                val.clone()
            };
            leaf_node.insert(measure_key.to_string(), val.clone());
        }
        if !ignore_group_agg {
            leaf_node.insert("group".to_string(), first_result.get("group").ok_or_else(|| "failed to get key group".to_string())?.clone());
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

pub async fn refresh_tsv(tag: &str, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (conn, table_name) = search_pg_initializer::init_table_and_conn(bs_inst, tag, ctx, false).await?;
    let result = conn.query_all(&format!("SELECT key, title FROM {table_name}"), vec![]).await?;
    let max_size = result.len();
    let mut page = 0;
    let page_size = 2000;
    loop {
        let current_result = &result[((page * page_size).min(max_size))..(((page + 1) * page_size).min(max_size))];
        if current_result.is_empty() {
            break;
        }
        join_all(
            current_result
                .iter()
                .map(|row| async move {
                    modify(
                        tag,
                        row.try_get::<String>("", "key").expect("not found key").as_str(),
                        &mut SearchItemModifyReq {
                            title: Some(row.try_get("", "title").expect("not found title")),
                            update_time: Some(Utc::now()),
                            ..Default::default()
                        },
                        funs,
                        ctx,
                        inst,
                    )
                    .await
                    .expect("modify error")
                })
                .collect_vec(),
        )
        .await;
        page += 1;
    }

    Ok(())
}

fn get_tokenizer() -> String {
    #[cfg(feature = "with-cn-tokenizer")]
    {
        "public.chinese_zh".to_string()
    }
    #[cfg(not(feature = "with-cn-tokenizer"))]
    {
        "simple".to_string()
    }
}

pub async fn export_data(export_req: &SearchExportDataReq, _funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<SearchExportDataResp> {
    let mut tag_data = HashMap::new();
    let bs_inst = inst.inst::<TardisRelDBClient>();
    for tag in &export_req.tags {
        let (conn, table_name) = search_pg_initializer::init_table_and_conn(bs_inst, tag, ctx, false).await?;
        let mut params = vec![Value::from(format!("{}%", ctx.own_paths.clone()))];
        let start_time = export_req.start_time.unwrap_or_else(|| Utc::now() - Duration::days(365 * 2));
        let end_time = export_req.end_time.unwrap_or_else(Utc::now);
        params.push(Value::from(start_time));
        params.push(Value::from(end_time));
        let kind_sql = if let Some(tag_kind) = &export_req.tag_kind {
            if tag_kind.contains_key(tag) && !tag_kind.get(tag).unwrap_or(&vec![]).is_empty() {
                let kinds = tag_kind.get(tag).unwrap_or(&vec![]).clone();
                let kind_sql = format!(
                    "AND kind IN ({})",
                    (0..kinds.len()).map(|idx| format!("${}", params.len() + idx + 1)).collect::<Vec<String>>().join(",")
                );
                for kind in &kinds {
                    params.push(Value::from(kind));
                }
                kind_sql
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        };
        let result = conn
            .query_all(
                &format!(
                    "
                SELECT key, kind, title, content, data_source, owner, own_paths, create_time, update_time, ext, visit_keys 
                FROM {table_name} 
                WHERE 
                    own_paths like $1
                    and (
                        (create_time > $2 and create_time < $3)
                     or (update_time > $2 and update_time <= $3)
                    )
                    {kind_sql}
                order by create_time desc",
                ),
                params,
            )
            .await?;
        let result = result
            .into_iter()
            .map(|item| {
                Ok(SearchExportAggResp {
                    kind: item.try_get("", "kind")?,
                    key: item.try_get("", "key")?,
                    title: item.try_get("", "title")?,
                    content: item.try_get("", "content")?,
                    data_source: item.try_get("", "data_source").unwrap_or_default(),
                    owner: item.try_get("", "owner")?,
                    own_paths: item.try_get("", "own_paths")?,
                    create_time: item.try_get("", "create_time")?,
                    update_time: item.try_get("", "update_time")?,
                    ext: item.try_get("", "ext")?,
                    visit_keys: item.try_get("", "visit_keys")?,
                    tag: tag.to_string(),
                })
            })
            .collect::<TardisResult<Vec<SearchExportAggResp>>>()?;
        tag_data.insert(tag.to_string(), result);
    }
    Ok(SearchExportDataResp { tag_data })
}

pub async fn import_data(import_req: &SearchImportDataReq, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<bool> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let mut num = 0;
    for (tag, tag_data) in &import_req.tag_data {
        let (conn, table_name) = search_pg_initializer::init_table_and_conn(bs_inst, tag, ctx, false).await?;
        for data in tag_data {
            num += 1;
            if num % 100 == 0 {
                tardis::tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
            let key = data.key.clone();
            let sql = format!("SELECT * FROM {} WHERE key = $1", table_name);
            let result = conn.query_all(&sql, vec![Value::from(key.clone())]).await?;
            if result.is_empty() {
                add(
                    &mut SearchItemAddReq {
                        tag: tag.to_string(),
                        kind: data.kind.clone(),
                        key: key.into(),
                        title: data.title.clone(),
                        content: data.content.clone(),
                        data_source: Some(import_req.data_source.clone()),
                        owner: Some(data.owner.clone()),
                        own_paths: Some(data.own_paths.clone()),
                        create_time: Some(data.create_time),
                        update_time: Some(data.update_time),
                        ext: Some(data.ext.clone()),
                        visit_keys: data.visit_keys.clone(),
                    },
                    funs,
                    ctx,
                    inst,
                )
                .await?;
            } else {
                modify(
                    tag,
                    &key,
                    &mut SearchItemModifyReq {
                        title: Some(data.title.clone()),
                        content: Some(data.content.clone()),
                        owner: Some(data.owner.clone()),
                        own_paths: Some(data.own_paths.clone()),
                        create_time: Some(data.create_time),
                        update_time: Some(data.update_time),
                        ext: Some(data.ext.clone()),
                        visit_keys: data.visit_keys.clone(),
                        kind: Some(data.kind.clone()),
                        ext_override: Some(true),
                    },
                    funs,
                    ctx,
                    inst,
                )
                .await?;
            }
        }
    }
    Ok(true)
}
