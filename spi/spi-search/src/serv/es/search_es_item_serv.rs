use std::collections::{linked_list::IterMut, HashMap};

use bios_basic::{basic_enumeration::BasicQueryOpKind, helper::db_helper, spi::spi_funs::SpiBsInstExtractor};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    chrono::Utc,
    db::{
        reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
        sea_orm::Value,
    },
    log::debug,
    search::search_client::TardisSearchClient,
    serde_json::{self, json},
    web::web_resp::TardisPage,
    TardisFuns, TardisFunsInst,
};

use crate::dto::search_item_dto::{
    SearchItemAddReq, SearchItemModifyReq, SearchItemQueryReq, SearchItemSearchCtxReq, SearchItemSearchPageReq, SearchItemSearchQScopeKind, SearchItemSearchReq,
    SearchItemSearchResp,
};

use super::search_es_initializer;

pub async fn add(add_req: &mut SearchItemAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let client = funs.bs(ctx).await?.inst::<TardisSearchClient>().0;
    search_es_initializer::init_index(client, &add_req.tag).await?;
    if !search(
        &mut SearchItemSearchReq {
            tag: add_req.tag.clone(),
            ctx: SearchItemSearchCtxReq {
                accounts: None,
                apps: None,
                tenants: None,
                roles: None,
                groups: None,
                cond_by_or: None,
            },
            query: SearchItemQueryReq {
                keys: Some(vec![add_req.key.clone()]),
                own_paths: if let Some(own_paths) = &add_req.own_paths {
                    Some(vec![own_paths.clone()])
                } else {
                    None
                },
                ..Default::default()
            },
            sort: None,
            page: SearchItemSearchPageReq {
                number: 1,
                size: 1,
                fetch_total: false,
            },
        },
        funs,
        ctx,
    )
    .await?
    .records
    .is_empty()
    {
        return Err(funs.err().conflict("search_es_item_serv", "add", "record already exists", "409-search-already-exist"));
    }
    let data = TardisFuns::json.obj_to_string(add_req)?;
    client.create_record(&add_req.tag, &data).await?;

    Ok(())
}

pub async fn modify(tag: &str, key: &str, modify_req: &mut SearchItemModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let client = funs.bs(ctx).await?.inst::<TardisSearchClient>().0;
    // find id by this key
    let q = gen_query_dsl(&SearchItemSearchReq {
        tag: tag.to_string(),
        ctx: SearchItemSearchCtxReq {
            accounts: None,
            apps: None,
            tenants: None,
            roles: None,
            groups: None,
            cond_by_or: None,
        },
        query: SearchItemQueryReq {
            keys: Some(vec![key.to_string().into()]),
            own_paths: Some(vec![ctx.own_paths.clone()]),
            ..Default::default()
        },
        sort: None,
        page: SearchItemSearchPageReq {
            number: 1,
            size: 1,
            fetch_total: false,
        },
    })?;
    let search_result = client.raw_search(tag, &q, Some(1), Some(0)).await?;
    if search_result.hits.hits.is_empty() {
        return Err(funs.err().conflict("search_es_item_serv", "modify", "not found record", "404-not-found-record"));
    }
    let id = search_result.hits.hits[0]._id.clone();
    let mut query = HashMap::new();
    if let Some(kind) = &modify_req.kind {
        query.insert("kind", kind.clone());
    }
    if let Some(title) = &modify_req.title {
        query.insert("title", title.clone());
    }
    if let Some(content) = &modify_req.content {
        query.insert("content", content.clone());
    }
    if let Some(owner) = &modify_req.owner {
        query.insert("owner", owner.clone());
    }
    if let Some(own_paths) = &modify_req.own_paths {
        query.insert("own_paths", own_paths.clone());
    }
    if let Some(create_time) = &modify_req.create_time {
        query.insert("create_time", create_time.to_rfc3339());
    }
    if let Some(update_time) = &modify_req.update_time {
        query.insert("update_time", update_time.to_rfc3339());
    }
    if let Some(ext) = &modify_req.ext {
        let mut ext = ext.clone();
        if !modify_req.ext_override.unwrap_or(false) {
            let storage_ext = TardisFuns::json.str_to_obj::<SearchItemAddReq>(&client.get_record(tag, &id).await?)?.ext.unwrap_or_default();
            merge(&mut ext, storage_ext);
        }
        for (key, value) in ext.as_object().ok_or_else(|| funs.err().internal_error("search_es_item_serv", "modify", "ext is not object", "500-search-internal-error")) {
            query.insert(&format!("ext.{}", key), value.clone());
        }
    }
    client.update(tag, &id, query).await?;
    
    Ok(())
}

pub async fn delete(tag: &str, key: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let client = funs.bs(ctx).await?.inst::<TardisSearchClient>().0;
    let q = gen_query_dsl(&SearchItemSearchReq {
        tag: tag.to_string(),
        ctx: SearchItemSearchCtxReq {
            accounts: None,
            apps: None,
            tenants: None,
            roles: Some(ctx.roles.clone()),
            groups: Some(ctx.groups.clone()),
            cond_by_or: None,
        },
        query: SearchItemQueryReq {
            keys: Some(vec![key.to_string().into()]),
            ..Default::default()
        },
        sort: None,
        page: SearchItemSearchPageReq {
            number: 1,
            size: 1,
            fetch_total: false,
        },
    })?;
    client.delete_by_query(tag, &q).await?;

    Ok(())
}

pub async fn search(search_req: &mut SearchItemSearchReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<TardisPage<SearchItemSearchResp>> {
    let q = gen_query_dsl(search_req)?;
    debug!("q: {:?}", q.to_string());
    let client = funs.bs(ctx).await?.inst::<TardisSearchClient>().0;
    let result = client
        .raw_search(
            &search_req.tag,
            &q.to_string(),
            Some(search_req.page.size as i32),
            Some(((search_req.page.number - 1) * search_req.page.size as u32) as i32),
        )
        .await?;
    debug!("raw_search[result]: {:?}", result);

    let mut total_size: i64 = 0;
    if search_req.page.fetch_total && total_size == 0 {
        total_size = result.hits.total.value as i64;
    }
    let records = result
        .hits
        .hits
        .iter()
        .map(|item| TardisFuns::json.str_to_obj::<SearchItemAddReq>(&item._source.clone().to_string()))
        .collect::<Result<Vec<_>, _>>()?
        .iter()
        .map(|item| SearchItemSearchResp {
            kind: item.kind.clone(),
            key: item.key.to_string(),
            title: item.title.clone(),
            owner: item.owner.clone().unwrap_or_default(),
            own_paths: item.own_paths.clone().unwrap_or_default(),
            create_time: item.create_time.unwrap_or_default(),
            update_time: item.update_time.unwrap_or_default(),
            ext: item.ext.clone().unwrap_or_default(),
            rank_title: 0.0,
            rank_content: 0.0,
        })
        .collect::<Vec<_>>();
    Ok(TardisPage {
        page_size: search_req.page.size as u64,
        page_number: search_req.page.number as u64,
        total_size: total_size as u64,
        records,
    })
}

fn gen_query_dsl(search_req: &SearchItemSearchReq) -> TardisResult<String> {
    let mut must_q = vec![];
    let mut must_not_q = vec![];
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
            must_q.push(json!({
                "term": {"kind": kind},
            }));
        }
    }
    if let Some(keys) = &search_req.query.keys {
        for key in keys {
            must_q.push(json!({
                "term": {"key": key.to_string()},
            }));
        }
    }
    if let Some(owners) = &search_req.query.owners {
        for owner in owners {
            must_q.push(json!({
                "term": {"owner": owner},
            }));
        }
    }
    if let Some(own_paths) = &search_req.query.own_paths {
        for own_path in own_paths {
            must_q.push(json!({
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
        for cond_info in ext {
            match cond_info.op {
                BasicQueryOpKind::Eq => {
                    let field = format!("ext.{}", cond_info.field.clone());
                    must_q.push(json!({
                        "term": {field: cond_info.value.clone()}
                    }));
                }
                BasicQueryOpKind::Ne => {
                    let field = format!("ext.{}", cond_info.field.clone());
                    must_not_q.push(json!({
                        "term": { field: cond_info.value.clone()}
                    }));
                }
                BasicQueryOpKind::Gt => {
                    let field = format!("ext.{}", cond_info.field.clone());
                    filter_q.push(json!({
                        "range": {field: {"gt": cond_info.value.clone()}},
                    }));
                }
                BasicQueryOpKind::Ge => {
                    let field = format!("ext.{}", cond_info.field.clone());
                    filter_q.push(json!({
                        "range": {field: {"gte": cond_info.value.clone()}},
                    }));
                }
                BasicQueryOpKind::Lt => {
                    let field = format!("ext.{}", cond_info.field.clone());
                    filter_q.push(json!({
                        "range": {field: {"lt": cond_info.value.clone()}},
                    }));
                }
                BasicQueryOpKind::Le => {
                    let field = format!("ext.{}", cond_info.field.clone());
                    filter_q.push(json!({
                        "range": {field: {"lte": cond_info.value.clone()}},
                    }));
                }
                BasicQueryOpKind::Like => {
                    let field = format!("ext.{}", cond_info.field.clone());
                    must_q.push(json!({
                        "match": {field: cond_info.value.clone()}
                    }));
                }
                BasicQueryOpKind::In => {
                    let field = format!("ext.{}", cond_info.field.clone());
                    must_q.push(json!({
                        "terms": {
                            field: cond_info.value.clone()
                        }
                    }));
                }
            }
        }
    }
    if let Some(sorts) = &search_req.sort {
        for sort in sorts {
            sort_q.push(json!({sort.field.clone(): { "order": sort.order.to_sql() }}));
        }
    } else {
        sort_q.push(json!({"create_time": { "order": "asc", "unmapped_type": "date"}}));
    }
    let q = json!({
        "query": {
            "bool": {
                "must":must_q,
                "must_not":must_not_q,
                "should":should_q,
                "filter": filter_q,
            }
        },
        "sort": sort_q,
    });
    Ok(q.to_string())
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