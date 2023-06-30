use std::collections::linked_list::IterMut;

use bios_basic::{basic_enumeration::BasicQueryOpKind, helper::db_helper, spi::spi_funs::SpiBsInstExtractor};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    chrono::Utc,
    db::{
        reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
        sea_orm::Value,
    },
    search::search_client::TardisSearchClient,
    serde_json::{self, json},
    web::web_resp::TardisPage,
    TardisFuns, TardisFunsInst, log::debug,
};

use crate::dto::search_item_dto::{SearchItemAddReq, SearchItemModifyReq, SearchItemSearchQScopeKind, SearchItemSearchReq, SearchItemSearchResp};

use super::search_es_initializer;

pub async fn add(add_req: &mut SearchItemAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let data = format!(r#"{{"data": {}}}"#, TardisFuns::json.obj_to_string(add_req)?);
    let client = funs.bs(ctx).await?.inst::<TardisSearchClient>().0;
    search_es_initializer::init_index(client, &add_req.tag).await?;
    client.create_record(&add_req.tag, &data).await?;
    Ok(())
}

pub async fn modify(tag: &str, key: &str, modify_req: &mut SearchItemModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    panic!("not implemented")
}

pub async fn delete(tag: &str, key: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    panic!("not implemented")
}

pub async fn search(search_req: &mut SearchItemSearchReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<TardisPage<SearchItemSearchResp>> {
    let mut must_q = vec![];
    let mut should_q = vec![];
    let mut filter_q = vec![];
    let mut sort_q = vec![];

    // ctx
    let mut ctx_q = vec![];
    if let Some(accounts) = &search_req.ctx.accounts {
        ctx_q.push(json!({
            "terms": {
                "visit_keys.accounts": accounts
            }
        }));
    }
    if let Some(apps) = &search_req.ctx.apps {
        ctx_q.push(json!({
            "terms": {
                "visit_keys.apps": apps
            }
        }));
    }
    if let Some(tenants) = &search_req.ctx.tenants {
        ctx_q.push(json!({
            "terms": {
                "visit_keys.tenants": tenants
            }
        }));
    }
    if let Some(roles) = &search_req.ctx.roles {
        ctx_q.push(json!({
            "terms": {
                "visit_keys.roles": roles
            }
        }));
    }
    if let Some(groups) = &search_req.ctx.groups {
        ctx_q.push(json!({
            "terms": {
                "visit_keys.groups": groups
            }
        }));
    }
    if search_req.ctx.cond_by_or.unwrap_or(false) {
        should_q.append(&mut ctx_q);
    } else {
        must_q.append(&mut ctx_q);
    }
    // query
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
        match search_req.query.q_scope.as_ref().unwrap_or(&SearchItemSearchQScopeKind::Title) {
            SearchItemSearchQScopeKind::Title => {
                let q_q = if q.contains('|') {
                    let mut q_q_should = vec![];
                    for s in q.split('|') {
                        q_q_should.push(json!({"match": { "title": s }}));
                    }
                    json!({
                        "bool": {"should": q_q_should}
                    })
                } else if q.contains('&') {
                    let mut q_q_must = vec![];
                    for s in q.split('&') {
                        q_q_must.push(json!({"match": { "title": s }}));
                    }
                    json!({"bool": {"must": q_q_must}})
                } else if q.contains('!') {
                    let mut q = q;
                    json!({"bool": {"must_not":{ "match": { "title": q.split_off(1) }}}})
                } else {
                    json!({"match": { "title": q }})
                };
                must_q.push(q_q);
            }
            SearchItemSearchQScopeKind::Content => {
                let q_q = if q.contains('|') {
                    let mut q_q_should = vec![];
                    for s in q.split('|') {
                        q_q_should.push(json!({"match": { "content": s }}));
                    }
                    json!({"bool": {"should": q_q_should}})
                } else if q.contains('&') {
                    let mut q_q_must = vec![];
                    for s in q.split('&') {
                        q_q_must.push(json!({"match": { "content": s }}));
                    }
                    json!({"bool": {"must": q_q_must}})
                } else if q.contains('!') {
                    let mut q = q;
                    json!({"bool": {"must_not":{ "match": { "content": q.split_off(1) }}}
                    })
                } else {
                    json!({"match": { "content": q }})
                };
                must_q.push(q_q);
            }
            SearchItemSearchQScopeKind::TitleContent => {
                let q_q = if q.contains('|') {
                    let mut q_q_content_should = vec![];
                    let mut q_q_title_should = vec![];
                    for s in q.split('|') {
                        q_q_title_should.push(json!({"match": { "title": s }}));
                        q_q_content_should.push(json!({"match": { "content": s }}));
                    }
                    json!({
                        "bool": {"should": [{"bool": {"should": q_q_title_should}},{"bool": {"should": q_q_content_should}}]}})
                } else if q.contains('&') {
                    let mut q_q_content_should = vec![];
                    let mut q_q_title_should = vec![];
                    for s in q.split('|') {
                        q_q_title_should.push(json!({"match": { "title": s }}));
                        q_q_content_should.push(json!({"match": { "content": s }}));
                    }
                    json!({
                        "bool": {
                            "must": [
                                {"bool": {"should": q_q_title_should}},
                                {"bool": {"should": q_q_content_should}}
                            ]
                        }
                    })
                } else if q.contains('!') {
                    let mut q = q;
                    json!({
                        "bool": {
                            "must_not":[
                                { "match": { "title": q.split_off(1) }},
                                { "match": { "content": q.split_off(1) }}
                            ]
                        }
                    })
                } else {
                    json!({
                        "bool": {
                            "should":[
                                { "match": { "title": q }},
                                { "match": { "content": q }}
                            ]
                        }
                    })
                };
                must_q.push(q_q);
            }
        }
    }
    if let Some(kinds) = &search_req.query.kinds {
        for kind in kinds {
            should_q.push(json!({
                "term": {"kind": kind},
            }));
        }
    }
    if let Some(keys) = &search_req.query.keys {
        for key in keys {
            should_q.push(json!({
                "term": {"key": key.to_string()},
            }));
        }
    }
    if let Some(owners) = &search_req.query.owners {
        for owner in owners {
            should_q.push(json!({
                "term": {"owner": owner},
            }));
        }
    }
    if let Some(own_paths) = &search_req.query.own_paths {
        for own_path in own_paths {
            should_q.push(json!({
                "term": {"own_path": own_path},
            }));
        }
    }
    if let Some(own_paths) = &search_req.query.own_paths {
        for own_path in own_paths {
            should_q.push(json!({
                "term": {"own_path": own_path},
            }));
        }
    }
    if let (Some(create_time_start), Some(create_time_end)) = (&search_req.query.create_time_start, &search_req.query.create_time_end) {
        filter_q.push(json!({
            "range": {"create_time": {"gte": create_time_start, "lt": create_time_end}},
        }));
    }
    if let (Some(update_time_start), Some(update_time_end)) = (&search_req.query.update_time_start, &search_req.query.update_time_end) {
        filter_q.push(json!({
            "range": {"update_time": {"gte": update_time_start, "lt": update_time_end}},
        }));
    }
    if let Some(ext) = &search_req.query.ext {
        // TODO
    }
    if let Some(sorts) = &search_req.sort {
        for sort in sorts {
            sort_q.push(json!({sort.field.clone(): { "order": sort.order.to_sql() }}));
        }
    }
    let q = json!({
        "query": {
            "bool": {
                "must":if must_q.is_empty() {json!({})} else {json!(must_q)},
                "should":if should_q.is_empty() {json!({})} else {json!(should_q)},
                "filter": if filter_q.is_empty() {json!({})} else {json!(filter_q)},
            }
        },
        "sort": if sort_q.is_empty() {json!({})} else {json!(sort_q)},
    });
    debug!("q: {:?}", q.to_string());
    let client = funs.bs(ctx).await?.inst::<TardisSearchClient>().0;
    let result = client.raw_search(&search_req.tag, &q.to_string(), Some(search_req.page.size as i32), Some((search_req.page.number * search_req.page.size as u32) as i32)).await?;
    debug!("raw_search[result]: {:?}", result);

    let mut total_size: i64 = 0;
    if search_req.page.fetch_total && total_size == 0 {
        total_size = result.hits.total.value as i64;
    }
    let records = result.hits.hits.iter().map(|item| TardisFuns::json.str_to_obj::<SearchItemSearchResp>(&item._source.clone().to_string())).collect::<Result<Vec<_>, _>>()?;

    Ok(TardisPage {
        page_size: search_req.page.size as u64,
        page_number: search_req.page.number as u64,
        total_size: total_size as u64,
        records,
    })
}
