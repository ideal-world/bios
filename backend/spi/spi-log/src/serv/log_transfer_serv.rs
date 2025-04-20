use bios_basic::process::task_processor::TaskProcessor;
use itertools::Itertools;
use std::collections::HashMap;
use std::result;

use bios_basic::spi::spi_funs::{SpiBsInst, SpiBsInstExtractor};
use tardis::basic::result::TardisResult;
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm::Value;
use tardis::futures::future::join_all;
use tardis::TardisFunsInst;
use tardis::{basic::dto::TardisContext, db::reldb_client::TardisRelDBClient};

use crate::dto::log_item_dto::{LogExportAggResp, LogExportV2AggResp, LogImportAggReq, LogImportV2AggReq, LogItemAddReq, LogItemAddV2Req};
use crate::log_config::LogConfig;
use crate::{
    dto::log_item_dto::{LogExportDataReq, LogExportDataResp, LogImportDataReq},
    log_initializer,
};

use super::pg;
use super::pgv2;

pub async fn export_data(req: &LogExportDataReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<LogExportDataResp> {
    let inst = funs.init(None, ctx, true, log_initializer::init_fun).await?;
    let tag_data = export_log_v1(req.tags.clone(), req.start_time, req.end_time, funs, ctx, &inst).await?;
    let tag_v2_data = export_log_v2(req.tags_v2.clone(), req.start_time, req.end_time, funs, ctx, &inst).await?;
    Ok(LogExportDataResp { tag_data, tag_v2_data })
}

async fn export_log_v1(
    tags: Vec<String>,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    _funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<HashMap<String, Vec<LogExportAggResp>>> {
    let mut tag_data = HashMap::new();
    let bs_inst = inst.inst::<TardisRelDBClient>();
    // let (mut conn, table_name) = log_pg_initializer::init_table_and_conn(bs_inst, &add_req.tag, ctx, true).await?;
    for tag in &tags {
        let (conn, table_name) = pg::log_pg_initializer::init_table_and_conn(bs_inst, tag, ctx, false).await?;
        let result = conn
            .query_all(
                &format!("SELECT key, kind, content, data_source, owner, own_paths, ext, op, rel_key, ts, id FROM {table_name} WHERE (ts > $1 or ts <= $2) order by ts desc"),
                vec![Value::from(start_time), Value::from(end_time)],
            )
            .await?;
        let result = result
            .into_iter()
            .map(|item| {
                Ok(LogExportAggResp {
                    kind: item.try_get("", "kind")?,
                    key: item.try_get("", "key")?,
                    content: item.try_get("", "content")?,
                    data_source: item.try_get("", "data_source").unwrap_or_default(),
                    owner: item.try_get("", "owner")?,
                    own_paths: item.try_get("", "own_paths")?,
                    ext: item.try_get("", "ext")?,
                    tag: tag.to_string(),
                    op: item.try_get("", "op")?,
                    rel_key: item.try_get("", "rel_key")?,
                    id: item.try_get("", "id")?,
                    ts: item.try_get("", "ts")?,
                })
            })
            .collect::<TardisResult<Vec<LogExportAggResp>>>()?;
        tag_data.insert(tag.to_string(), result);
    }
    Ok(tag_data)
}

async fn export_log_v2(
    tags: Vec<String>,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    _funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<HashMap<String, Vec<LogExportV2AggResp>>> {
    let mut tag_data = HashMap::new();
    let bs_inst = inst.inst::<TardisRelDBClient>();
    // let (mut conn, table_name) = log_pg_initializer::init_table_and_conn(bs_inst, &add_req.tag, ctx, true).await?;
    for tag in &tags {
        let (conn, table_name) = pgv2::log_pg_initializer::init_table_and_conn(bs_inst, tag, ctx, false).await?;
        let result = conn
            .query_all(
                &format!("SELECT key, kind, content, data_source, owner, owner_name, own_paths, ext, tag, op, rel_key, ts, idempotent_id, disable, msg, push FROM {table_name} WHERE (ts > $1 or ts <= $2) order by ts desc"),
                vec![Value::from(start_time), Value::from(end_time)],
            )
            .await?;
        let result = result
            .into_iter()
            .map(|item| {
                Ok(LogExportV2AggResp {
                    kind: item.try_get("", "kind")?,
                    key: item.try_get("", "key")?,
                    content: item.try_get("", "content")?,
                    data_source: item.try_get("", "data_source").unwrap_or_default(),
                    owner: item.try_get("", "owner")?,
                    owner_name: item.try_get("", "owner_name")?,
                    own_paths: item.try_get("", "own_paths")?,
                    ext: item.try_get("", "ext")?,
                    tag: tag.to_string(),
                    op: item.try_get("", "op")?,
                    rel_key: item.try_get("", "rel_key")?,
                    ts: item.try_get("", "ts")?,
                    idempotent_id: item.try_get("", "idempotent_id")?,
                    disable: item.try_get("", "disable")?,
                    msg: item.try_get("", "msg")?,
                    push: item.try_get("", "push")?,
                })
            })
            .collect::<TardisResult<Vec<LogExportV2AggResp>>>()?;
        tag_data.insert(tag.to_string(), result);
    }
    Ok(tag_data)
}

pub async fn import_data(import_req: &LogImportDataReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
    let inst = funs.init(None, ctx, true, log_initializer::init_fun).await?;
    let ctx_cloned = ctx.clone();
    let init_cloned = inst.clone();
    let data_source = import_req.data_source.clone();
    let tag_data = import_req.tag_data.clone();
    let tag_v2_data = import_req.tag_v2_data.clone();
    TaskProcessor::execute_task_with_ctx(
        &funs.conf::<LogConfig>().cache_key_async_task_status,
        {
            move |_task_id| async move {
                let funs = crate::get_tardis_inst();
                let _ = import_log_v1(&data_source, tag_data.clone(), &funs, &ctx_cloned, &init_cloned).await?;
                let _ = import_log_v2(&data_source, tag_v2_data.clone(), &funs, &ctx_cloned, &init_cloned).await?;
                Ok(())
            }
        },
        &funs.cache(),
        "spi-log".to_string(),
        Some(vec![format!("account/{}", ctx.owner)]),
        ctx,
    )
    .await?;
    Ok(true)
}

pub async fn import_log_v1(data_source: &str, tag_data: HashMap<String, Vec<LogImportAggReq>>, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<bool> {
    for (tag, tag_data) in &tag_data {
        let max_size = tag_data.len();
        let mut page = 0;
        let page_size = 2000;
        loop {
            let current_result = &tag_data[((page * page_size).min(max_size))..(((page + 1) * page_size).min(max_size))];
            if current_result.is_empty() {
                break;
            }
            join_all(
                current_result
                    .iter()
                    .map(|row| async move {
                        pg::log_pg_item_serv::add(
                            &mut LogItemAddReq {
                                tag: tag.to_string(),
                                kind: Some(row.kind.clone().into()),
                                key: Some(row.key.clone().into()),
                                content: row.content.clone(),
                                data_source: Some(data_source.to_string()),
                                owner: Some(row.owner.clone()),
                                own_paths: Some(row.own_paths.clone()),
                                ext: Some(row.ext.clone()),
                                op: Some(row.op.clone()),
                                rel_key: Some(row.rel_key.clone().into()),
                                id: Some(row.id.clone()),
                                ts: Some(row.ts),
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
    }
    Ok(true)
}

pub async fn import_log_v2(
    data_source: &str,
    tag_data: HashMap<String, Vec<LogImportV2AggReq>>,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<bool> {
    for (tag, tag_data) in &tag_data {
        let max_size = tag_data.len();
        let mut page = 0;
        let page_size = 2000;
        loop {
            let current_result = &tag_data[((page * page_size).min(max_size))..(((page + 1) * page_size).min(max_size))];
            if current_result.is_empty() {
                break;
            }
            join_all(
                current_result
                    .iter()
                    .map(|row| async move {
                        pgv2::log_pg_item_serv::addv2(
                            &mut LogItemAddV2Req {
                                tag: tag.to_string(),
                                kind: Some(row.kind.clone().into()),
                                key: Some(row.key.clone().into()),
                                content: row.content.clone(),
                                data_source: Some(data_source.to_string()),
                                owner: Some(row.owner.clone()),
                                own_paths: Some(row.own_paths.clone()),
                                ext: Some(row.ext.clone()),
                                op: Some(row.op.clone()),
                                rel_key: Some(row.rel_key.clone().into()),
                                ts: Some(row.ts),
                                idempotent_id: Some(row.idempotent_id.clone()),
                                owner_name: Some(row.owner_name.clone()),
                                disable: Some(row.disable),
                                push: row.push,
                                msg: Some(row.msg.clone()),
                                ignore_push: Some(true),
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
    }
    Ok(true)
}
