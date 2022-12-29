use bios_basic::spi::spi_funs::SpiBsInstExtractor;
use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::TardisFunsInst;

use crate::dto::object_dto::ObjectObjPresignKind;
use crate::{object_constants, object_initializer};

use super::s3;

pub struct ObjectObjServ;

impl ObjectObjServ {
    pub async fn presign_obj_url(
        presign_kind: ObjectObjPresignKind,
        object_path: String,
        max_width: Option<String>,
        max_height: Option<String>,
        exp_secs: u32,
        private: bool,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<String> {
        let kind_code = funs.init(ctx, true, object_initializer::init_fun).await?;
        match kind_code.as_str() {
            #[cfg(feature = "spi-s3")]
            object_constants::SPI_S3_KIND_CODE => s3::object_s3_obj_serv::presign_obj_url(presign_kind, object_path, max_width, max_height, exp_secs, private, funs, ctx).await,
            _ => Err(TardisError::not_implemented(
                &format!("Backend service kind {} does not exist or SPI feature is not enabled", kind_code),
                "406-rbum-*-enum-init-error",
            )),
        }
    }
}
