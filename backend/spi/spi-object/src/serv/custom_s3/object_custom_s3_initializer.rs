use std::collections::HashMap;

use bios_basic::spi::{dto::spi_bs_dto::SpiBsCertResp, spi_funs::SpiBsInst};
use tardis::{
    basic::{dto::TardisContext, error::TardisError, result::TardisResult},
    config::config_dto::OSModuleConfig,
    os::os_client::TardisOSClient,
    TardisFuns,
};

use tardis::serde_json::Value as JsonValue;

/// 自定义外部obs服务初始化
/// 外部服务由API申请资源时初始化，不需要随系统spi初始化
pub async fn init(bs_cert: &SpiBsCertResp, _ctx: &TardisContext, _mgr: bool) -> TardisResult<SpiBsInst> {
    let ext = TardisFuns::json.str_to_json(&bs_cert.ext)?;
    let region = ext
        .get("region")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| TardisError::bad_request("Tardis context ext should have a `region` field with type string", "400-spi-invalid-tardis-ctx"))?;
    let default_bucket = ext
        .get("default_bucket")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| TardisError::bad_request("Tardis context ext should have a `region` field with type string", "400-spi-invalid-tardis-ctx"))?;
    let tardis_os_config = OSModuleConfig::builder().kind("s3").endpoint(&bs_cert.conn_uri).ak(&bs_cert.ak).sk(&bs_cert.sk).region(region).default_bucket(default_bucket).build();
    let client = TardisOSClient::init(&tardis_os_config)?;
    Ok(SpiBsInst {
        client: Box::new(client),
        ext: HashMap::new(),
    })
}
