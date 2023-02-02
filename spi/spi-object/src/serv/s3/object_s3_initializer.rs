use std::collections::HashMap;

use bios_basic::spi::{dto::spi_bs_dto::SpiBsCertResp, spi_funs::SpiBsInst, spi_initializer};
use tardis::{
    basic::{dto::TardisContext, error::TardisError, result::TardisResult},
    os::os_client::TardisOSClient,
    TardisFuns,
};

pub async fn init(bs_cert: &SpiBsCertResp, ctx: &TardisContext, _: bool) -> TardisResult<SpiBsInst> {
    let ext = TardisFuns::json.str_to_json(&bs_cert.ext)?;
    let client = TardisOSClient::init(
        "s3",
        ext.get("endpoint").unwrap().as_str().unwrap(),
        ext.get("ak").unwrap().as_str().unwrap(),
        ext.get("sk").unwrap().as_str().unwrap(),
        ext.get("region").unwrap().as_str().unwrap(),
        if let Some(default_bucket) = ext.get("default_bucket") {
            default_bucket.as_str().unwrap()
        } else {
            ""
        },
    )?;
    let mut ext = HashMap::new();
    if !bs_cert.private {
        let bucket_name_prefix = spi_initializer::common::get_isolation_flag_from_context(ctx);
        let resp = client.bucket_create_simple(&format!("{}-pri", bucket_name_prefix), true).await;
        if resp.is_err() && resp.err().unwrap().code != "409" {
            return Err(TardisError::internal_error(&format!("Bucket {}-pri creation failed", bucket_name_prefix), ""));
        }
        let resp = client.bucket_create_simple(&format!("{}-pub", bucket_name_prefix), false).await;
        if resp.is_err() && resp.err().unwrap().code != "409" {
            return Err(TardisError::internal_error(&format!("Bucket {}-pub creation failed", bucket_name_prefix), ""));
        }
        spi_initializer::common::set_isolation_flag_to_ext(&bucket_name_prefix, &mut ext);
    };
    Ok(SpiBsInst { client: Box::new(client), ext })
}
