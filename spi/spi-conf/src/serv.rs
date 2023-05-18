use bios_basic::spi::{spi_funs::SpiBsInstExtractor, spi_constants};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    TardisFunsInst,
};

use crate::{conf_initializer, dto::conf_namespace_dto::*};
#[cfg(feature = "spi-pg")]
mod pg;
#[cfg(feature = "spi-pg")]
use pg::*;

macro_rules! dispatch_servive {
    ($(
        $service:ident($($arg: ident: $type: ty),*) -> $ret:ty;
    )*) => {
        $(
            pub async fn $service($($arg: $type),*,funs: &TardisFunsInst, ctx: &TardisContext) -> $ret {
                match funs.init(ctx, true, conf_initializer::init_fun).await?.as_str() {
                    #[cfg(feature = "spi-pg")]
                    spi_constants::SPI_PG_KIND_CODE => conf_pg_namespace_serv::$service($($arg),*, funs, ctx).await,
                    kind_code => Err(funs.bs_not_implemented(kind_code)),
                }
            }
        )*
    };
}

dispatch_servive! {
    create_namespace(attribute: &mut NamespaceAttribute) -> TardisResult<()>;
    get_namespace(discriptor: &mut NamespaceDescriptor) -> TardisResult<NamespaceItem>;
}
// pub async fn create_namespace(attribute: &mut NamespaceAttribute, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
//     match funs.init(ctx, true, conf_initializer::init_fun).await?.as_str() {
//         #[cfg(feature = "spi-pg")]
//         spi_constants::SPI_PG_KIND_CODE => conf_pg_namespace_serv::create_namespace(attribute, funs, ctx).await,
//         kind_code => Err(funs.bs_not_implemented(kind_code)),
//     }
// }

// pub async fn get_namespace(discriptor: &mut NamespaceDescriptor, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<NamespaceItem> {
//     match funs.init(ctx, true, conf_initializer::init_fun).await?.as_str() {
//         #[cfg(feature = "spi-pg")]
//         spi_constants::SPI_PG_KIND_CODE => conf_pg_namespace_serv::get_namespace(discriptor, funs, ctx).await,
//         kind_code => Err(funs.bs_not_implemented(kind_code)),
//     }
// }