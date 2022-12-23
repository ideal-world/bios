use std::collections::HashMap;

use bios_basic::spi::{dto::spi_bs_dto::SpiBsCertResp, spi_funs::SpiBsInst, spi_initializer};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    db::reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
    TardisFuns,
};

pub async fn init(bs_cert: &SpiBsCertResp, ctx: &TardisContext) -> TardisResult<SpiBsInst> {
    let ext = TardisFuns::json.str_to_json(&bs_cert.ext)?;
    let client = TardisRelDBClient::init(
        &bs_cert.conn_uri,
        ext.get("max_connections").unwrap().as_u64().unwrap() as u32,
        ext.get("min_connections").unwrap().as_u64().unwrap() as u32,
        None,
        None,
    )
    .await?;
    let mut ext = HashMap::new();
    let schema_name = if bs_cert.private {
        "".to_string()
    } else {
        spi_initializer::init_pg_schema(&client, ctx).await?
    };
    spi_initializer::set_pg_schema_to_ext(&schema_name, &mut ext);
    Ok(SpiBsInst { client: Box::new(client), ext })
}

pub async fn init_conn(bs_inst: (&TardisRelDBClient, &HashMap<String, String>, String)) -> TardisResult<TardisRelDBlConnection> {
   let conn = bs_inst.0.conn();
    if let Some(schema_name) = spi_initializer::get_pg_schema_from_ext(bs_inst.1) {
        spi_initializer::set_pg_schema_to_session(&schema_name, &conn).await?;
    }
    Ok(conn)
}
