use std::any::Any;

use bios_basic::spi::dto::spi_bs_dto::SpiBsCertResp;
use bios_basic::spi::spi_funs::{self, SpiBsInstExtractor};
use tardis::basic::result::TardisResult;
use tardis::{basic::dto::TardisContext, db::reldb_client::TardisRelDBClient};
use tardis::{TardisFuns, TardisFunsInst};

use crate::dto::reldb_exec_dto::{ReldbDmlReq, ReldbDmlResp};

pub struct ReldbExecServ;

async fn init_fun<T, F>(cert: SpiBsCertResp) -> TardisResult<Box<dyn Any + Send>> {
    let ext = TardisFuns::json.str_to_json(&cert.ext)?;
    let client = TardisRelDBClient::init(
        &cert.conn_uri,
        ext.get("max_connections").unwrap().as_u64().unwrap() as u32,
        ext.get("min_connections").unwrap().as_u64().unwrap() as u32,
        None,
        None,
    )
    .await?;
    let client = Box::new(client);
    Ok(client)
}

impl ReldbExecServ {
    pub async fn dml(dml_req: ReldbDmlReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<ReldbDmlResp> {
        let client = funs
            .bs(ctx, init_fun)
            .await?
            .as_ref();

        Ok(ReldbDmlResp {
            affected_rows: 0,
            new_row_ids: Vec::new(),
        })
    }
}
