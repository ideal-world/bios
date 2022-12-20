use std::any::Any;

use bios_basic::spi::dto::spi_bs_dto::SpiBsCertResp;
use bios_basic::spi::spi_funs::SpiBsInstExtractor;
use tardis::basic::result::TardisResult;
use tardis::db::sea_orm::Value;
use tardis::{basic::dto::TardisContext, db::reldb_client::TardisRelDBClient};
use tardis::{TardisFuns, TardisFunsInst};

use crate::dto::reldb_exec_dto::{ReldbDmlReq, ReldbDmlResp};

pub struct ReldbExecServ;

async fn init_fun(cert: SpiBsCertResp) -> TardisResult<Box<dyn Any + Send>> {
    let ext = TardisFuns::json.str_to_json(&cert.ext)?;
    let client = TardisRelDBClient::init(
        &cert.conn_uri,
        ext.get("max_connections").unwrap().as_u64().unwrap() as u32,
        ext.get("min_connections").unwrap().as_u64().unwrap() as u32,
        None,
        None,
    )
    .await?;
    Ok(Box::new(client))
}

impl ReldbExecServ {
    pub async fn dml(dml_req: &mut ReldbDmlReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<ReldbDmlResp> {
        let client = funs.bs(ctx, init_fun).await?.as_ref().downcast_ref::<TardisRelDBClient>().unwrap();
        let params = dml_req.params.as_array().unwrap();
        let params = params
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
            .collect::<Vec<Value>>();
        client.conn().execute_one(&dml_req.sql, params).await?;
        Ok(ReldbDmlResp {
            affected_rows: 0,
            new_row_ids: Vec::new(),
        })
    }
}
