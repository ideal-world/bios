use bios_basic::spi::spi_funs::SpiBsInstExtractor;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    db::{reldb_client::TardisRelDBClient, sea_orm::Value},
    web::web_resp::TardisPage,
    TardisFunsInst,
};

use crate::dto::log_item_dto::{LogItemAddReq, LogItemFindReq, LogItemFindResp};

use super::log_pg_initializer;

pub async fn add(add_req: &mut LogItemAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let mut params = vec![
        Value::from(add_req.key.as_ref().unwrap_or(&"".into()).to_string()),
        Value::from(add_req.op.as_ref().unwrap_or(&"".to_string()).as_str()),
        Value::from(add_req.content.as_str()),
        Value::from(add_req.rel_key.as_ref().unwrap_or(&"".into()).to_string()),
    ];
    if let Some(ts) = add_req.ts {
        params.push(Value::from(ts));
    }

    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = log_pg_initializer::init_table_and_conn(bs_inst, &add_req.tag, ctx, true).await?;
    conn.begin().await?;
    conn.execute_one(
        &format!(
            r#"INSERT INTO {table_name} 
    (key, op, content, rel_key{})
VALUES
    ($1, $2, $3, $4{})
	"#,
            if add_req.ts.is_some() { ", ts" } else { "" },
            if add_req.ts.is_some() { ", $5" } else { "" },
        ),
        params,
    )
    .await?;
    conn.commit().await?;
    Ok(())
}

pub async fn find(find_req: &mut LogItemFindReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<TardisPage<LogItemFindResp>> {
    let mut where_fragments: Vec<String> = Vec::new();
    let mut sql_vals: Vec<Value> = vec![];

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
    if let Some(ts_start) = find_req.ts_start {
        sql_vals.push(Value::from(ts_start));
        where_fragments.push(format!("ts >= ${}", sql_vals.len()));
    }
    if let Some(ts_end) = find_req.ts_end {
        sql_vals.push(Value::from(ts_end));
        where_fragments.push(format!("ts <= ${}", sql_vals.len()));
    }
    if where_fragments.is_empty() {
        where_fragments.push("1 = 1".to_string());
    }

    sql_vals.push(Value::from(find_req.page_size));
    sql_vals.push(Value::from((find_req.page_number - 1) * find_req.page_size as u32));
    let page_fragments = format!("LIMIT ${} OFFSET ${}", sql_vals.len() - 1, sql_vals.len());

    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (conn, table_name) = log_pg_initializer::init_table_and_conn(bs_inst, &find_req.tag, ctx, false).await?;
    let result = conn
        .query_all(
            format!(
                r#"SELECT ts, key, op, content, rel_key, count(*) OVER() AS total
FROM {table_name}
WHERE 
    {}
ORDER BY ts DESC
{}"#,
                where_fragments.join(" AND "),
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
            LogItemFindResp {
                ts: item.try_get("", "ts").unwrap(),
                key: item.try_get("", "key").unwrap(),
                op: item.try_get("", "op").unwrap(),
                content: item.try_get("", "content").unwrap(),
                rel_key: item.try_get("", "rel_key").unwrap(),
            }
        })
        .collect::<Vec<LogItemFindResp>>();

    Ok(TardisPage {
        page_size: find_req.page_size as u64,
        page_number: find_req.page_number as u64,
        total_size: total_size as u64,
        records: result,
    })
}
