use bios_basic::process::task_processor::TaskProcessor;
use bios_basic::spi::spi_initializer::common_pg::{self, package_table_name};
use itertools::Itertools;
use std::collections::HashMap;
use tardis::basic::error::TardisError;

use bios_basic::spi::spi_funs::{SpiBsInst, SpiBsInstExtractor};
use tardis::basic::result::TardisResult;
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm::{FromQueryResult, Value};
use tardis::futures::future::join_all;
use tardis::TardisFunsInst;
use tardis::{basic::dto::TardisContext, db::reldb_client::TardisRelDBClient};

use crate::dto::stats_record_dto::StatsFactRecordsLoadReq;
use crate::dto::stats_transfer_dto::{
    StatsExportAggResp, StatsExportDataReq, StatsExportDataResp, StatsExportDelAggResp, StatsImportAggReq, StatsImportDataReq, StatsImportDelAggReq,
};
use crate::stats_config::StatsConfig;
use crate::stats_initializer;

use super::pg::{stats_pg_conf_fact_serv, stats_pg_record_serv};

pub async fn export_data(req: &StatsExportDataReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<StatsExportDataResp> {
    let inst = funs.init(None, ctx, true, stats_initializer::init_fun).await?;
    let mut fact_conf_data = HashMap::new();
    let mut fact_conf_data_del = HashMap::new();
    let fact_confs = stats_pg_conf_fact_serv::find(None, None, None, None, None, None, funs, ctx, &inst).await?;
    for fact_conf in fact_confs {
        let result = find_fact_record(&fact_conf.key, req.start_time, req.end_time, funs, ctx, &inst).await?;
        let result_del = find_fact_record_del(&fact_conf.key, req.start_time, req.end_time, funs, ctx, &inst).await?;
        fact_conf_data.insert(fact_conf.key.clone(), result);
        fact_conf_data_del.insert(fact_conf.key, result_del);
    }
    Ok(StatsExportDataResp {
        fact_conf_data,
        fact_conf_data_del,
    })
}

pub(crate) async fn find_fact_record(
    fact_conf_key: &str,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    _funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<Vec<StatsExportAggResp>> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (conn, _) = common_pg::init_conn(bs_inst).await?;
    let table_name = package_table_name(&format!("stats_inst_fact_{fact_conf_key}"), ctx);
    let result = conn
        .query_all(
            &format!(
                r#"SELECT *
FROM {table_name}
WHERE 
     (ct > $1 or ct <= $2) and own_paths like $3
ORDER BY ct DESC
"#,
            ),
            vec![Value::from(start_time), Value::from(end_time), Value::from(format!("{}%", ctx.own_paths.clone()))],
        )
        .await?;

    let records = result
        .iter()
        .map(|item: &tardis::db::sea_orm::QueryResult| {
            let column_names = item.column_names();
            let mut data_map = HashMap::new();
            // 对每个列单独处理，转换为适当类型
            for k in column_names.iter() {
                if !["key", "own_paths", "ext", "ct", "idempotent_id"].contains(&k.as_str()) {
                    // 尝试以不同类型获取值，然后转换为 serde_json::Value
                    if let Ok(val) = item.try_get::<String>("", k) {
                        data_map.insert(k.clone(), serde_json::Value::String(val));
                    } else if let Ok(val) = item.try_get::<i64>("", k) {
                        data_map.insert(k.clone(), serde_json::Value::Number(val.into()));
                    } else if let Ok(val) = item.try_get::<f64>("", k) {
                        // 注意：这可能需要更复杂的转换，因为 serde_json::Number 不直接支持浮点数
                        if let Some(num) = serde_json::Number::from_f64(val) {
                            data_map.insert(k.clone(), serde_json::Value::Number(num));
                        }
                    } else if let Ok(val) = item.try_get::<bool>("", k) {
                        data_map.insert(k.clone(), serde_json::Value::Bool(val));
                    } else if let Ok(val) = item.try_get::<Option<String>>("", k) {
                        match val {
                            Some(s) => data_map.insert(k.clone(), serde_json::Value::String(s)),
                            None => data_map.insert(k.clone(), serde_json::Value::Null),
                        };
                    }
                    // 可以根据需要添加更多类型判断
                }
            }
            // 将 data_map 转换为 JsonValue
            let data = serde_json::Value::Object(data_map.into_iter().collect());
            Ok(StatsExportAggResp {
                key: item.try_get("", "key")?,
                own_paths: item.try_get("", "own_paths")?,
                ext: item.try_get("", "ext")?,
                ct: item.try_get("", "ct")?,
                idempotent_id: item.try_get("", "idempotent_id")?,
                data,
            })
        })
        .collect::<TardisResult<Vec<StatsExportAggResp>>>()?;
    Ok(records)
}

pub(crate) async fn find_fact_record_del(
    fact_conf_key: &str,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    _funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<Vec<StatsExportDelAggResp>> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (conn, _) = common_pg::init_conn(bs_inst).await?;
    let table_name = package_table_name(&format!("stats_inst_fact_{fact_conf_key}_del"), ctx);
    let result = conn
        .query_all(
            &format!(
                r#"SELECT key, ct
FROM {table_name}
WHERE 
     (ct > $1 or ct <= $2) and own_paths like $3
ORDER BY ct DESC
"#,
            ),
            vec![Value::from(start_time), Value::from(end_time), Value::from(format!("{}%", ctx.own_paths.clone()))],
        )
        .await?;

    let records = result
        .iter()
        .map(|item: &tardis::db::sea_orm::QueryResult| {
            Ok(StatsExportDelAggResp {
                key: item.try_get("", "key")?,
                ct: item.try_get("", "ct")?,
            })
        })
        .collect::<TardisResult<Vec<StatsExportDelAggResp>>>()?;
    Ok(records)
}

pub async fn import_data(import_req: &StatsImportDataReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
    let inst = funs.init(None, ctx, true, stats_initializer::init_fun).await?;
    let ctx_cloned = ctx.clone();
    let init_cloned = inst.clone();
    let fact_conf_data = import_req.fact_conf_data.clone();
    let fact_conf_data_del = import_req.fact_conf_data_del.clone();
    TaskProcessor::execute_task_with_ctx(
        &funs.conf::<StatsConfig>().cache_key_async_task_status,
        {
            move |_task_id| async move {
                let funs = crate::get_tardis_inst();
                let _ = import_fact_record(fact_conf_data.clone(), &funs, &ctx_cloned, &init_cloned).await?;
                let _ = import_fact_record_del(fact_conf_data_del.clone(), &funs, &ctx_cloned, &init_cloned).await?;
                Ok(())
            }
        },
        &funs.cache(),
        "spi-stats".to_string(),
        Some(vec![format!("account/{}", ctx.owner)]),
        ctx,
    )
    .await?;
    Ok(true)
}

pub async fn import_fact_record(fact_conf_data: HashMap<String, Vec<StatsImportAggReq>>, funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<bool> {
    for (fact_conf_key, fact_records) in &fact_conf_data {
        let load_records = fact_records
            .iter()
            .map(|record| {
                Ok(StatsFactRecordsLoadReq {
                    key: record.key.clone(),
                    own_paths: record.own_paths.clone(),
                    ext: record.ext.clone(),
                    ct: record.ct.clone(),
                    idempotent_id: record.idempotent_id.clone(),
                    ignore_updates: Some(true),
                    data: record.data.clone(),
                })
            })
            .collect::<TardisResult<Vec<StatsFactRecordsLoadReq>>>()?;
        stats_pg_record_serv::fact_records_load(fact_conf_key, load_records, funs, ctx, inst).await?;
    }
    Ok(true)
}

pub async fn import_fact_record_del(
    fact_conf_data_del: HashMap<String, Vec<StatsImportDelAggReq>>,
    _funs: &TardisFunsInst,
    ctx: &TardisContext,
    inst: &SpiBsInst,
) -> TardisResult<bool> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, _) = common_pg::init_conn(bs_inst).await?;
    conn.begin().await?;
    for (fact_conf_key, fact_records) in &fact_conf_data_del {
        if !stats_pg_conf_fact_serv::online(fact_conf_key, &conn, ctx).await? {
            continue;
        }
        let table_name = package_table_name(&format!("stats_inst_fact_{fact_conf_key}_del"), ctx);
        for delete_record in fact_records {
            let count: i64 = conn
                .query_one(
                    &format!("SELECT count(1) as _count FROM {table_name} WHERE key = $1 and ct = $2"),
                    vec![Value::from(delete_record.key.clone()), Value::from(delete_record.ct.clone())],
                )
                .await?
                .ok_or_else(|| TardisError::not_found("not found", "404-spi-log-not-found"))?
                .try_get("", "_count")?;
            if count > 0 {
                continue;
            }
            conn.execute_one(
                &format!(
                    r#"INSERT INTO {table_name}
    (key, ct)
    VALUES
    ($1, $2)
    "#,
                ),
                vec![Value::from(delete_record.key.clone()), Value::from(delete_record.ct.clone())],
            )
            .await?;
        }
    }
    conn.commit().await?;
    Ok(true)
}
