use std::collections::HashMap;

use tardis::{
    basic::{dto::TardisContext, error::TardisError, result::TardisResult},
    search::search_client::TardisSearchClient,
    serde_json::{self, json, Value},
    web::web_resp::TardisPage,
    TardisFuns, TardisFunsInst,
};

use bios_basic::{
    enumeration::BasicQueryOpKind,
    spi::{spi_funs::SpiBsInst, spi_initializer::common},
};

use crate::dto::search_item_dto::{
    GroupSearchItemSearchReq, GroupSearchItemSearchResp, MultipleSearchItemSearchReq, SearchExportDataReq, SearchExportDataResp, SearchImportDataReq, SearchItemAddReq,
    SearchItemModifyReq, SearchItemQueryReq, SearchItemSearchCtxReq, SearchItemSearchPageReq, SearchItemSearchQScopeKind, SearchItemSearchReq, SearchItemSearchResp,
    SearchQueryMetricsReq, SearchQueryMetricsResp,
};

use super::search_es_initializer;
const INNER_FIELD: [&str; 7] = ["key", "title", "content", "owner", "own_paths", "create_time", "update_time"];

fn format_index(req_index: &str, ext: &HashMap<String, String>) -> String {
    if let Some(key_prefix) = common::get_isolation_flag_from_ext(ext) {
        format!("{key_prefix}{req_index}")
    } else {
        req_index.to_string()
    }
}

fn gen_data_mappings(ext: &Option<Value>) -> String {
    let mut ext_string = r#"{"type": "object"}"#.to_string();
    let mut ext_properties = vec![];
    if let Some(ext) = ext {
        for (k, v) in ext.as_object().expect("ext is not object") {
            if v.is_string() {
                ext_properties.push(format!(r#""{k}":{{"type": "keyword"}}"#));
            }
        }
    }
    if !ext_properties.is_empty() {
        ext_string = format!(
            "\"properties\": {{
            {}
        }}",
            ext_properties.join(",")
        );
    }

    format!(
        r#"{{
        "mappings": {{
            "properties": {{
                "tag":{{"type": "keyword"}},
                "kind":{{"type": "keyword"}},
                "key":{{"type": "keyword"}},
                "title":{{"type": "text"}},
                "content":{{"type": "text"}},
                "owner":{{"type": "keyword"}},
                "own_paths":{{"type": "keyword"}},
                "create_time":{{"type": "date"}},
                "update_time":{{"type": "date"}},
                "ext":{{{ext_string}}},
                "visit_keys":{{
                    "properties": {{
                        "accounts": {{ "type": "keyword" }},
                        "apps": {{ "type": "keyword" }},
                        "tenants": {{ "type": "keyword" }},
                        "roles": {{ "type": "keyword" }},
                        "groups": {{ "type": "keyword" }}
                      }}
                }}
            }}
        }}
    }}"#
    )
}

pub async fn add(add_req: &mut SearchItemAddReq, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let (client, ext, _) = inst.inst::<TardisSearchClient>();
    let index = format_index(&add_req.tag, ext);

    if search_es_initializer::init_index(client, &index, Some(&gen_data_mappings(&add_req.ext))).await.is_err() {
        return Err(funs.err().bad_request("search_es_item_serv", "add", "index not exist", "400-search-index-not-exist"));
    }
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
                // own_paths: add_req.own_paths.as_ref().map(|own_paths| vec![own_paths.clone()]),
                ..Default::default()
            },
            sort: None,
            page: SearchItemSearchPageReq {
                number: 1,
                size: 1,
                fetch_total: false,
            },
            adv_by_or: None,
            adv_query: None,
        },
        funs,
        ctx,
        inst,
    )
    .await?
    .records
    .is_empty()
    {
        return Err(funs.err().conflict("search_es_item_serv", "add", "record already exists", "409-search-already-exist"));
    }
    let data = TardisFuns::json.obj_to_string(add_req)?;
    client.create_record(&index, &data).await?;

    Ok(())
}

pub async fn modify(tag: &str, key: &str, modify_req: &mut SearchItemModifyReq, funs: &TardisFunsInst, _ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let (client, ext, _) = inst.inst::<TardisSearchClient>();
    let index = format_index(tag, ext);
    if !client.check_index_exist(&index).await? {
        return Err(funs.err().bad_request("search_es_item_serv", "add", "index not exist", "400-search-index-not-exist"));
    }
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
            ..Default::default()
        },
        sort: None,
        page: SearchItemSearchPageReq {
            number: 1,
            size: 1,
            fetch_total: false,
        },
        adv_by_or: None,
        adv_query: None,
    })?;
    let mut search_result = client.raw_search(&index, &q, Some(1), Some(0), None).await?;
    let id = search_result.hits.hits.pop().ok_or_else(|| funs.err().conflict("search_es_item_serv", "modify", "not found record", "404-not-found-record"))?._id.clone();
    let mut query = HashMap::new();
    if let Some(kind) = &modify_req.kind {
        query.insert("kind".to_string(), json!(kind.clone()).to_string());
    }
    if let Some(title) = &modify_req.title {
        query.insert("title".to_string(), json!(title.clone()).to_string());
    }
    if let Some(content) = &modify_req.content {
        query.insert("content".to_string(), json!(content.clone()).to_string());
    }
    if let Some(owner) = &modify_req.owner {
        query.insert("owner".to_string(), json!(owner.clone()).to_string());
    }
    if let Some(own_paths) = &modify_req.own_paths {
        query.insert("own_paths".to_string(), json!(own_paths.clone()).to_string());
    }
    if let Some(create_time) = &modify_req.create_time {
        query.insert("create_time".to_string(), json!(create_time.to_rfc3339()).to_string());
    }
    if let Some(update_time) = &modify_req.update_time {
        query.insert("update_time".to_string(), json!(update_time.to_rfc3339()).to_string());
    }
    if let Some(ext) = &modify_req.ext {
        let mut ext = ext.clone();
        if !modify_req.ext_override.unwrap_or(false) {
            let mut storage_ext = TardisFuns::json.str_to_obj::<SearchItemAddReq>(&client.get_record(&index, &id).await?)?.ext.unwrap_or_default();
            merge(&mut storage_ext, ext);
            ext = storage_ext;
        }
        if let Some(ext) = ext.as_object() {
            for (key, value) in ext {
                query.insert(format!("ext.{}", key), value.to_string());
            }
        }
    }
    if let Some(visit_keys) = &modify_req.visit_keys {
        if let Some(accounts) = &visit_keys.accounts {
            query.insert("visit_keys.accounts".to_string(), json!(accounts.clone()).to_string());
        }
        if let Some(apps) = &visit_keys.apps {
            query.insert("visit_keys.apps".to_string(), json!(apps.clone()).to_string());
        }
        if let Some(tenants) = &visit_keys.tenants {
            query.insert("visit_keys.tenants".to_string(), json!(tenants.clone()).to_string());
        }
        if let Some(roles) = &visit_keys.roles {
            query.insert("visit_keys.roles".to_string(), json!(roles.clone()).to_string());
        }
        if let Some(groups) = &visit_keys.groups {
            query.insert("visit_keys.groups".to_string(), json!(groups.clone()).to_string());
        }
    }

    client.update(&index, &id, query).await?;

    Ok(())
}

pub async fn delete(tag: &str, key: &str, funs: &TardisFunsInst, _ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let (client, ext, _) = inst.inst::<TardisSearchClient>();
    let index = format_index(tag, ext);
    if !client.check_index_exist(&index).await? {
        return Err(funs.err().bad_request("search_es_item_serv", "delete", "index not exist", "400-search-index-not-exist"));
    }
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
            ..Default::default()
        },
        sort: None,
        page: SearchItemSearchPageReq {
            number: 1,
            size: 1,
            fetch_total: false,
        },
        adv_by_or: None,
        adv_query: None,
    })?;
    client.delete_by_query(&index, &q).await?;

    Ok(())
}

pub async fn delete_by_ownership(tag: &str, onw_paths: &str, funs: &TardisFunsInst, _ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let (client, ext, _) = inst.inst::<TardisSearchClient>();
    let index = format_index(tag, ext);
    if !client.check_index_exist(&index).await? {
        return Err(funs.err().bad_request("search_es_item_serv", "delete", "index not exist", "400-search-index-not-exist"));
    }
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
            own_paths: Some(vec![onw_paths.to_string()]),
            ..Default::default()
        },
        sort: None,
        page: SearchItemSearchPageReq {
            number: 1,
            size: 1,
            fetch_total: false,
        },
        adv_by_or: None,
        adv_query: None,
    })?;
    client.delete_by_query(&index, &q).await?;

    Ok(())
}

pub async fn search(search_req: &mut SearchItemSearchReq, funs: &TardisFunsInst, _ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<TardisPage<SearchItemSearchResp>> {
    let q = gen_query_dsl(search_req)?;
    let mut track_scores = None;
    if let Some(sorts) = &search_req.sort {
        if sorts.iter().any(|sort| sort.field == "rank_title" || sort.field == "rank_content") {
            track_scores = Some(true);
        }
    }
    let (client, ext, _) = inst.inst::<TardisSearchClient>();
    let index = format_index(&search_req.tag, ext);
    if !client.check_index_exist(&index).await? {
        return Err(funs.err().bad_request("search_es_item_serv", "add", "index not exist", "400-search-index-not-exist"));
    }

    let result = client
        .raw_search(
            &index,
            &q,
            Some(search_req.page.size as i32),
            Some(((search_req.page.number - 1) * search_req.page.size as u32) as i32),
            track_scores,
        )
        .await?;

    let mut total_size: i64 = 0;
    if search_req.page.fetch_total && total_size == 0 {
        total_size = result.hits.total.value as i64;
    }
    let records = result
        .hits
        .hits
        .iter()
        .map(|raw_item| {
            if let Ok(item) = TardisFuns::json.str_to_obj::<SearchItemAddReq>(&raw_item._source.clone().to_string()) {
                Ok(SearchItemSearchResp {
                    kind: item.kind.clone(),
                    key: item.key.to_string(),
                    title: item.title.clone(),
                    content: item.content.clone(),
                    data_source: item.data_source.clone().unwrap_or_default(),
                    owner: item.owner.clone().unwrap_or_default(),
                    own_paths: item.own_paths.clone().unwrap_or_default(),
                    create_time: item.create_time.unwrap_or_default(),
                    update_time: item.update_time.unwrap_or_default(),
                    ext: item.ext.unwrap_or_default(),
                    rank_title: raw_item._score.unwrap_or_default(),
                    rank_content: raw_item._score.unwrap_or_default(),
                })
            } else {
                Err(funs.err().format_error("search_es_item_serv", "search", "search result format error", "500-result-format-error"))
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TardisPage {
        page_size: search_req.page.size as u64,
        page_number: search_req.page.number as u64,
        total_size: total_size as u64,
        records,
    })
}

pub async fn group_search(search_req: &mut GroupSearchItemSearchReq, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<Vec<GroupSearchItemSearchResp>> {
    Ok(vec![])
}

pub async fn multiple_search(
    search_req: &mut MultipleSearchItemSearchReq,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<TardisPage<serde_json::Value>> {
    Ok(TardisPage {
        page_size: search_req.page.size as u64,
        page_number: search_req.page.number as u64,
        total_size: 0,
        records: vec![],
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
        must_q.push(json!({
            "terms": {
                "kind": kinds.clone()
            }
        }));
    }
    if let Some(keys) = &search_req.query.keys {
        must_q.push(json!({
            "terms": {
                "key": keys.clone()
            }
        }));
    }
    if let Some(owners) = &search_req.query.owners {
        must_q.push(json!({
            "terms": {
                "owner": owners.clone()
            }
        }));
    }
    if let Some(own_paths) = &search_req.query.own_paths {
        must_q.push(json!({
            "terms": {
                "own_paths": own_paths.clone()
            }
        }));
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
            let field = format!("ext.{}", cond_info.field.clone());
            match cond_info.op {
                BasicQueryOpKind::Eq => {
                    must_q.push(json!({
                        "term": {field: cond_info.value.clone()}
                    }));
                }
                BasicQueryOpKind::Ne => {
                    must_not_q.push(json!({
                        "term": { field: cond_info.value.clone()}
                    }));
                }
                BasicQueryOpKind::Gt => {
                    filter_q.push(json!({
                        "range": {field: {"gt": cond_info.value.clone()}},
                    }));
                }
                BasicQueryOpKind::Ge => {
                    filter_q.push(json!({
                        "range": {field: {"gte": cond_info.value.clone()}},
                    }));
                }
                BasicQueryOpKind::Lt => {
                    filter_q.push(json!({
                        "range": {field: {"lt": cond_info.value.clone()}},
                    }));
                }
                BasicQueryOpKind::Le => {
                    filter_q.push(json!({
                        "range": {field: {"lte": cond_info.value.clone()}},
                    }));
                }
                BasicQueryOpKind::Like
                | BasicQueryOpKind::LLike
                | BasicQueryOpKind::RLike
                | BasicQueryOpKind::NotLike
                | BasicQueryOpKind::NotLLike
                | BasicQueryOpKind::NotRLike => {
                    must_q.push(json!({
                        "match": {field: cond_info.value.clone()}
                    }));
                }
                BasicQueryOpKind::In => {
                    let value = if cond_info.value.is_array() {
                        cond_info.value.clone()
                    } else {
                        json!(vec![cond_info.value.clone()])
                    };
                    must_q.push(json!({
                        "terms": {
                            field: value
                        }
                    }));
                }
                BasicQueryOpKind::NotIn => {
                    let value = if cond_info.value.is_array() {
                        cond_info.value.clone()
                    } else {
                        json!(vec![cond_info.value.clone()])
                    };
                    must_not_q.push(json!({
                        "terms": { field: value}
                    }));
                }
                BasicQueryOpKind::IsNull => {
                    must_not_q.push(json!({
                        "exists": {"field": field}
                    }));
                }
                BasicQueryOpKind::IsNotNull => {
                    must_q.push(json!({
                        "exists": {"field": field}
                    }));
                }
                BasicQueryOpKind::IsNullOrEmpty => {
                    must_q.push(json!({
                        "bool": {
                            "should": [
                                {"term": {field.clone(): "".to_string()}},
                                {"bool": {
                                    "must_not": [{
                                        "exists": {"field": field}
                                    }],
                                }}
                            ]
                        }
                    }));
                }
                BasicQueryOpKind::Len => {
                    return Err(TardisError {
                        code: "500-not-supports".to_owned(),
                        message: "search_es_item_serv len op not supports".to_owned(),
                    });
                }
                BasicQueryOpKind::ArrayLen => {
                    return Err(TardisError {
                        code: "500-not-supports".to_owned(),
                        message: "search_es_item_serv len op not supports".to_owned(),
                    });
                }
            }
        }
    }

    if let Some(adv_query) = &search_req.adv_query {
        let mut adv_query_must_q = vec![];
        let mut adv_query_should_q = vec![];
        for group_query in adv_query {
            let mut group_query_q: Vec<Value> = vec![];
            for cond_info in group_query.ext.clone().unwrap_or_default() {
                let field = if !INNER_FIELD.contains(&cond_info.field.clone().as_str()) {
                    format!("ext.{}", cond_info.field)
                } else {
                    cond_info.field.clone()
                };
                match cond_info.op {
                    BasicQueryOpKind::Eq => {
                        group_query_q.push(json!({
                            "term": {field: cond_info.value.clone()}
                        }));
                    }
                    BasicQueryOpKind::Ne => {
                        group_query_q.push(json!({
                            "bool": {
                                "must_not": {
                                    "term": { field: cond_info.value.clone()}
                                }
                            }
                        }));
                    }
                    BasicQueryOpKind::Gt => {
                        group_query_q.push(json!({
                            "bool": {
                                "filter": {
                                    "range": {field: {"gt": cond_info.value.clone()}},
                                }
                            }
                        }));
                    }
                    BasicQueryOpKind::Ge => {
                        group_query_q.push(json!({
                            "bool": {
                                "filter": {
                                    "range": {field: {"gte": cond_info.value.clone()}},
                                }
                            }
                        }));
                    }
                    BasicQueryOpKind::Lt => {
                        group_query_q.push(json!({
                            "bool": {
                                "filter": {
                                    "range": {field: {"lt": cond_info.value.clone()}},
                                }
                            }
                        }));
                    }
                    BasicQueryOpKind::Le => {
                        group_query_q.push(json!({
                            "bool": {
                                "filter": {
                                    "range": {field: {"lte": cond_info.value.clone()}},
                                }
                            }
                        }));
                    }
                    BasicQueryOpKind::Like | BasicQueryOpKind::LLike | BasicQueryOpKind::RLike => {
                        group_query_q.push(json!({
                            "match": {field: cond_info.value.clone()}
                        }));
                    }
                    BasicQueryOpKind::NotLike | BasicQueryOpKind::NotLLike | BasicQueryOpKind::NotRLike => {
                        group_query_q.push(json!({
                            "bool": {
                                "must_not": {
                                    "match": { field: cond_info.value.clone()}
                                }
                            }
                        }));
                    }
                    BasicQueryOpKind::In => {
                        let value = if cond_info.value.is_array() {
                            cond_info.value.clone()
                        } else {
                            json!(vec![cond_info.value.clone()])
                        };
                        group_query_q.push(json!({
                            "terms": {
                                field: value
                            }
                        }));
                    }
                    BasicQueryOpKind::NotIn => {
                        let value = if cond_info.value.is_array() {
                            cond_info.value.clone()
                        } else {
                            json!(vec![cond_info.value.clone()])
                        };
                        group_query_q.push(json!({
                            "bool": {
                                "must_not": {
                                    "terms": { field: value}
                                }
                            }
                        }));
                    }
                    BasicQueryOpKind::IsNull => {
                        group_query_q.push(json!({
                            "bool": {
                                "must_not": {
                                    "exists": {"field": field}
                                }
                            }
                        }));
                    }
                    BasicQueryOpKind::IsNotNull => {
                        group_query_q.push(json!({
                            "exists": {"field": field}
                        }));
                    }
                    BasicQueryOpKind::IsNullOrEmpty => {
                        group_query_q.push(json!({
                            "bool": {
                                "should": [
                                    {"term": {field.clone(): "".to_string()}},
                                    {"bool": {
                                        "must_not": [{
                                            "exists": {"field": field}
                                        }],
                                    }}
                                ]
                            }
                        }));
                    }
                    BasicQueryOpKind::Len => {
                        return Err(TardisError {
                            code: "500-not-supports".to_owned(),
                            message: "search_es_item_serv len op not supports".to_owned(),
                        })
                    }
                    BasicQueryOpKind::ArrayLen => {
                        return Err(TardisError {
                            code: "500-not-supports".to_owned(),
                            message: "search_es_item_serv len op not supports".to_owned(),
                        })
                    }
                }
            }
            match group_query.group_by_or.unwrap_or(false) {
                true => {
                    adv_query_must_q.push(json!({
                        "bool": {
                            "must": if group_query.ext_by_or.unwrap_or(false) { group_query_q.clone() } else { vec![] },
                            "should": if group_query.ext_by_or.unwrap_or(false) { vec![] } else { group_query_q.clone() },
                        }
                    }));
                }
                false => {
                    adv_query_should_q.push(json!({
                        "bool": {
                            "must": if group_query.ext_by_or.unwrap_or(false) { group_query_q.clone() } else { vec![] },
                            "should": if group_query.ext_by_or.unwrap_or(false) { vec![] } else { group_query_q.clone() },
                        }
                    }));
                }
            }
        }
        must_q.push(json!({
                "bool": {
                    "must": adv_query_must_q,
                    "should": adv_query_should_q,
                }
        }));
    }
    if let Some(sorts) = &search_req.sort {
        for sort_item in sorts {
            if sort_item.field.to_lowercase() == "key"
                || sort_item.field.to_lowercase() == "title"
                || sort_item.field.to_lowercase() == "owner"
                || sort_item.field.to_lowercase() == "own_paths"
                || sort_item.field.to_lowercase() == "create_time"
                || sort_item.field.to_lowercase() == "update_time"
            {
                sort_q.push(json!({sort_item.field.clone(): { "order": sort_item.order.to_sql() }}));
            } else if sort_item.field.to_lowercase() == "rank_title" || sort_item.field.to_lowercase() == "rank_content" {
                sort_q.push(json!({"_score": { "order": sort_item.order.to_sql() }}));
            } else {
                let sort_ket = format!("ext.{}", sort_item.field.clone());
                sort_q.push(json!({sort_ket: { "order": sort_item.order.to_sql() }}));
            }
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

pub async fn query_metrics(_query_req: &SearchQueryMetricsReq, funs: &TardisFunsInst, _ctx: &TardisContext, _inst: &SpiBsInst) -> TardisResult<SearchQueryMetricsResp> {
    Err(funs.err().format_error("search_es_item_serv", "query_metrics", "not supports", "500-not-supports"))
}

pub async fn refresh_tsv(_tag: &str, funs: &TardisFunsInst, _ctx: &TardisContext, _inst: &SpiBsInst) -> TardisResult<()> {
    Err(funs.err().format_error("search_es_item_serv", "refresh_tsv", "not supports", "500-not-supports"))
}

pub async fn export_data(_export_req: &SearchExportDataReq, _funs: &TardisFunsInst, _ctx: &TardisContext, _inst: &SpiBsInst) -> TardisResult<SearchExportDataResp> {
    let tag_data = HashMap::new();
    Ok(SearchExportDataResp { tag_data })
}

pub async fn import_data(_import_req: &SearchImportDataReq, _funs: &TardisFunsInst, _ctx: &TardisContext, _inst: &SpiBsInst) -> TardisResult<bool> {
    Ok(true)
}
