use tardis::{basic::result::TardisResult, os::os_client::TardisOSClient};

use crate::serv::s3::S3;

/// OBS need manually configure lifecycle rules
pub(crate) struct OBSService;
impl S3 for OBSService {
    async fn rebuild_path(_bucket_name: Option<&str>, origin_path: &str, _obj_exp: Option<u32>, _client: &TardisOSClient) -> TardisResult<String> {
        Ok(origin_path.to_string())
    }
}
