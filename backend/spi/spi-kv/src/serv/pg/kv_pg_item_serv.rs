use bios_basic::{
    rbum::{dto::rbum_filer_dto::RbumBasicFilterReq, helper::rbum_scope_helper},
    spi::spi_funs::{SpiBsInst, SpiBsInstExtractor},
};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult}, db::{reldb_client::TardisRelDBClient, sea_orm::Value}, serde_json::json, web::web_resp::TardisPage, TardisFuns, TardisFunsInst
};

use crate::{dto::kv_item_dto::{KvItemAddOrModifyReq, KvItemDetailResp, KvItemMatchReq, KvItemSummaryResp, KvNameAddOrModifyReq, KvNameFindResp, KvTagAddOrModifyReq, KvTagFindResp}, kv_constants};

use super::kv_pg_initializer;

pub async fn add_or_modify_item(add_or_modify_req: &KvItemAddOrModifyReq, _funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let params = vec![
        Value::from(add_or_modify_req.key.to_string()),
        Value::from(add_or_modify_req.value.clone()),
        Value::from(add_or_modify_req.info.as_ref().unwrap_or(&"".to_string()).as_str()),
        Value::from(ctx.owner.clone()),
        Value::from(ctx.own_paths.clone()),
        Value::from(add_or_modify_req.scope_level.unwrap_or(0)),
    ];
    let mut update_opt_fragments: Vec<&str> = Vec::new();
    update_opt_fragments.push("v = $2");
    if add_or_modify_req.info.is_some() {
        update_opt_fragments.push("info = $3");
    }
    update_opt_fragments.push("owner = $4");
    update_opt_fragments.push("own_paths = $5");
    if add_or_modify_req.scope_level.is_some() {
        update_opt_fragments.push("scope_level = $6");
    }
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = kv_pg_initializer::init_table_and_conn(bs_inst, ctx, true).await?;
    conn.begin().await?;
    conn.execute_one(
        &format!(
            r#"INSERT INTO {} 
    (k, v, info, owner, own_paths, scope_level)
VALUES
    ($1, $2, $3, $4, $5, $6)
ON CONFLICT (k)
DO UPDATE SET
    {}
"#,
            table_name,
            update_opt_fragments.join(", ")
        ),
        params,
    )
    .await?;
    conn.commit().await?;
    Ok(())
}

pub async fn add_or_modify_key_name(add_or_modify_req: &mut KvNameAddOrModifyReq, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let req = KvItemAddOrModifyReq {
        key: format!("{}{}", kv_constants::KEY_PREFIX_BY_KEY_NAME, add_or_modify_req.key).into(),
        value: json!(add_or_modify_req.name),
        scope_level: add_or_modify_req.scope_level,
        info: None,
    };
    self::add_or_modify_item(&req, funs, ctx, inst).await
}


pub async fn add_or_modify_tag(add_or_modify_req: &mut KvTagAddOrModifyReq, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let req = KvItemAddOrModifyReq {
        key: format!("{}{}", kv_constants::KEY_PREFIX_BY_TAG, add_or_modify_req.key).into(),
        value: TardisFuns::json.obj_to_json(&add_or_modify_req.items)?,
        scope_level: add_or_modify_req.scope_level,
        info: None,
    };
    self::add_or_modify_item(&req, funs, ctx, &inst).await
}

pub async fn get_item(key: String, extract: Option<String>, _funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<Option<KvItemDetailResp>> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (conn, table_name) = kv_pg_initializer::init_table_and_conn(bs_inst, ctx, true).await?;
    let result = conn
        .get_dto_by_sql::<KvItemDetailResp>(
            &format!(
                r#"SELECT k AS key, v{} AS value, info, owner, own_paths, scope_level, create_time, update_time
FROM {}
WHERE 
    k = $1"#,
                if let Some(extract) = extract { format!("->'{extract}'") } else { "".to_string() },
                table_name,
            ),
            vec![Value::from(key)],
        )
        .await?;
    if let Some(detail) = result.as_ref() {
        if !rbum_scope_helper::check_scope(
            &detail.own_paths,
            Some(detail.scope_level),
            &RbumBasicFilterReq {
                ignore_scope: false,
                ..Default::default()
            },
            &ctx.own_paths,
        ) {
            return Ok(None);
        }
    }
    Ok(result)
}

pub async fn find_items(keys: Vec<String>, extract: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext, _inst: &SpiBsInst) -> TardisResult<Vec<KvItemSummaryResp>> {
    let mut sql_vals: Vec<Value> = vec![];
    let place_holder = keys
        .iter()
        .map(|key| {
            sql_vals.push(Value::from(key.to_string()));
            format!("${}", sql_vals.len())
        })
        .collect::<Vec<String>>()
        .join(",");
    let inst_arc = funs.bs(ctx).await?;
    let bs_inst = inst_arc.inst::<TardisRelDBClient>();
    let (conn, table_name) = kv_pg_initializer::init_table_and_conn(bs_inst, ctx, true).await?;
    let result = conn
        .find_dtos_by_sql::<KvItemSummaryResp>(
            &format!(
                r#"SELECT k AS key, v{} AS value, info, owner, own_paths, scope_level, create_time, update_time
FROM {}
WHERE 
    k IN ({})"#,
                if let Some(extract) = extract { format!("->'{extract}'") } else { "".to_string() },
                table_name,
                place_holder
            ),
            sql_vals,
        )
        .await?
        .into_iter()
        .filter(|item| {
            rbum_scope_helper::check_scope(
                &item.own_paths,
                Some(item.scope_level),
                &RbumBasicFilterReq {
                    ignore_scope: false,
                    ..Default::default()
                },
                &ctx.own_paths,
            )
        })
        .collect();
    Ok(result)
}


pub async fn find_key_names(keys: Vec<String>, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<Vec<KvNameFindResp>> {
    let keys = keys.into_iter().map(|key| format!("{}{}", kv_constants::KEY_PREFIX_BY_KEY_NAME, key)).collect();
    self::find_items(keys, None, funs, ctx, &inst).await
    .and_then(|items| {
        items
            .into_iter()
            .map::<TardisResult<KvNameFindResp>, _>(|item| {
                Ok(KvNameFindResp {
                    key: item.key.strip_prefix(kv_constants::KEY_PREFIX_BY_KEY_NAME).unwrap_or("").to_string(),
                    name: item.value.as_str().unwrap_or("").to_string(),
                    create_time: item.create_time,
                    update_time: item.update_time,
                })
            })
            .collect::<TardisResult<Vec<KvNameFindResp>>>()
    })
}


pub async fn find_tags(keys: Vec<String>, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<Vec<KvTagFindResp>> {
    let keys = keys.iter().map(|r| format!("{}{}", kv_constants::KEY_PREFIX_BY_TAG, r)).collect::<Vec<_>>();
    self::find_items(keys, None, funs, ctx, &inst).await
    .and_then(|items| {
        items
            .into_iter()
            .map(|item| {
                Ok(KvTagFindResp {
                    key: item.key.strip_prefix(kv_constants::KEY_PREFIX_BY_TAG).unwrap_or("").to_string(),
                    items: TardisFuns::json.json_to_obj(item.value)?,
                    create_time: item.create_time,
                    update_time: item.update_time,
                })
            })
            .collect::<TardisResult<Vec<_>>>()
    })
}
pub async fn match_items(match_req: KvItemMatchReq, _funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<TardisPage<KvItemSummaryResp>> {
    let mut where_fragments: Vec<String> = Vec::new();
    let mut sql_vals: Vec<Value> = vec![];
    let mut order_fragments: Vec<String> = Vec::new();
    sql_vals.push(Value::from(format!("{}%", match_req.key_prefix)));
    where_fragments.push(format!("k LIKE ${}", sql_vals.len()));

    if let Some(query_path) = match_req.query_path {
        let query_values = if let Some(query_values) = match_req.query_values {
            query_values.to_string()
        } else {
            "".to_string()
        };
        where_fragments.push(format!(
            "jsonb_path_exists(
            v,
            '{query_path}',
            '{query_values}'
          )"
        ));
    }

    if let Some(create_time_start) = match_req.create_time_start {
        sql_vals.push(Value::from(create_time_start));
        where_fragments.push(format!("create_time >= ${}", sql_vals.len()));
    }
    if let Some(create_time_end) = match_req.create_time_end {
        sql_vals.push(Value::from(create_time_end));
        where_fragments.push(format!("create_time <= ${}", sql_vals.len()));
    }

    if let Some(update_time_start) = match_req.update_time_start {
        sql_vals.push(Value::from(update_time_start));
        where_fragments.push(format!("update_time >= ${}", sql_vals.len()));
    }
    if let Some(update_time_end) = match_req.update_time_end {
        sql_vals.push(Value::from(update_time_end));
        where_fragments.push(format!("update_time <= ${}", sql_vals.len()));
    }
    if let Some(desc_sort_by_create) = match_req.desc_sort_by_create {
        order_fragments.push(format!("create_time {}", if desc_sort_by_create { "DESC" } else { "ASC" }));
    }
    if let Some(desc_sort_by_update) = match_req.desc_sort_by_update {
        order_fragments.push(format!("update_time {}", if desc_sort_by_update { "DESC" } else { "ASC" }));
    }

    sql_vals.push(Value::from(match_req.page_size));
    sql_vals.push(Value::from((match_req.page_number - 1) * match_req.page_size as u32));
    let page_fragments = format!("LIMIT ${} OFFSET ${}", sql_vals.len() - 1, sql_vals.len());

    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (conn, table_name) = kv_pg_initializer::init_table_and_conn(bs_inst, ctx, true).await?;
    let result = conn
        .query_all(
            &format!(
                r#"SELECT k, v{} AS v, info, owner, own_paths, scope_level, create_time, update_time, count(*) OVER() AS total
FROM {}
WHERE 
    {}
{}
{}"#,
                if let Some(extract) = match_req.extract {
                    format!("->'{extract}'")
                } else {
                    "".to_string()
                },
                table_name,
                where_fragments.join(" AND "),
                if order_fragments.is_empty() {
                    "".to_string()
                } else {
                    format!("ORDER BY {}", order_fragments.join(", "))
                },
                page_fragments
            ),
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
            Ok(KvItemSummaryResp {
                key: item.try_get("", "k")?,
                value: item.try_get("", "v")?,
                info: item.try_get("", "info")?,
                owner: item.try_get("", "owner")?,
                own_paths: item.try_get("", "own_paths")?,
                scope_level: item.try_get("", "scope_level")?,
                create_time: item.try_get("", "create_time")?,
                update_time: item.try_get("", "update_time")?,
            })
        })
        .filter(|item| {
            item.is_ok()
                && rbum_scope_helper::check_scope(
                    &item.as_ref().expect("invalid result").own_paths,
                    Some(item.as_ref().expect("invalid result").scope_level),
                    &RbumBasicFilterReq {
                        ignore_scope: false,
                        ..Default::default()
                    },
                    &ctx.own_paths,
                )
        })
        .collect::<TardisResult<Vec<_>>>()?;
    Ok(TardisPage {
        page_size: match_req.page_size as u64,
        page_number: match_req.page_number as u64,
        total_size: total_size as u64,
        records: result,
    })
}

pub async fn delete_item(key: String, _funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = kv_pg_initializer::init_table_and_conn(bs_inst, ctx, true).await?;
    conn.begin().await?;
    conn.execute_one(&format!("DELETE FROM {table_name} WHERE k = $1"), vec![Value::from(key)]).await?;
    conn.commit().await?;
    Ok(())
}

pub async fn page_tags(
    key_prefix: String,
    page_number: u32,
    page_size: u16,
    desc_sort_by_create: Option<bool>,
    desc_sort_by_update: Option<bool>,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<TardisPage<KvTagFindResp>> {
    let key_prefix = format!("{}{}", kv_constants::KEY_PREFIX_BY_TAG, key_prefix);
    self::match_items(
        KvItemMatchReq {
            key_prefix,
            page_number,
            page_size,
            desc_sort_by_create,
            desc_sort_by_update,
            ..Default::default()
        },
        funs,
        ctx,
        &inst,
    )
    .await
    .and_then(|items| {
        Ok(TardisPage {
            page_size: items.page_size,
            page_number: items.page_number,
            total_size: items.total_size,
            records: items
                .records
                .into_iter()
                .map(|item| {
                    Ok(KvTagFindResp {
                        key: item.key.strip_prefix(kv_constants::KEY_PREFIX_BY_TAG).unwrap_or("").to_string(),
                        items: TardisFuns::json.json_to_obj(item.value)?,
                        create_time: item.create_time,
                        update_time: item.update_time,
                    })
                })
                .collect::<TardisResult<Vec<_>>>()?,
        })
    })
}
