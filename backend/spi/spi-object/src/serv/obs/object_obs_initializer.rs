use std::collections::HashMap;

use bios_basic::spi::{dto::spi_bs_dto::SpiBsCertResp, spi_funs::SpiBsInst, spi_initializer};
use tardis::{
    basic::{dto::TardisContext, error::TardisError, result::TardisResult},
    config::config_dto::OSModuleConfig,
    os::os_client::TardisOSClient,
    TardisFuns,
};

use crate::object_constants::USE_REGION_ENDPOINT;

use tardis::serde_json::Value as JsonValue;

/// obs存储不支持在 Regional endpoint 下建桶。
/// 所以为了保持内置服务的逻辑统一，我们使用桶域名作为 s3 的 endpoint 创建连接。
/// 此时不再需要初始化建桶的操作，我们在桶域名下指定桶，obs会将指定的桶名当作一级目录。
/// 存在例外的情况，目前发现 copy 接口只能在 Regional endpoint 下正常调用，在桶域名下会报错找不到对象。
/// 所以我们需要两个连接。
/// 一个是桶域名连接，此时指定的 bucket 会被当作桶内的一级目录。
/// 一个是区域域名连接，此时将桶域名指定的桶当作default bucket 初始化时传入。实际使用时的bucket_name当作路径拼接在object_path前端。
/// 若ctx中存在USE_REGION_ENDPOINT标识，则使用区域域名连接。
/// 举例说明：假设obs服务的 Regional endpoint: obs.ap-southeast-1.myhuaweicloud.com 。业务使用的桶为 bios-test 。桶中存在文件路径为 pri/test.txt 。
/// 1、使用桶域名访问该文件，endpoint为bios-test.obs.ap-southeast-1.myhuaweicloud.com。bucket_name传pri,object_path为text.txt。
/// 2、使用区域域名访问该文件，endpoint为obs.ap-southeast-1.myhuaweicloud.com。bucket_name传bios-test,object_path为 pri/test.txt。
///
/// The obs storage does not support buckets under the Regional endpoint.
/// So in order to keep the logic of the built-in services uniform, we use the bucket domain name as the endpoint for s3 to create the connection.
/// At this point there is no need to initialize the bucket building operation, we specify the bucket under the bucket domain name and obs will treat the specified bucket name as a first level directory.
/// There is an exception to this, as it has been found that the copy interface can only be invoked under the Regional endpoint, but under the bucket domain, it will report that the object was not found.
/// So we need two connections.
/// A bucket connection, where the specified bucket is treated as a first-level directory within the bucket.
/// One is a zone domain connection, where the bucket specified by the bucket domain is passed in as the default bucket when it is initialized. The actual bucket_name is used as a path to be spliced in front of object_path.
/// If the USE_REGION_ENDPOINT flag exists in the ctx, then the regional domain name connection is used.
/// For example, suppose the regional endpoint for the obs service: obs.ap-southeast-1.myhuaweicloud.com. The bucket used by the service is bios-test. The path to the file in the bucket is pri/test.txt.
/// 1. Access the file using the bucket domain name with endpoint bios-test.obs.ap-southeast-1.myhuaweicloud.com. bucket_name passes pri,object_path is text.txt.
/// 2. Access the file using the zone domain name, endpoint is obs.ap-southeast-1.myhuaweicloud.com. bucket_name passes bios-test,object_path is pri/test.txt.
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
        init_obs_spi_bs(conn_uri, &bs_cert.ak, &bs_cert.sk, region, default_bucket, bs_cert.private, ctx, mgr).await
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
