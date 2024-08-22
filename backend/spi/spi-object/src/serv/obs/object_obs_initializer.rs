use std::collections::HashMap;

use bios_basic::spi::{dto::spi_bs_dto::SpiBsCertResp, spi_funs::SpiBsInst, spi_initializer};
use tardis::{basic::{dto::TardisContext, error::TardisError, result::TardisResult}, config::config_dto::OSModuleConfig, os::os_client::TardisOSClient, TardisFuns};

use crate::object_constants::USE_REGION_ENDPOINT;

use tardis::serde_json::Value as JsonValue;

/// obs存储不支持初始化建桶，并且默认使用指定的桶域名为conn_uri。
/// 若ctx中存在USE_REGION_ENDPOINT标识，则使用区域域名（截取桶域名前端部分），并将截取的部分赋值为默认桶名。
/// The obs store does not support initial bucket building and uses the specified bucket domain name for conn_uri by default.
/// if the USE_REGION_ENDPOINT identifier is present in the ctx, the region domain name is used (intercepting the front portion of the bucket domain name) and assigning the intercepted portion to the default bucket name.
pub async fn init(bs_cert: &SpiBsCertResp, ctx: &TardisContext, mgr: bool) -> TardisResult<SpiBsInst> {
    let ext = TardisFuns::json.str_to_json(&bs_cert.ext)?;
    let region = ext
        .get("region")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| TardisError::bad_request("Tardis context ext should have a `region` field with type string", "400-spi-invalid-tardis-ctx"))?;
    if ctx.ext.read().await.contains_key(USE_REGION_ENDPOINT) {
        let default_bucket = bs_cert.conn_uri.split_once("//").map(|(_, s)| s.to_string()).map(|s| s.split_once(".").map(|(s,_)| s.to_string()).unwrap_or_default()).unwrap_or_default();
        let conn_uri = if default_bucket.is_empty() { bs_cert.conn_uri.clone() } else { bs_cert.conn_uri.split_once(&format!("{}.", default_bucket)).map(|(scheme, uri)| format!("{}{}", scheme, uri)).unwrap_or_default() };
        init_obs_spi_bs(&conn_uri, &bs_cert.ak, &bs_cert.sk, region,& default_bucket, bs_cert.private, ctx, mgr).await
    } else {
        let default_bucket = if let Some(default_bucket) = ext.get("default_bucket") {
            default_bucket
                .as_str()
                .ok_or_else(|| TardisError::bad_request("Tardis context ext should have a `default_bucket` field with type string", "400-spi-invalid-tardis-ctx"))?
        } else {
            ""
        };
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