use std::collections::HashMap;

use bios_basic::spi::{
    spi_funs::SpiBsInst,
    spi_initializer::common_pg::{self, package_table_name},
};

use bios_sdk_invoke::clients::schedule_client::{AddOrModifySyncTaskReq, ScheduleClient};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    db::{
        reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
        sea_orm::Value,
    },
    web::web_resp::TardisPage,
    TardisFuns, TardisFunsInst,
};

use crate::{
    dto::stats_conf_dto::{StatsConfFactAddReq, StatsConfFactColInfoResp, StatsConfFactInfoResp, StatsConfFactModifyReq},
    serv::stats_valid_serv,
    stats_config::StatsConfig,
    stats_constants::SYNC_FACT_TASK_CODE,
};

use super::{stats_pg_conf_fact_col_serv, stats_pg_initializer, stats_pg_sync_serv};

pub async fn online(fact_conf_key: &str, conn: &TardisRelDBlConnection, ctx: &TardisContext) -> TardisResult<bool> {
    common_pg::check_table_exit(&format!("stats_inst_fact_{fact_conf_key}"), conn, ctx).await
}

pub(crate) async fn add(add_req: &StatsConfFactAddReq, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = stats_pg_initializer::init_conf_fact_table_and_conn(bs_inst, ctx, true).await?;
    conn.begin().await?;
    if conn.count_by_sql(&format!("SELECT 1 FROM {table_name} WHERE key = $1"), vec![Value::from(&add_req.key)]).await? != 0 {
        return Err(funs.err().conflict(
            "fact_conf",
            "add",
            "The fact config already exists, please delete it and then add it.",
            "409-spi-stats-fact-conf-exist",
        ));
    }
    if let Some(sync_sql) = &add_req.sync_sql {
        if !stats_valid_serv::validate_select_sql(sync_sql) {
            return Err(funs.err().conflict("fact_conf", "add", "The sync_sql is not a valid sql.", "409-spi-stats-fact-conf-sync-sql-not-valid"));
        }
    }
    let params = vec![
        Value::from(add_req.key.to_string()),
        Value::from(add_req.show_name.clone()),
        Value::from(add_req.query_limit),
        Value::from(add_req.remark.as_ref().unwrap_or(&"".to_string()).as_str()),
        Value::from(add_req.redirect_path.clone()),
        Value::from(add_req.is_online.unwrap_or_default()),
        Value::from(add_req.rel_cert_id.as_ref().unwrap_or(&"".to_string()).as_str()),
        Value::from(add_req.sync_sql.as_ref().unwrap_or(&"".to_string()).as_str()),
        Value::from(add_req.sync_cron.as_ref().unwrap_or(&"".to_string()).as_str()),
        Value::from(add_req.is_sync.unwrap_or_default()),
    ];

    conn.execute_one(
        &format!(
            r#"INSERT INTO {table_name}
(key, show_name, query_limit, remark, redirect_path, is_online, rel_cert_id, sync_sql, sync_cron, is_sync)
VALUES
($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
"#,
        ),
        params,
    )
    .await?;
    conn.commit().await?;
    //handle sync fact task
    let mut callback_headers: HashMap<String, String> = HashMap::new();
    callback_headers.insert("Tardis-Context".to_string(), TardisFuns::crypto.base64.encode(TardisFuns::json.obj_to_string(&ctx)?));
    ScheduleClient::add_or_modify_sync_task(
        AddOrModifySyncTaskReq {
            code: format!("{}_{}", SYNC_FACT_TASK_CODE, add_req.key),
            cron: add_req.sync_cron.clone().unwrap_or("".to_string()),
            callback_url: format!("{}/ci/fact/{}/sync", funs.conf::<StatsConfig>().base_url, add_req.key),
            callback_method: "PUT".to_string(),
            callback_body: None,
            enable: add_req.is_sync.unwrap_or_default(),
            callback_headers,
        },
        funs,
        ctx,
    )
    .await?;

    Ok(())
}

pub(crate) async fn modify(fact_conf_key: &str, modify_req: &StatsConfFactModifyReq, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = stats_pg_initializer::init_conf_fact_table_and_conn(bs_inst, ctx, true).await?;
    conn.begin().await?;
    let Some(fact_conf) = get(fact_conf_key, &conn, ctx).await? else {
        return Err(funs.err().not_found("starsys_stats_conf_fact", "find", "fact conf not found", "404-fact-conf-not-found"));
    };
    let mut sql_sets = vec![];
    let mut params = vec![Value::from(fact_conf_key.to_string())];
    // todo cancel online check
    // if online(fact_conf_key, &conn, ctx).await? {
    //     if modify_req.is_online.is_none() {
    //         return Err(funs.err().conflict(
    //             "fact_conf",
    //             "modify",
    //             "The fact instance table already exists, please delete it and then modify it.",
    //             "409-spi-stats-fact-inst-exist",
    //         ));
    //     }
    // }
    if let Some(show_name) = &modify_req.show_name {
        sql_sets.push(format!("show_name = ${}", params.len() + 1));
        params.push(Value::from(show_name.to_string()));
    }
    if let Some(query_limit) = modify_req.query_limit {
        sql_sets.push(format!("query_limit = ${}", params.len() + 1));
        params.push(Value::from(query_limit));
    }
    if let Some(remark) = &modify_req.remark {
        sql_sets.push(format!("remark = ${}", params.len() + 1));
        params.push(Value::from(remark.to_string()));
    }
    if let Some(redirect_path) = &modify_req.redirect_path {
        sql_sets.push(format!("redirect_path = ${}", params.len() + 1));
        params.push(Value::from(redirect_path));
    }
    if let Some(rel_cert_id) = &modify_req.rel_cert_id {
        sql_sets.push(format!("rel_cert_id = ${}", params.len() + 1));
        params.push(Value::from(rel_cert_id.to_string()));
    }
    if let Some(sync_sql) = &modify_req.sync_sql {
        sql_sets.push(format!("sync_sql = ${}", params.len() + 1));
        params.push(Value::from(sync_sql.to_string()));
    }
    if let Some(sync_cron) = &modify_req.sync_cron {
        sql_sets.push(format!("sync_cron = ${}", params.len() + 1));
        params.push(Value::from(sync_cron.to_string()));
    }
    if let Some(is_sync) = &modify_req.is_sync {
        sql_sets.push(format!("is_sync = ${}", params.len() + 1));
        params.push(Value::from(*is_sync));
    }

    if let Some(is_online) = &modify_req.is_online {
        sql_sets.push(format!("is_online = ${}", params.len() + 1));
        params.push(Value::from(*is_online));
    }
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
    //handle sync fact task
    let mut callback_headers: HashMap<String, String> = HashMap::new();
    callback_headers.insert("Tardis-Context".to_string(), TardisFuns::crypto.base64.encode(TardisFuns::json.obj_to_string(&ctx)?));
    ScheduleClient::add_or_modify_sync_task(
        AddOrModifySyncTaskReq {
            code: format!("{}_{}", SYNC_FACT_TASK_CODE, fact_conf_key),
            enable: fact_conf.is_sync.unwrap_or_default(),
            cron: modify_req.sync_cron.clone().unwrap_or(fact_conf.sync_cron.clone().unwrap_or("".to_string())),
            callback_url: format!("{}/ci/fact/{}/sync", funs.conf::<StatsConfig>().base_url, fact_conf_key),
            callback_method: "PUT".to_string(),
            callback_body: None,
            callback_headers,
        },
        funs,
        ctx,
    )
    .await?;
    Ok(())
}

pub(crate) async fn delete(fact_conf_key: &str, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = stats_pg_initializer::init_conf_fact_table_and_conn(bs_inst, ctx, true).await?;
    let (_, fact_col_table_name) = stats_pg_initializer::init_conf_fact_col_table_and_conn(bs_inst, ctx, true).await?;
    conn.begin().await?;
    conn.execute_one(&format!("DELETE FROM {table_name} WHERE key = $1"), vec![Value::from(fact_conf_key)]).await?;
    // delete related fact column config
    conn.execute_one(&format!("DELETE FROM {fact_col_table_name} WHERE rel_conf_fact_key = $1"), vec![Value::from(fact_conf_key)]).await?;
    // The lazy loading mechanism may cause the ``<schema>.starsys_stats_inst_fact_<key>_col`` table to not be created
    if common_pg::check_table_exit(&format!("stats_inst_fact_{fact_conf_key}_col"), &conn, ctx).await? {
        conn.execute_one(
            &format!("DELETE FROM {} WHERE rel_conf_fact_key = $1", package_table_name("stats_conf_fact_col", ctx)),
            vec![Value::from(fact_conf_key)],
        )
        .await?;
    }
    if online(fact_conf_key, &conn, ctx).await? {
        conn.execute_one(&format!("DROP TABLE {}{fact_conf_key}", package_table_name("stats_inst_fact_", ctx)), vec![]).await?;
        conn.execute_one(&format!("DROP TABLE {}{fact_conf_key}_del", package_table_name("stats_inst_fact_", ctx)), vec![]).await?;
    }
    ScheduleClient::delete_sync_task(&format!("{}_{}", SYNC_FACT_TASK_CODE, fact_conf_key), funs, ctx).await?;
    conn.commit().await?;
    Ok(())
}

pub(in crate::serv::pg) async fn get(fact_conf_key: &str, conn: &TardisRelDBlConnection, ctx: &TardisContext) -> TardisResult<Option<StatsConfFactInfoResp>> {
    do_paginate(Some(vec![fact_conf_key.to_string()]), None, None, None, 1, 1, None, None, conn, ctx).await.map(|page| page.records.into_iter().next())
}

pub(crate) async fn paginate(
    fact_conf_keys: Option<Vec<String>>,
    show_name: Option<String>,
    dim_rel_conf_dim_keys: Option<Vec<String>>,
    is_online: Option<bool>,
    page_number: u32,
    page_size: u32,
    desc_by_create: Option<bool>,
    desc_by_update: Option<bool>,
    _funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<TardisPage<StatsConfFactInfoResp>> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (conn, _) = stats_pg_initializer::init_conf_fact_table_and_conn(bs_inst, ctx, true).await?;

    do_paginate(
        fact_conf_keys,
        show_name,
        dim_rel_conf_dim_keys,
        is_online,
        page_number,
        page_size,
        desc_by_create,
        desc_by_update,
        &conn,
        ctx,
    )
    .await
}

async fn do_paginate(
    fact_conf_keys: Option<Vec<String>>,
    show_name: Option<String>,
    dim_rel_conf_dim_keys: Option<Vec<String>>,
    is_online: Option<bool>,
    page_number: u32,
    page_size: u32,
    desc_by_create: Option<bool>,
    desc_by_update: Option<bool>,
    conn: &TardisRelDBlConnection,
    ctx: &TardisContext,
) -> TardisResult<TardisPage<StatsConfFactInfoResp>> {
    let table_name = package_table_name("stats_conf_fact", ctx);
    let table_col_name = package_table_name("stats_conf_fact_col", ctx);
    let mut sql_where = vec!["1 = 1".to_string()];
    let mut sql_order = vec![];
    let mut sql_left = "".to_string();
    let mut params: Vec<Value> = vec![Value::from(page_size), Value::from((page_number - 1) * page_size)];
    if let Some(fact_conf_keys) = &fact_conf_keys {
        sql_where.push(format!(
            "fact.key IN ({})",
            (0..fact_conf_keys.len()).map(|idx| format!("${}", params.len() + idx + 1)).collect::<Vec<String>>().join(",")
        ));
        for fact_conf_key in fact_conf_keys {
            params.push(Value::from(fact_conf_key.to_string()));
        }
    }
    if let Some(show_name) = &show_name {
        sql_where.push(format!("fact.show_name LIKE ${}", params.len() + 1));
        params.push(Value::from(format!("%{show_name}%")));
    }
    if let Some(is_online) = &is_online {
        sql_where.push(format!("fact.is_online = ${}", params.len() + 1));
        params.push(Value::from(*is_online));
    }
    if let Some(dim_rel_conf_dim_keys) = &dim_rel_conf_dim_keys {
        if !dim_rel_conf_dim_keys.is_empty() {
            sql_left = format!(
                r#" LEFT JOIN (SELECT rel_conf_fact_key,COUNT(rel_conf_fact_key) FROM {table_col_name}  WHERE dim_rel_conf_dim_key IN ({}) GROUP BY rel_conf_fact_key HAVING COUNT(rel_conf_fact_key) = {}) AS fact_col ON fact.key = fact_col.rel_conf_fact_key"#,
                (0..dim_rel_conf_dim_keys.len()).map(|idx| format!("${}", params.len() + idx + 1)).collect::<Vec<String>>().join(","),
                dim_rel_conf_dim_keys.len()
            );
            for dim_rel_conf_dim_key in dim_rel_conf_dim_keys {
                params.push(Value::from(dim_rel_conf_dim_key.to_string()));
            }
            sql_where.push("fact_col.rel_conf_fact_key IS NOT NULL".to_string());
        }
    }
    if let Some(desc_by_create) = desc_by_create {
        sql_order.push(format!("fact.create_time {}", if desc_by_create { "DESC" } else { "ASC" }));
    }
    if let Some(desc_by_update) = desc_by_update {
        sql_order.push(format!("fact.update_time {}", if desc_by_update { "DESC" } else { "ASC" }));
    }

    let result = conn
        .query_all(
            &format!(
                r#"SELECT t.*, count(*) OVER () AS total FROM (
SELECT distinct fact.key as key, fact.show_name as show_name, fact.query_limit as query_limit, fact.remark as remark, fact.redirect_path as redirect_path, fact.is_online as is_online, fact.rel_cert_id as rel_cert_id, fact.sync_sql as sync_sql, fact.sync_cron as sync_cron, fact.is_sync as is_sync, fact.create_time as create_time, fact.update_time as update_time
FROM {table_name} as fact
{}
WHERE 
    {}
    {}
    ) as t
LIMIT $1 OFFSET $2
"#,
                sql_left,
                sql_where.join(" AND "),
                if sql_order.is_empty() {
                    "".to_string()
                } else {
                    format!("ORDER BY {}", sql_order.join(","))
                }
            ),
            params,
        )
        .await?;

    let mut total_size: i64 = 0;
    let mut final_result = vec![];
    for item in result {
        if total_size == 0 {
            total_size = item.try_get("", "total")?;
        }
        final_result.push(StatsConfFactInfoResp {
            key: item.try_get("", "key")?,
            show_name: item.try_get("", "show_name")?,
            query_limit: item.try_get("", "query_limit")?,
            remark: item.try_get("", "remark")?,
            create_time: item.try_get("", "create_time")?,
            update_time: item.try_get("", "update_time")?,
            online: online(&item.try_get::<String>("", "key")?, conn, ctx).await?,
            is_online: item.try_get("", "is_online")?,
            redirect_path: item.try_get("", "redirect_path")?,
            rel_cert_id: item.try_get("", "rel_cert_id")?,
            sync_sql: item.try_get("", "sync_sql")?,
            sync_cron: item.try_get("", "sync_cron")?,
            is_sync: item.try_get("", "is_sync")?,
        });
    }
    Ok(TardisPage {
        page_size: page_size as u64,
        page_number: page_number as u64,
        total_size: total_size as u64,
        records: final_result,
    })
}

pub(crate) async fn find(
    fact_conf_keys: Option<Vec<String>>,
    show_name: Option<String>,
    dim_rel_conf_dim_keys: Option<Vec<String>>,
    is_online: Option<bool>,
    desc_by_create: Option<bool>,
    desc_by_update: Option<bool>,
    _funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<Vec<StatsConfFactInfoResp>> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (conn, _) = stats_pg_initializer::init_conf_fact_table_and_conn(bs_inst, ctx, true).await?;

    do_find(fact_conf_keys, show_name, dim_rel_conf_dim_keys, is_online, desc_by_create, desc_by_update, &conn, ctx).await
}

async fn do_find(
    fact_conf_keys: Option<Vec<String>>,
    show_name: Option<String>,
    dim_rel_conf_dim_keys: Option<Vec<String>>,
    is_online: Option<bool>,
    desc_by_create: Option<bool>,
    desc_by_update: Option<bool>,
    conn: &TardisRelDBlConnection,
    ctx: &TardisContext,
) -> TardisResult<Vec<StatsConfFactInfoResp>> {
    let table_name = package_table_name("stats_conf_fact", ctx);
    let table_col_name = package_table_name("stats_conf_fact_col", ctx);
    let mut sql_where = vec!["1 = 1".to_string()];
    let mut sql_order = vec![];
    let mut sql_left = "".to_string();
    let mut params: Vec<Value> = vec![];
    if let Some(fact_conf_keys) = &fact_conf_keys {
        sql_where.push(format!(
            "fact.key IN ({})",
            (0..fact_conf_keys.len()).map(|idx| format!("${}", params.len() + idx + 1)).collect::<Vec<String>>().join(",")
        ));
        for fact_conf_key in fact_conf_keys {
            params.push(Value::from(fact_conf_key.to_string()));
        }
    }
    if let Some(show_name) = &show_name {
        sql_where.push(format!("fact.show_name LIKE ${}", params.len() + 1));
        params.push(Value::from(format!("%{show_name}%")));
    }
    if let Some(is_online) = &is_online {
        sql_where.push(format!("fact.is_online = ${}", params.len() + 1));
        params.push(Value::from(*is_online));
    }
    if let Some(dim_rel_conf_dim_keys) = &dim_rel_conf_dim_keys {
        if !dim_rel_conf_dim_keys.is_empty() {
            sql_left = format!(
                r#" LEFT JOIN (SELECT rel_conf_fact_key,COUNT(rel_conf_fact_key) FROM {table_col_name}  WHERE dim_rel_conf_dim_key IN ({}) GROUP BY rel_conf_fact_key HAVING COUNT(rel_conf_fact_key) = {}) AS fact_col ON fact.key = fact_col.rel_conf_fact_key"#,
                (0..dim_rel_conf_dim_keys.len()).map(|idx| format!("${}", params.len() + idx + 1)).collect::<Vec<String>>().join(","),
                dim_rel_conf_dim_keys.len()
            );
            for dim_rel_conf_dim_key in dim_rel_conf_dim_keys {
                params.push(Value::from(dim_rel_conf_dim_key.to_string()));
            }
            sql_where.push("fact_col.rel_conf_fact_key IS NOT NULL".to_string());
        }
    }
    if let Some(desc_by_create) = desc_by_create {
        sql_order.push(format!("fact.create_time {}", if desc_by_create { "DESC" } else { "ASC" }));
    }
    if let Some(desc_by_update) = desc_by_update {
        sql_order.push(format!("fact.update_time {}", if desc_by_update { "DESC" } else { "ASC" }));
    }

    let result = conn
        .query_all(
            &format!(
                r#"SELECT t.* FROM (
SELECT distinct fact.key as key, fact.show_name as show_name, fact.query_limit as query_limit, fact.remark as remark, fact.redirect_path as redirect_path, fact.is_online as is_online, fact.rel_cert_id as rel_cert_id, fact.sync_sql as sync_sql, fact.sync_cron as sync_cron, fact.is_sync as is_sync, fact.create_time as create_time, fact.update_time as update_time
FROM {table_name} as fact
{}
WHERE 
    {}
    {}
    ) as t
"#,
                sql_left,
                sql_where.join(" AND "),
                if sql_order.is_empty() {
                    "".to_string()
                } else {
                    format!("ORDER BY {}", sql_order.join(","))
                }
            ),
            params,
        )
        .await?;

    let mut final_result = vec![];
    for item in result {
        final_result.push(StatsConfFactInfoResp {
            key: item.try_get("", "key")?,
            show_name: item.try_get("", "show_name")?,
            query_limit: item.try_get("", "query_limit")?,
            remark: item.try_get("", "remark")?,
            create_time: item.try_get("", "create_time")?,
            update_time: item.try_get("", "update_time")?,
            online: online(&item.try_get::<String>("", "key")?, conn, ctx).await?,
            is_online: item.try_get("", "is_online")?,
            redirect_path: item.try_get("", "redirect_path")?,
            rel_cert_id: item.try_get("", "rel_cert_id")?,
            sync_sql: item.try_get("", "sync_sql")?,
            sync_cron: item.try_get("", "sync_cron")?,
            is_sync: item.try_get("", "is_sync")?,
        });
    }
    Ok(final_result)
}

/// Create fact instance table.
///
/// The table name is `starsys_stats_inst_fact_<fact key>`
/// The table fields are:
/// - key                   the incoming primary key value
/// - own_paths             data owner, used for data permission control
/// - ct                    create time
/// - [xxx,xxx,xxx,...]     all fields contained in the fact table
///
/// At the same time, a record deletion table will be created.
/// The table name is `starsys_stats_inst_fact_<fact key>_del`. It contains `key,ct` fields.
///
/// # Examples
/// ```
/// CREATE TABLE spi617070303031.starsys_stats_inst_fact_req (
///  key character varying NOT NULL,
///  own_paths character varying NOT NULL,
///  status character varying NOT NULL,
///  priority integer NOT NULL,
///  tag character varying [] NOT NULL,
///  creator character varying NOT NULL,
///  source character varying NOT NULL,
///  act_hours integer NOT NULL,
///  plan_hours integer NOT NULL,
///  ct timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP
/// )
///
/// CREATE TABLE spi617070303031.starsys_stats_inst_fact_req_del (
///  key character varying NOT NULL,
///  ct timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP
/// )
/// ```
pub(crate) async fn create_inst(fact_conf_key: &str, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<()> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, _) = common_pg::init_conn(bs_inst).await?;
    conn.begin().await?;

    let fact_conf = get(fact_conf_key, &conn, ctx)
        .await?
        .ok_or_else(|| funs.err().not_found("fact_conf", "create_inst", "The fact config does not exist.", "404-spi-stats-fact-conf-not-exist"))?;
    let fact_col_conf = stats_pg_conf_fact_col_serv::find_by_fact_conf_key(&fact_conf.key, funs, ctx, inst).await?;
    if fact_col_conf.is_empty() {
        return Err(funs.err().not_found(
            "fact_col_conf",
            "create_inst",
            "The fact column config does not exist.",
            "404-spi-stats-fact-col-conf-not-exist",
        ));
    }

    if online(fact_conf_key, &conn, ctx).await? {
        return Err(funs.err().conflict(
            "fact_inst",
            "create_inst",
            "The fact instance table already exists, please delete it and then create it.",
            "409-spi-stats-fact-inst-exist",
        ));
    }
    create_inst_table(&fact_conf, &fact_col_conf, &conn, funs, ctx, inst).await?;
    conn.commit().await?;
    Ok(())
}

async fn create_inst_table(
    fact_conf: &StatsConfFactInfoResp,
    fact_col_conf_set: &Vec<StatsConfFactColInfoResp>,
    conn: &TardisRelDBlConnection,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<()> {
    // Create fact inst table
    let mut sql = vec![];
    let mut index = vec![];
    sql.push("key character varying NOT NULL".to_string());
    sql.push("own_paths character varying NOT NULL".to_string());
    sql.push("ext jsonb NOT NULL".to_string());
    sql.push("idempotent_id character varying NOT NULL".to_string());
    index.push(("own_paths".to_string(), "btree"));
    for fact_col_conf in fact_col_conf_set {
        let col_sql = stats_pg_conf_fact_col_serv::fact_col_column_sql(fact_col_conf.clone(), &mut index, true, conn, funs, ctx, inst).await?;
        sql.push(col_sql)
    }
    sql.push("ct timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP".to_string());
    index.push(("ct".to_string(), "btree"));
    index.push(("date(timezone('UTC', ct))".to_string(), "btree"));
    index.push(("date_part('hour',timezone('UTC', ct))".to_string(), "btree"));
    index.push(("date_part('day',timezone('UTC', ct))".to_string(), "btree"));
    index.push(("date_part('month',timezone('UTC', ct))".to_string(), "btree"));

    let mut swap_index = vec![];
    for i in &index {
        swap_index.push((&i.0[..], i.1));
    }
    common_pg::init_table(conn, Some(&fact_conf.key), "stats_inst_fact", sql.join(",\r\n").as_str(), swap_index, None, None, ctx).await?;

    // Create fact inst delete status table
    common_pg::init_table(
        conn,
        Some(&format!("{}_del", fact_conf.key)),
        "stats_inst_fact",
        r#"key character varying NOT NULL,
    ct timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP"#,
        vec![],
        Some(vec!["key", "ct"]),
        None,
        ctx,
    )
    .await?;

    Ok(())
}
