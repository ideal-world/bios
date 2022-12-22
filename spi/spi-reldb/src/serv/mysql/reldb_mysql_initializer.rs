use std::collections::HashMap;

use bios_basic::spi::{dto::spi_bs_dto::SpiBsCertResp, spi_constants, spi_initializer};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    db::reldb_client::TardisRelDBClient,
};

pub async fn init(bs_cert: &SpiBsCertResp, client: &TardisRelDBClient, ctx: &TardisContext) -> TardisResult<HashMap<String, String>> {
    let mut ext = HashMap::new();
    Ok(ext)
}
