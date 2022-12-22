use std::collections::HashMap;

use bios_basic::spi::{dto::spi_bs_dto::SpiBsCertResp, spi_constants, spi_initializer};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    db::reldb_client::TardisRelDBClient,
};

pub async fn init(bs_cert: &SpiBsCertResp, client: &TardisRelDBClient, ctx: &TardisContext) -> TardisResult<HashMap<String, String>> {
    let mut ext = HashMap::new();
    let schema_name = if bs_cert.private {
        "".to_string()
    } else {
        spi_initializer::init_pg_schema(client, ctx).await?
    };
    ext.insert(spi_constants::SPI_PG_SCHEMA_NAME_FLAG.to_string(), schema_name);
    Ok(ext)
}
