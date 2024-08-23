use std::collections::HashMap;

use bios_basic::spi::{dto::spi_bs_dto::SpiBsCertResp, spi_funs::SpiBsInst, spi_initializer};
use tardis::{basic::{dto::TardisContext, error::TardisError, result::TardisResult}, config::config_dto::OSModuleConfig, os::os_client::TardisOSClient, TardisFuns};

use crate::object_constants::USE_REGION_ENDPOINT;

use tardis::serde_json::Value as JsonValue;

/// obs存储不支持初始化建桶，并且默认使用指定的桶域名为endpoint。
/// 若ctx中存在USE_REGION_ENDPOINT标识，则使用ext中的region_endpoint作为endpoint。
/// The obs store does not support initial bucket building and uses the specified bucket domain name for conn_uri by default.
/// If the USE_REGION_ENDPOINT identifier exists in the ctx. the region_endpoint in ext is used as the endpoint.
pub async fn init(bs_cert: &SpiBsCertResp, ctx: &TardisContext, mgr: bool) -> TardisResult<SpiBsInst> {
    let ext = TardisFuns::json.str_to_json(&bs_cert.ext)?;
    let region = ext
        .get("region")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| TardisError::bad_request("Tardis context ext should have a `region` field with type string", "400-spi-invalid-tardis-ctx"))?;
    if ctx.ext.read().await.contains_key(USE_REGION_ENDPOINT) {
        let default_bucket = ext
            .get("default_bucket")
            .and_then(JsonValue::as_str)
            .ok_or_else(|| TardisError::bad_request("Tardis context ext should have a `default_bucket` field with type string", "400-spi-invalid-tardis-ctx"))?;
        let conn_uri = ext
            .get("region_endpoint")
            .and_then(JsonValue::as_str)
            .ok_or_else(|| TardisError::bad_request("Tardis context ext should have a `region_endpoint` field with type string", "400-spi-invalid-tardis-ctx"))?;
        init_obs_spi_bs(&conn_uri, &bs_cert.ak, &bs_cert.sk, region,& default_bucket, bs_cert.private, ctx, mgr).await
    } else {
        // When using a bucket domain, default_bucket is set to empty
        let default_bucket = "";
        init_obs_spi_bs(&bs_cert.conn_uri, &bs_cert.ak, &bs_cert.sk, region, default_bucket, bs_cert.private, ctx, mgr).await
    }
}

async fn init_obs_spi_bs(conn_uri: &str, ak: &str, sk: &str, region: &str, default_bucket: &str, private: bool, ctx: &TardisContext, _: bool) -> TardisResult<SpiBsInst> {
    let tardis_os_config = OSModuleConfig::builder().kind("s3").endpoint(conn_uri).ak(ak).sk(sk).region(region).default_bucket(default_bucket).build();
    let client = TardisOSClient::init(&tardis_os_config)?;
    let mut ext = HashMap::new();
    if !private {
        let bucket_name_prefix = spi_initializer::common::get_isolation_flag_from_context(ctx);
        spi_initializer::common::set_isolation_flag_to_ext(&bucket_name_prefix, &mut ext);
    };
    Ok(SpiBsInst { client: Box::new(client), ext })
}