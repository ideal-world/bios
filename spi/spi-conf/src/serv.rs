use bios_basic::spi::{spi_constants, spi_funs::SpiBsInstExtractor};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    db::sea_orm::prelude::Uuid,
    TardisFunsInst,
};

use crate::{
    conf_initializer,
    dto::{conf_config_dto::*, conf_namespace_dto::*},
};
#[cfg(feature = "spi-pg")]
mod pg;

macro_rules! dispatch_servive {
    ($(
        $(#[$attr:meta])*
        $service:ident($($arg: ident: $type: ty),*) -> $ret:ty;
    )*) => {
        $(
            $(#[$attr])*
            pub async fn $service($($arg: $type,)* funs: &TardisFunsInst, ctx: &TardisContext) -> $ret {
                match funs.init(ctx, true, conf_initializer::init_fun).await?.as_str() {
                    #[cfg(feature = "spi-pg")]
                    spi_constants::SPI_PG_KIND_CODE => pg::$service($($arg,)* funs, ctx).await,
                    kind_code => Err(funs.bs_not_implemented(kind_code)),
                }
            }
        )*
    };
}

dispatch_servive! {
    // for namespace
    /// create a new namespace
    create_namespace(attribute: &mut NamespaceAttribute) -> TardisResult<()>;
    /// get a namespace
    get_namespace(discriptor: &mut NamespaceDescriptor) -> TardisResult<NamespaceItem>;
    /// update namespace
    edit_namespace(attribute: &mut NamespaceAttribute) -> TardisResult<()>;
    /// delete namespace
    delete_namespace(discriptor: &mut NamespaceDescriptor) -> TardisResult<()>;
    /// list namespace
    get_namespace_list() -> TardisResult<Vec<NamespaceItem>>;


    // for configs
    /// publich config
    publish_config(req: &mut ConfigPublishRequest) -> TardisResult<bool>;
    /// get config
    get_config(descriptor: &mut ConfigDescriptor) -> TardisResult<String>;
    /// get content's md5 value by descriptor
    get_md5(descriptor: &mut ConfigDescriptor) -> TardisResult<String>;
    /// delete config
    delete_config(descriptor: &mut ConfigDescriptor) -> TardisResult<bool>;
    /// get config by history
    get_configs_by_namespace(namespace_id: &NamespaceId) -> TardisResult<Vec<ConfigItemDigest>>;

    // for config history
    /// get config history list
    get_history_list_by_namespace(req: &mut ConfigHistoryListRequest) -> TardisResult<ConfigListResponse>;
    /// find come certain history
    find_history(descriptor: &mut ConfigDescriptor, id: &Uuid) -> TardisResult<ConfigItem>;
    /// find previous history
    find_previous_history(descriptor: &mut ConfigDescriptor, id: &Uuid) -> TardisResult<ConfigItem>;
}
