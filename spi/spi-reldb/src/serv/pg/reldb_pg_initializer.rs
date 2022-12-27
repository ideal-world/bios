use std::collections::HashMap;

use bios_basic::spi::{dto::spi_bs_dto::SpiBsCertResp, spi_initializer};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    db::reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
};

pub async fn init(bs_cert: &SpiBsCertResp, client: &TardisRelDBClient, ctx: &TardisContext) -> TardisResult<HashMap<String, String>> {
    let mut ext = HashMap::new();
    let schema_name = if bs_cert.private {
        "".to_string()
    } else {
        spi_initializer::common_pg::create_schema(client, ctx).await?
    };
    spi_initializer::common_pg::set_schema_name_to_ext(&schema_name, &mut ext);
    Ok(ext)
}

pub async fn init_conn(conn: TardisRelDBlConnection, ext: &HashMap<String, String>) -> TardisResult<TardisRelDBlConnection> {
    if let Some(schema_name) = spi_initializer::common_pg::get_schema_name_from_ext(ext) {
        spi_initializer::common_pg::set_schema_to_session(&schema_name, &conn).await?;
    }
    Ok(conn)
}
