use bios_basic::spi::{spi_funs::SpiBsInst, spi_initializer::common};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    os::os_client::TardisOSClient,
    TardisFunsInst,
};

use crate::serv::s3::S3;

/// OBS need manually configure lifecycle rules
/// Most interfaces of the obs service use bucket domains to share the code logic of s3.
/// However, the copy interface requires additional processing, which only supports specifying absolute file paths, so use a region domain and specify the bucket in the bucket domain as the default bucket, and pass in the bucket as a directory prefix in the code.
/// obs服务大多数接口使用桶域名共用s3的代码逻辑。
/// 但copy接口需额外处理，它只支持指定绝对文件路径，所以使用区域域名并将桶域名中的桶指定为默认桶，同时将代码中的桶当作目录前缀传入。
pub(crate) struct OBSService;
impl S3 for OBSService {
    async fn rebuild_path(_bucket_name: Option<&str>, origin_path: &str, _obj_exp: Option<u32>, _client: &TardisOSClient) -> TardisResult<String> {
        Ok(origin_path.to_string())
    }

    fn get_bucket_name(private: Option<bool>, special: Option<bool>, obj_exp: Option<u32>, bucket_name: Option<&str>, bs_id: Option<&str>, inst: &SpiBsInst) -> Option<String> {
        let bs_inst = inst.inst::<TardisOSClient>();
        common::get_isolation_flag_from_ext(bs_inst.1).map(|bucket_name_prefix| {
            format!(
                "{}-{}",
                bucket_name_prefix,
                if special.unwrap_or(false) {
                    "spe"
                } else if private.unwrap_or(true) {
                    "pri"
                } else {
                    "pub"
                }
            )
        })
    }

    //
    async fn object_copy(
        from: &str,
        to: &str,
        private: Option<bool>,
        special: Option<bool>,
        bs_id: Option<&str>,
        bucket: Option<&str>,
        _funs: &TardisFunsInst,
        _ctx: &TardisContext,
        inst: &SpiBsInst,
    ) -> TardisResult<()> {
        let bs_inst = inst.inst::<TardisOSClient>();
        let client = bs_inst.0;
        let bucket_name = Self::get_bucket_name(private, special, None, bucket, bs_id, inst).unwrap_or_default();
        client
            .object_copy(
                &format!("{}/{}", &bucket_name, from.strip_prefix('/').unwrap_or(from),),
                &format!("{}/{}", &bucket_name, to.strip_prefix('/').unwrap_or(to),),
                None,
            )
            .await
    }
}
