use std::collections::HashMap;

use bios_basic::spi::{dto::spi_bs_dto::SpiBsCertResp, spi_funs::SpiBsInst, spi_initializer};
use tardis::{
    basic::{dto::TardisContext, error::TardisError, result::TardisResult},
    config::config_dto::OSModuleConfig,
    os::os_client::TardisOSClient,
    TardisFuns,
};

use tardis::serde_json::Value as JsonValue;

pub async fn init(bs_cert: &SpiBsCertResp, ctx: &TardisContext, _: bool) -> TardisResult<SpiBsInst> {
    let ext = TardisFuns::json.str_to_json(&bs_cert.ext)?;
    let region = ext.get("region").and_then(JsonValue::as_str).ok_or(TardisError::bad_request(
        "Tardis context ext should have a `region` field with type string",
        "400-spi-invalid-tardis-ctx",
    ))?;
    let default_bucket = if let Some(default_bucket) = ext.get("default_bucket") {
        default_bucket.as_str().ok_or(TardisError::bad_request(
            "Tardis context ext should have a `default_bucket` field with type string",
            "400-spi-invalid-tardis-ctx",
        ))?
    } else {
        ""
    };
    let tardis_os_config = OSModuleConfig::builder().kind("s3").endpoint(&bs_cert.conn_uri).ak(&bs_cert.ak).sk(&bs_cert.sk).region(region).default_bucket(default_bucket).build();
    let client = TardisOSClient::init(&tardis_os_config)?;
    let mut ext = HashMap::new();
    if !bs_cert.private {
        let bucket_name_prefix = spi_initializer::common::get_isolation_flag_from_context(ctx);
        let resp = client.bucket_create_simple(&format!("{bucket_name_prefix}-pri"), true).await;
        if let Err(e) = resp {
            if e.code != "409" {
                return Err(TardisError::internal_error(
                    &format!("Bucket {bucket_name_prefix}-pri creation failed"),
                    &format!("{:?}", e),
                ));
            }
        }
        let resp = client.bucket_create_simple(&format!("{bucket_name_prefix}-pub"), false).await;
        if let Err(e) = resp {
            if e.code != "409" {
                return Err(TardisError::internal_error(
                    &format!("Bucket {bucket_name_prefix}-pub creation failed"),
                    &format!("{:?}", e),
                ));
            }
        }
        spi_initializer::common::set_isolation_flag_to_ext(&bucket_name_prefix, &mut ext);
    };
    Ok(SpiBsInst { client: Box::new(client), ext })
}
