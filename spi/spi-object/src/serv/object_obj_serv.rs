use bios_basic::spi::spi_funs::SpiBsInstExtractor;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::TardisFunsInst;

use crate::dto::object_dto::ObjectObjPresignKind;
use crate::{object_constants, object_initializer};

use super::s3;

pub async fn presign_obj_url(
    presign_kind: ObjectObjPresignKind,
    object_path: &str,
    max_width: Option<String>,
    max_height: Option<String>,
    exp_secs: u32,
    private: bool,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<String> {
    let inst = funs.init(ctx, true, object_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-s3")]
        object_constants::SPI_S3_KIND_CODE => s3::object_s3_obj_serv::presign_obj_url(presign_kind, object_path, max_width, max_height, exp_secs, private, funs, ctx, &inst).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}
