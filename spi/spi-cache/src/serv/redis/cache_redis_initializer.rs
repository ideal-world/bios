use std::collections::HashMap;

use bios_basic::spi::{dto::spi_bs_dto::SpiBsCertResp, spi_funs::SpiBsInst, spi_initializer};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    cache::cache_client::TardisCacheClient,
};

pub async fn init(bs_cert: &SpiBsCertResp, ctx: &TardisContext, _: bool) -> TardisResult<SpiBsInst> {
    let client = TardisCacheClient::init(&bs_cert.conn_uri).await?;
    let mut ext = HashMap::new();
    if !bs_cert.private {
        let key_prefix = spi_initializer::common::get_isolation_flag_from_context(ctx);
        spi_initializer::common::set_isolation_flag_to_ext(&key_prefix, &mut ext);
    };
    Ok(SpiBsInst { client: Box::new(client), ext })
}
