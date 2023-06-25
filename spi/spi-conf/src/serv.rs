use bios_basic::spi::{spi_constants, spi_funs::SpiBsInstExtractor};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    db::sea_orm::{prelude::Uuid, sea_query::token},
    web::{poem, reqwest::StatusCode},
    TardisFunsInst,
};

use crate::{
    conf_config::ConfConfig,
    conf_initializer,
    dto::{conf_config_dto::*, conf_config_nacos_dto::NacosJwtClaim, conf_namespace_dto::*},
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
    /// get config detail
    get_config_detail(descriptor: &mut ConfigDescriptor) -> TardisResult<ConfigItem>;
    /// get content's md5 value by descriptor
    get_md5(descriptor: &mut ConfigDescriptor) -> TardisResult<String>;
    /// delete config
    delete_config(descriptor: &mut ConfigDescriptor) -> TardisResult<bool>;
    /// get config by namespace
    get_configs_by_namespace(namespace_id: &NamespaceId) -> TardisResult<Vec<ConfigItemDigest>>;
    /// get config
    get_configs(req: ConfigListRequest, mode: SearchMode) -> TardisResult<ConfigListResponse>;

    // for config history
    /// get config history list
    get_history_list_by_namespace(req: &mut ConfigHistoryListRequest) -> TardisResult<ConfigListResponse>;
    /// find come certain history
    find_history(descriptor: &mut ConfigDescriptor, id: &Uuid) -> TardisResult<ConfigItem>;
    /// find previous history
    find_previous_history(descriptor: &mut ConfigDescriptor, id: &Uuid) -> TardisResult<ConfigItem>;
}

pub fn auth(username: &str, password: &str, funs: &TardisFunsInst) -> bool {
    let cfg = funs.conf::<ConfConfig>();
    cfg.auth_username == username && cfg.auth_password == password
}

pub fn jwt_sign(funs: &TardisFunsInst) -> poem::Result<String> {
    use jsonwebtoken::*;
    let cfg = funs.conf::<ConfConfig>();
    let claim = NacosJwtClaim::gen(cfg.token_ttl as u64, &cfg.auth_username);

    let key =
        EncodingKey::from_base64_secret(&cfg.auth_key).map_err(|_| poem::Error::from_string("spi-conf nacosmocker using an invalid authkey", StatusCode::INTERNAL_SERVER_ERROR))?;

    let token = encode(&Header::new(Algorithm::HS256), &claim, &key).map_err(|e| {
        dbg!(e);
        poem::Error::from_string("spi-conf nacosmocker fail to encode auth token", StatusCode::INTERNAL_SERVER_ERROR)
    })?;

    Ok(token)
}

pub fn jwt_validate(token: &str, funs: &TardisFunsInst) -> poem::Result<NacosJwtClaim> {
    use jsonwebtoken::*;
    let cfg = funs.conf::<ConfConfig>();
    let mut validation = Validation::new(Algorithm::HS256);
    validation.sub = Some(cfg.auth_username.clone());
    let key =
        DecodingKey::from_base64_secret(&cfg.auth_key).map_err(|_| poem::Error::from_string("spi-conf nacosmocker using an invalid authkey", StatusCode::INTERNAL_SERVER_ERROR))?;
    let token_data = decode::<NacosJwtClaim>(token, &key, &validation).map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::FORBIDDEN))?;
    Ok(token_data.claims)
}
