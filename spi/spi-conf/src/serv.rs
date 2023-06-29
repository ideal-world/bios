use std::{collections::BTreeMap, sync::Arc};

use bios_basic::{
    rbum::{
        dto::{
            rbum_cert_dto::RbumCertAddReq,
            rbum_filer_dto::{RbumBasicFilterReq, RbumCertFilterReq, RbumItemRelFilterReq},
        },
        rbum_enumeration::{RbumCertRelKind, RbumCertStatusKind, RbumRelFromKind},
        serv::{rbum_cert_serv::RbumCertServ, rbum_crud_serv::RbumCrudOperation, rbum_item_serv::RbumItemCrudOperation},
    },
    spi::{
        dto::spi_bs_dto::SpiBsFilterReq,
        serv::spi_bs_serv::SpiBsServ,
        spi_constants::{self, SPI_IDENT_REL_TAG},
        spi_funs::SpiBsInstExtractor,
    },
};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    db::sea_orm::prelude::Uuid,
    log,
    serde_json::{self, json},
    tokio::{
        sync::OnceCell,
        time::{interval, Duration, Instant},
    },
    tokio::{sync::RwLock, task::JoinHandle},
    web::{poem, reqwest::StatusCode},
    TardisFunsInst,
};

use crate::{
    conf_config::ConfConfig,
    conf_constants::{error::*, *},
    conf_initializer,
    dto::{
        conf_auth_dto::{RegisterRequest, RegisterResponse},
        conf_config_dto::*,
        conf_config_nacos_dto::NacosJwtClaim,
        conf_namespace_dto::*,
    },
    utils::*,
};
#[cfg(feature = "spi-pg")]
mod pg;

macro_rules! dispatch_service {
    ($(
        $(#[$attr:meta])*
        $service:ident($($arg: ident: $type: ty),*) -> $ret:ty;
    )*) => {
        $(
            $(#[$attr])*
            pub async fn $service($($arg: $type,)* funs: &TardisFunsInst, ctx: &TardisContext) -> $ret {
                let inst = funs.init(ctx, true, conf_initializer::init_fun).await?;
                match inst.kind_code() {
                    #[cfg(feature = "spi-pg")]
                    spi_constants::SPI_PG_KIND_CODE => pg::$service($($arg,)* funs, ctx, inst).await,
                    kind_code => Err(funs.bs_not_implemented(kind_code)),
                }
            }
        )*
    };
}

macro_rules! call {
    ($fun:ident, $funs:ident, $ctx:ident, $inst:ident, @args: {$($args: ident),*}) => {
        $fun($($arg,)* $funs, $ctx, $inst).await
    };
}
macro_rules! dispatch_function {
    (
        $service:ident,
        $funs:ident, $ctx:ident, $inst:ident, 
        @dispatch: {
            $(
                $(#[$attr:meta])*
                $code:pat=>$mod:path,
            )*
        },
        @args: $args: tt
    ) => {
        match $inst.kind_code() {
            $(
                $(#[$attr])*
                $code => call!($mod::$service, $funs, $ctx, $inst, @args: $args)),
            )*
            kind_code => Err($funs.bs_not_implemented(kind_code)),
        }
        
    };
}
macro_rules! dispatch_service2 {
    (
        // mgr
        $mgr: expr,
        // init fun
        $init: expr,
        // dispacher
        @dispatch: $dispatch:tt,
        @method: {
            $(
                $(#[$attr:meta])*
                $service:ident($($arg: ident: $type: ty),*) -> $ret:ty;
            )*
        }

    ) => {
        $(
            $(#[$attr])*
            pub async fn $service($($arg: $type,)* funs: &TardisFunsInst, ctx: &TardisContext) -> $ret {
                let inst = funs.init(ctx, $mgr, $init).await?;
                dispatch_function!($service, funs, ctx, inst, @dispatch: $dispatch, @args: {$($arg),*})
            }
        )*
    };
}

dispatch_service2! {
    true,
    conf_initializer::init_fun,
    @dispatch: {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg,
    },
    @method: {
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

}

lazy_static::lazy_static! {
    static ref TOKEN_CTX_MAP: Arc<RwLock<BTreeMap<String, (TardisContext, Instant)>>> = Default::default();
    static ref MAP_CLEANER_TASK: OnceCell<JoinHandle<()>> = Default::default();
}

/// register a cert for nacos
pub async fn register(req: RegisterRequest, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<RegisterResponse> {
    const GEN_AK_MAX_RETRY: usize = 8;
    let ak = req.ak();
    let is_using_generated_ak = ak.is_none();
    let rand_ak = random_ak();
    let rand_sk = random_sk();
    let ak = ak.unwrap_or(rand_ak.as_str());
    let sk = req.sk().unwrap_or(rand_sk.as_str());
    // find backend spi
    let spi_bs = SpiBsServ::find_one_item(
        &SpiBsFilterReq {
            basic: RbumBasicFilterReq {
                enabled: Some(true),
                ..Default::default()
            },
            rel: Some(RbumItemRelFilterReq {
                rel_by_from: true,
                tag: Some(SPI_IDENT_REL_TAG.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Item),
                rel_item_id: Some(ctx.owner.clone()),
                ..Default::default()
            }),
            kind_code: None,
            domain_code: Some(funs.module_code().to_string()),
            ..Default::default()
        },
        funs,
        ctx,
    )
    .await?
    .ok_or_else(|| funs.err().not_found(&SpiBsServ::get_obj_name(), "register", "not found backend service", "404-spi-bs-not-exist"))?;
    // check if exist
    let mut find_filter = RbumCertFilterReq {
        kind: Some(String::from(SPI_CONF_CERT_KIND)),
        ak: Some(ak.to_string()),
        ..Default::default()
    };
    // conf cert using another context
    let conf_cert_ctx = TardisContext::default();
    if is_using_generated_ak {
        let mut retry = 1;
        while let Some(result) = RbumCertServ::find_one_rbum(&find_filter, funs, &conf_cert_ctx).await? {
            let conflict_ak = result.ak;
            let conflict_id = result.id;
            log::warn!("Random ak conflict on [{conflict_ak}] of id [{conflict_id}]");
            find_filter.ak = Some(random_ak());
            retry += 1;
            if retry > GEN_AK_MAX_RETRY {
                return Err(funs.err().conflict(
                    "spi-conf",
                    "register",
                    "Generate non-conclict username attempts exceed max retry limit",
                    EXCEED_MAX_RETRY_TIMES,
                ));
            }
        }
    } else if let Some(result) = RbumCertServ::find_one_rbum(&find_filter, funs, &conf_cert_ctx).await? {
        let conflict_ak = result.ak;
        let supplier = result.supplier;
        let owner = result.owner;
        let error_message = format!("conflict username [{conflict_ak}] owned by [{owner}] with supplier [{supplier}]");
        return Err(funs.err().conflict("spi-conf", "register", &error_message, CONLICT_AK));
    }
    // add a cert
    let ext = json!({
        "owner": ctx.owner,
        "owner_paths": ctx.own_paths
    })
    .to_string();
    let mut add_cert_req = RbumCertAddReq {
        kind: SPI_CONF_CERT_KIND.to_owned().into(),
        supplier: Some(ctx.owner.clone()),
        ak: ak.to_owned().into(),
        sk: Some(sk.to_owned().into()),
        is_ignore_check_sk: false,
        vcode: None,
        ext: Some(ext),
        start_time: None,
        end_time: None,
        conn_uri: Some(spi_bs.conn_uri),
        status: RbumCertStatusKind::Enabled,
        rel_rbum_cert_conf_id: None,
        rel_rbum_kind: RbumCertRelKind::Item,
        rel_rbum_id: spi_bs.id,
        is_outside: false,
    };
    RbumCertServ::add_rbum(&mut add_cert_req, funs, &conf_cert_ctx).await?;
    Ok(RegisterResponse::new(ak, sk))
}

/// convert ak and sk to corresponded tardis context
pub async fn auth(ak: &str, sk: &str, funs: &TardisFunsInst) -> TardisResult<TardisContext> {
    let find_filter = RbumCertFilterReq {
        kind: Some(String::from(SPI_CONF_CERT_KIND)),
        ak: Some(ak.to_string()),
        ..Default::default()
    };
    let valid_err = || funs.err().unauthorized("spi-conf", "auth", "validation error", "401-rbum-usrpwd-cert-valid-error");
    let mut ctx = TardisContext::default();
    let cert = RbumCertServ::find_one_rbum(&find_filter, funs, &ctx).await?.ok_or_else(valid_err)?;
    let real_sk = RbumCertServ::show_sk(&cert.id, &find_filter, funs, &ctx).await?;
    if sk != real_sk {
        return Err(valid_err());
    }
    let ext: serde_json::Value = serde_json::from_str(&cert.ext).map_err(|_| funs.err().internal_error("spi-conf", "auth", "invalid ext", "500-conf-invalid-cert-ext"))?;
    let owner = ext.get("owner").and_then(serde_json::Value::as_str).unwrap_or_default();
    let own_paths = ext.get("own_paths").and_then(serde_json::Value::as_str).unwrap_or_default();
    ctx.owner = owner.to_owned();
    ctx.own_paths = own_paths.to_owned();
    Ok(ctx)
}

/// bind a jwt token with a tardis context
async fn bind_token_ctx(token: &str, ttl: u64, ctx: &TardisContext) {
    TOKEN_CTX_MAP.write().await.insert(token.to_string(), (ctx.clone(), Instant::now() + Duration::from_secs(ttl)));
}

/// get the tardis context by jwt token
async fn get_ctx_by_token(token: &str) -> Option<TardisContext> {
    TOKEN_CTX_MAP.read().await.get(token).map(|(ctx, _exp)| ctx.clone())
}

/// init context-jwt map cleanner task
async fn init_map_cleaner_task() -> JoinHandle<()> {
    tardis::tokio::spawn(async move {
        let mut tick = interval(Duration::from_secs(1800));
        loop {
            let time = tick.tick().await;
            TOKEN_CTX_MAP.write().await.retain(|_, (_, exp)| *exp > time);
        }
    })
}

/// sign a jwt for a tardis context
pub async fn jwt_sign(funs: &TardisFunsInst, ctx: &TardisContext) -> poem::Result<String> {
    use jsonwebtoken::*;
    let cfg = funs.conf::<ConfConfig>();
    let ttl = cfg.token_ttl as u64;
    let claim = NacosJwtClaim::gen(ttl, &cfg.auth_username);
    MAP_CLEANER_TASK.get_or_init(init_map_cleaner_task).await;
    let key =
        EncodingKey::from_base64_secret(&cfg.auth_key).map_err(|_| poem::Error::from_string("spi-conf nacosmocker using an invalid authkey", StatusCode::INTERNAL_SERVER_ERROR))?;

    let token = encode(&Header::new(Algorithm::HS256), &claim, &key)
        .map_err(|_| poem::Error::from_string("spi-conf nacosmocker fail to encode auth token", StatusCode::INTERNAL_SERVER_ERROR))?;
    bind_token_ctx(&token, ttl, ctx).await;
    Ok(token)
}

/// sign validate jwt, return context if valid
pub async fn jwt_validate(token: &str, funs: &TardisFunsInst) -> poem::Result<TardisContext> {
    use jsonwebtoken::*;
    let cfg = funs.conf::<ConfConfig>();
    let mut validation = Validation::new(Algorithm::HS256);
    validation.sub = Some(cfg.auth_username.clone());
    let key =
        DecodingKey::from_base64_secret(&cfg.auth_key).map_err(|_| poem::Error::from_string("spi-conf nacosmocker using an invalid authkey", StatusCode::INTERNAL_SERVER_ERROR))?;
    let _ = decode::<NacosJwtClaim>(token, &key, &validation).map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::FORBIDDEN))?;
    if let Some(ctx) = get_ctx_by_token(token).await {
        Ok(ctx)
    } else {
        Err(poem::Error::from_string("Unknown token", StatusCode::FORBIDDEN))
    }
}
