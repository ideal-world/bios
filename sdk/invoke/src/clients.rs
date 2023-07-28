use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    TardisFuns,
};

use crate::invoke_constants::TARDIS_CONTEXT;

#[cfg(feature = "spi-base")]
mod base_spi_client;
#[cfg(feature = "iam")]
pub mod iam_client;
#[cfg(feature = "spi-kv")]
pub mod spi_kv_client;
#[cfg(feature = "spi-log")]
pub mod spi_log_client;

pub trait TardisCtxTrait {
    fn get_ctx(&self) -> &TardisContext;
    fn get_tardis_context(&self)  -> TardisResult<(String, String)> {
        let ctx = self.get_ctx();
        Ok((TARDIS_CONTEXT.to_string(), TardisFuns::crypto.base64.encode(&TardisFuns::json.obj_to_string(ctx)?)))
    }
}