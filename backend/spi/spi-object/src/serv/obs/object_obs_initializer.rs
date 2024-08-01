use bios_basic::spi::{dto::spi_bs_dto::SpiBsCertResp, spi_funs::SpiBsInst};
use tardis::basic::{dto::TardisContext, result::TardisResult};

use crate::serv::s3;

pub async fn init(bs_cert: &SpiBsCertResp, ctx: &TardisContext, mgr: bool) -> TardisResult<SpiBsInst> {
    s3::object_s3_initializer::init(bs_cert, ctx, mgr).await
}
