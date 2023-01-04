use crate::dto::reldb_exec_dto::{ReldbDdlReq, ReldbDmlReq, ReldbDmlResp, ReldbDqlReq, ReldbTxResp};
use crate::reldb_initializer;
use bios_basic::spi::spi_funs::SpiBsInstExtractor;
use lazy_static::lazy_static;
use std::collections::HashMap;
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::chrono::Utc;
use tardis::db::reldb_client::TardisRelDBlConnection;
use tardis::db::sea_orm::{FromQueryResult, Value};
use tardis::log::trace;
use tardis::tokio::sync::RwLock;
use tardis::tokio::time::{self, Duration};
use tardis::{basic::dto::TardisContext, db::reldb_client::TardisRelDBClient};
use tardis::{serde_json, TardisFuns, TardisFunsInst};

lazy_static! {
    static ref TX_CONTAINER: RwLock<HashMap<String, (TardisRelDBlConnection, i64, bool)>> = RwLock::new(HashMap::new());
}

fn parse_params(params: &serde_json::Value) -> Vec<Value> {
    params
        .as_array()
        .unwrap()
        .iter()
        .map(|item| {
            if item.is_string() {
                Value::from(item.as_str().unwrap())
            } else if item.is_boolean() {
                Value::from(item.as_bool().unwrap())
            } else if item.is_u64() {
                Value::from(item.as_u64().unwrap())
            } else if item.is_f64() {
                Value::from(item.as_f64().unwrap())
            } else if item.is_i64() {
                Value::from(item.as_i64().unwrap())
            } else {
                // TODO
                Value::from(item.as_str().unwrap())
            }
        })
        .collect::<Vec<Value>>()
}

pub async fn tx_begin(auto_commit: bool, exp_sec: Option<u8>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<ReldbTxResp> {
    let tx_id = TardisFuns::crypto.hex.encode(TardisFuns::field.nanoid());
    let exp_ts_at = Utc::now().timestamp_millis() + (exp_sec.unwrap_or(5)) as i64 * 1000;
    let bs_inst = funs.init_bs(ctx, true, reldb_initializer::init_fun).await?.inst::<TardisRelDBClient>();
    let mut conn = reldb_initializer::inst_conn(bs_inst).await?;
    conn.begin().await?;
    let mut tx_container = TX_CONTAINER.write().await;
    tx_container.insert(tx_id.clone(), (conn, exp_ts_at, auto_commit));
    Ok(ReldbTxResp { tx_id, exp_ts_at })
}

pub async fn tx_commit(tx_id: String) -> TardisResult<()> {
    let mut tx_container = TX_CONTAINER.write().await;
    match tx_container.remove(&tx_id) {
        Some((conn, _, _)) => conn.commit().await?,
        None => return Err(TardisError::bad_request("tx not exist", "")),
    }
    Ok(())
}

pub async fn tx_rollback(tx_id: String) -> TardisResult<()> {
    let mut tx_container = TX_CONTAINER.write().await;
    match tx_container.remove(&tx_id) {
        Some((conn, _, _)) => conn.rollback().await?,
        None => return Err(TardisError::bad_request("tx not exist", "")),
    }
    Ok(())
}

pub async fn ddl(ddl_req: &mut ReldbDdlReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.init_bs(ctx, true, reldb_initializer::init_fun).await?.inst::<TardisRelDBClient>();
    let conn = reldb_initializer::inst_conn(bs_inst).await?;
    let params = parse_params(&ddl_req.params);
    conn.execute_one(&ddl_req.sql, params).await?;
    Ok(())
}

pub async fn dml(dml_req: &mut ReldbDmlReq, tx_id: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<ReldbDmlResp> {
    let bs_inst = funs.init_bs(ctx, true, reldb_initializer::init_fun).await?.inst::<TardisRelDBClient>();
    let params = parse_params(&dml_req.params);
    let resp = if let Some(tx_id) = &tx_id {
        let tx_container = TX_CONTAINER.read().await;
        match tx_container.get(tx_id) {
            Some((conn, _, _)) => conn.execute_one(&dml_req.sql, params).await,
            None => Err(TardisError::bad_request("tx not exist", "")),
        }
    } else {
        let conn = reldb_initializer::inst_conn(bs_inst).await?;
        conn.execute_one(&dml_req.sql, params).await
    };
    match resp {
        Ok(resp) => Ok(ReldbDmlResp {
            affected_rows: resp.rows_affected(),
        }),
        Err(e) => {
            if let Some(tx_id) = tx_id {
                tx_rollback(tx_id).await?;
            }
            trace!("[SPI-Reldb] dml error: {}", e);
            Err(e)
        }
    }
}

pub async fn dql(dql_req: &mut ReldbDqlReq, tx_id: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<serde_json::Value>> {
    let bs_inst = funs.init_bs(ctx, true, reldb_initializer::init_fun).await?.inst::<TardisRelDBClient>();
    let params = parse_params(&dql_req.params);
    let resp = if let Some(tx_id) = tx_id {
        let tx_container = TX_CONTAINER.read().await;
        match tx_container.get(&tx_id) {
            Some((conn, _, _)) => conn.query_all(&dql_req.sql, params).await,
            None => Err(TardisError::bad_request("tx not exist", "")),
        }
    } else {
        let conn = reldb_initializer::inst_conn(bs_inst).await?;
        conn.query_all(&dql_req.sql, params).await
    }?;
    let mut result: Vec<serde_json::Value> = Vec::new();
    for row in resp {
        let json = serde_json::Value::from_query_result_optional(&row, "")?.unwrap();
        result.push(json);
    }
    Ok(result)
}

pub async fn clean(clean_interval_sec: u8) {
    tardis::tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(clean_interval_sec as u64));
        loop {
            {
                let mut tx_container = TX_CONTAINER.write().await;
                let remove_keys = tx_container.iter().filter(|(_, v)| v.1 < Utc::now().timestamp_millis()).map(|(k, _)| k.to_string()).collect::<Vec<String>>();
                for key in remove_keys {
                    trace!("[SPI-Reldb] tx {} expired", key);
                    match tx_container.remove(&key) {
                        Some((conn, _, true)) => {
                            conn.commit().await.unwrap();
                        }
                        Some((conn, _, false)) => {
                            conn.rollback().await.unwrap();
                        }
                        _ => (),
                    }
                }
            }
            interval.tick().await;
        }
    });
}
