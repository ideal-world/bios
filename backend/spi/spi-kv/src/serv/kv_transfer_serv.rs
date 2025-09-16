use bios_basic::process::task_processor::TaskProcessor;

use bios_basic::spi::spi_funs::{SpiBsInst, SpiBsInstExtractor};
use tardis::basic::result::TardisResult;
use tardis::db::sea_orm::Value;
use tardis::TardisFunsInst;
use tardis::{basic::dto::TardisContext, db::reldb_client::TardisRelDBClient};

use crate::dto::kv_transfer_dto::{KvExportAggResp, KvExportDataReq, KvExportDataResp, KvImportAggReq, KvImportDataReq};
use crate::kv_config::KvConfig;
use crate::kv_initializer;

use super::pg::kv_pg_initializer;

pub async fn export_data(export_req: &KvExportDataReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<KvExportDataResp> {
    let inst_arc = funs.init(None, ctx, true, kv_initializer::init_fun).await?;
    let bs_inst = inst_arc.inst::<TardisRelDBClient>();
    let (conn, table_name) = kv_pg_initializer::init_table_and_conn(bs_inst, ctx, true).await?;
    let kv_data = conn
        .find_dtos_by_sql::<KvExportAggResp>(
            &format!(
                r#"SELECT k AS key, v AS value, info, owner, own_paths, disable, scope_level, create_time, update_time
FROM {}
WHERE ((create_time > $1 and create_time < $2) or (update_time > $1 and update_time <= $2)) AND k NOT LIKE 'flow:config:%'
ORDER BY create_time DESC
"#,
                table_name
            ),
            vec![Value::from(export_req.start_time.clone()), Value::from(export_req.end_time.clone())],
        )
        .await?;
    Ok(KvExportDataResp { kv_data })
}

pub async fn import_data(receive_req: &KvImportDataReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
    let inst = funs.init(None, ctx, true, kv_initializer::init_fun).await?;
    let ctx_cloned = ctx.clone();
    let init_cloned = inst.clone();
    let kv_data = receive_req.kv_data.clone();
    TaskProcessor::execute_task_with_ctx(
        &funs.conf::<KvConfig>().cache_key_async_task_status,
        {
            move |_task_id| async move {
                let funs = crate::get_tardis_inst();
                let _ = import_kv(kv_data.clone(), &funs, &ctx_cloned, &init_cloned).await?;
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

pub async fn import_kv(kv_data: Vec<KvImportAggReq>, _funs: &TardisFunsInst, ctx: &TardisContext, inst: &SpiBsInst) -> TardisResult<bool> {
    let bs_inst = inst.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = kv_pg_initializer::init_table_and_conn(bs_inst, ctx, true).await?;
    conn.begin().await?;
    for kv in &kv_data {
        let sql = format!(
            r#"INSERT INTO {} (k, v, info, owner, own_paths, disable, scope_level, create_time, update_time)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) 
ON CONFLICT (k) DO UPDATE SET v = EXCLUDED.v, info = EXCLUDED.info, owner = EXCLUDED.owner, own_paths = EXCLUDED.own_paths, disable = EXCLUDED.disable, scope_level = EXCLUDED.scope_level, create_time = EXCLUDED.create_time, update_time = EXCLUDED.update_time"#,
            table_name
        );
        let params = vec![
            Value::from(kv.key.clone()),
            Value::from(kv.value.clone()),
            Value::from(kv.info.clone()),
            Value::from(kv.owner.clone()),
            Value::from(kv.own_paths.clone()),
            Value::from(kv.disable),
            Value::from(kv.scope_level),
            Value::from(kv.create_time),
            Value::from(kv.update_time),
        ];
        conn.execute_one(&sql, params).await?;
    }
    conn.commit().await?;
    Ok(true)
}
