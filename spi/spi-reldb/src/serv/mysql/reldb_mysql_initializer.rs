use std::collections::HashMap;

use bios_basic::spi::dto::spi_bs_dto::SpiBsCertResp;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    db::reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
};

pub async fn init(bs_cert: &SpiBsCertResp, client: &TardisRelDBClient, ctx: &TardisContext) -> TardisResult<HashMap<String, String>> {
    let ext = HashMap::new();
    // TODO
    Ok(ext)
}

pub async fn init_conn(conn: TardisRelDBlConnection, ext: &HashMap<String, String>) -> TardisResult<TardisRelDBlConnection> {
    // TODO
    Ok(conn)
}
