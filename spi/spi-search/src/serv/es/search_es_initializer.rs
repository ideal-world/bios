use std::collections::HashMap;

use bios_basic::spi::{dto::spi_bs_dto::SpiBsCertResp, spi_funs::SpiBsInst, spi_initializer};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    search::search_client::TardisSearchClient,
};

pub async fn init(bs_cert: &SpiBsCertResp, ctx: &TardisContext, mgr: bool) -> TardisResult<SpiBsInst> {
    let client = TardisSearchClient::init(&bs_cert.conn_uri, 60)?;
    let mut ext = HashMap::new();
    if !bs_cert.private {
        let key_prefix = spi_initializer::common::get_isolation_flag_from_context(ctx);
        spi_initializer::common::set_isolation_flag_to_ext(&key_prefix, &mut ext);
    };
    Ok(SpiBsInst { client: Box::new(client), ext })
}

pub async fn init_index(client: &TardisSearchClient, index: &str, mappings: Option<&str>) -> TardisResult<()> {
    if client.check_index_exist(index).await? {
        return Ok(());
    } else {
        client.create_index(index, mappings).await?
    }
    Ok(())
}
