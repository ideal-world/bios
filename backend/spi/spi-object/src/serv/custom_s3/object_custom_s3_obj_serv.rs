use bios_basic::spi::dto::spi_bs_dto::SpiBsCertResp;
use bios_basic::spi::dto::spi_bs_dto::SpiBsDetailResp;
use bios_basic::spi::spi_constants;
use bios_basic::spi::spi_funs::SpiBsInst;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::OnceLock;

use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    os::os_client::TardisOSClient,
    tokio::sync::RwLock,
};

use crate::serv::s3::S3;

/// 自定义外部s3服务
/// 文件服务支持绕过spi配置而直接根据外部传入的配置创建客户端连接。
pub(crate) struct CustomS3Service;
impl S3 for CustomS3Service {
    async fn rebuild_path(_bucket_name: Option<&str>, origin_path: &str, _obj_exp: Option<u32>, _client: &TardisOSClient) -> TardisResult<String> {
        Ok(origin_path.to_string())
    }
}

impl CustomS3Service {
    pub async fn get_bs(bs_cert: &SpiBsDetailResp, ctx: &TardisContext) -> TardisResult<Arc<SpiBsInst>> {
        {
            let read = Self::get_custom_bs_caches().read().await;
            if let Some(inst) = read.get(&bs_cert.id).cloned() {
                return Ok(inst);
            }
        }
        let mut spi_bs_inst = crate::serv::custom_s3::object_custom_s3_initializer::init(
            &SpiBsCertResp {
                kind_code: bs_cert.kind_code.clone(),
                conn_uri: bs_cert.conn_uri.clone(),
                ak: bs_cert.ak.clone(),
                sk: bs_cert.sk.clone(),
                ext: bs_cert.ext.clone(),
                private: bs_cert.private,
            },
            ctx,
            true,
        )
        .await?;
        {
            let mut write = Self::get_custom_bs_caches().write().await;
            spi_bs_inst.ext.insert(spi_constants::SPI_KIND_CODE_FLAG.to_string(), bs_cert.kind_code.clone());
            return Ok(write.entry(bs_cert.id.clone()).or_insert(Arc::new(spi_bs_inst)).clone());
        }
    }
    fn get_custom_bs_caches() -> &'static RwLock<HashMap<String, Arc<SpiBsInst>>> {
        static CUSTOM_BS_CACHES: OnceLock<RwLock<HashMap<String, Arc<SpiBsInst>>>> = OnceLock::new();
        CUSTOM_BS_CACHES.get_or_init(Default::default)
    }
}
