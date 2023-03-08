use bios_basic::spi::spi_funs::SpiBsInstExtractor;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    db::{reldb_client::TardisRelDBClient, sea_orm::Value},
    web::web_resp::TardisPage,
    TardisFunsInst,
};

use crate::dto::kv_item_dto::{KvItemAddOrModifyReq, KvItemDetailResp, KvItemSummaryResp};

use super::kv_pg_initializer;

pub async fn add_or_modify_item(add_or_modify_req: &KvItemAddOrModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let params = vec![
        Value::from(add_or_modify_req.key.to_string()),
        Value::from(add_or_modify_req.value.clone()),
        Value::from(add_or_modify_req.info.as_ref().unwrap_or(&"".to_string()).as_str()),
    ];
    let mut update_opt_fragments: Vec<&str> = Vec::new();
    update_opt_fragments.push("v = $2");
    if add_or_modify_req.info.is_some() {
        update_opt_fragments.push("info = $3");
    }
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = kv_pg_initializer::init_table_and_conn(bs_inst, ctx, true).await?;
    conn.begin().await?;
    conn.execute_one(
        &format!(
            r#"INSERT INTO {} 
    (k, v, info)
VALUES
    ($1, $2, $3)
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

pub async fn get_item(key: String, extract: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<KvItemDetailResp>> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (conn, table_name) = kv_pg_initializer::init_table_and_conn(bs_inst, ctx, true).await?;
    let result = conn
        .get_dto_by_sql(
            &format!(
                r#"SELECT k AS key, v{} AS value, info, create_time, update_time
FROM {}
WHERE 
    k = $1"#,
                if let Some(extract) = extract { format!("->'{extract}'") } else { "".to_string() },
                table_name,
            ),
            vec![Value::from(key)],
        )
        .await?;
    Ok(result)
}

pub async fn find_items(keys: Vec<String>, extract: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<KvItemSummaryResp>> {
    let mut sql_vals: Vec<Value> = vec![];
    let place_holder = keys
        .iter()
        .map(|key| {
            sql_vals.push(Value::from(key.to_string()));
            format!("${}", sql_vals.len())
        })
        .collect::<Vec<String>>()
        .join(",");
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (conn, table_name) = kv_pg_initializer::init_table_and_conn(bs_inst, ctx, true).await?;
    let result = conn
        .find_dtos_by_sql(
            &format!(
                r#"SELECT k AS key, v{} AS value, info, create_time, update_time
FROM {}
WHERE 
    k IN ({})"#,
                if let Some(extract) = extract { format!("->'{extract}'") } else { "".to_string() },
                table_name,
                place_holder
            ),
            sql_vals,
        )
        .await?;
    Ok(result)
}

pub async fn match_items(
    key_prefix: String,
    extract: Option<String>,
    page_number: u32,
    page_size: u16,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<TardisPage<KvItemSummaryResp>> {
    let mut sql_vals: Vec<Value> = vec![];
    sql_vals.push(Value::from(format!("{key_prefix}%")));
    sql_vals.push(Value::from(page_size));
    sql_vals.push(Value::from((page_number - 1) * page_size as u32));

    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (conn, table_name) = kv_pg_initializer::init_table_and_conn(bs_inst, ctx, true).await?;
    let result = conn
        .query_all(
            &format!(
                r#"SELECT k, v{} AS v, info, create_time, update_time, count(*) OVER() AS total
FROM {}
WHERE 
    k LIKE $1
LIMIT $2 OFFSET $3"#,
                if let Some(extract) = extract { format!("->'{extract}'") } else { "".to_string() },
                table_name,
            ),
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
            KvItemSummaryResp {
                key: item.try_get("", "k").unwrap(),
                value: item.try_get("", "v").unwrap(),
                info: item.try_get("", "info").unwrap(),
                create_time: item.try_get("", "create_time").unwrap(),
                update_time: item.try_get("", "update_time").unwrap(),
            }
        })
        .collect();
    Ok(TardisPage {
        page_size: page_size as u64,
        page_number: page_number as u64,
        total_size: total_size as u64,
        records: result,
    })
}

pub async fn delete_item(key: String, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = kv_pg_initializer::init_table_and_conn(bs_inst, ctx, true).await?;
    conn.begin().await?;
    conn.execute_one(&format!("DELETE FROM {table_name} WHERE k = $1"), vec![Value::from(key)]).await?;
    conn.commit().await?;
    Ok(())
}
