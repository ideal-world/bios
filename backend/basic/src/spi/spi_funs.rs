//! SPI backend service instance initialization and extraction
//! SPI后端服务实例初始化和提取
use async_trait::async_trait;
use std::any::Any;
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::sync::OnceLock;
use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::tokio::sync::RwLock;
use tardis::TardisFuns;
use tardis::TardisFunsInst;

use crate::spi::dto::spi_bs_dto::SpiBsCertResp;

use super::serv::spi_bs_serv::SpiBsServ;
use super::spi_constants;

pub struct SpiBsInst {
    pub client: Box<dyn Any + Send + Sync>,
    pub ext: HashMap<String, String>,
}

pub type TypedSpiBsInst<'a, T> = (&'a T, &'a HashMap<String, String>, &'a str);

impl SpiBsInst {
    pub fn inst<T>(&self) -> TypedSpiBsInst<'_, T>
    // T is 'static meaning it's an owned type or only holds static references
    // dyn Any + Send + Sync ==downcast==> T: 'static + Send + Sync
    where
        T: 'static,
    {
        let c = self.client.as_ref().downcast_ref::<T>().expect("ignore");
        (c, &self.ext, self.kind_code())
    }

    pub fn kind_code(&self) -> &str {
        self.ext.get(spi_constants::SPI_KIND_CODE_FLAG).expect("ignore")
    }
}

fn get_spi_bs_caches() -> &'static RwLock<HashMap<String, Arc<SpiBsInst>>> {
    static SPI_BS_CACHES: OnceLock<RwLock<HashMap<String, Arc<SpiBsInst>>>> = OnceLock::new();
    SPI_BS_CACHES.get_or_init(Default::default)
}

#[async_trait]
pub trait SpiBsInstExtractor {
    async fn init<'a, F, T>(&self, custom_cache_key: Option<String>, ctx: &'a TardisContext, mgr: bool, init_funs: F) -> TardisResult<Arc<SpiBsInst>>
    where
        F: Fn(SpiBsCertResp, &'a TardisContext, bool) -> T + Send + Sync,
        T: Future<Output = TardisResult<SpiBsInst>> + Send;

    async fn bs<'a>(&self, ctx: &'a TardisContext) -> TardisResult<Arc<SpiBsInst>>;

    async fn init_bs<'a, F, T>(&self, ctx: &'a TardisContext, mgr: bool, init_funs: F) -> TardisResult<Arc<SpiBsInst>>
    where
        F: Fn(SpiBsCertResp, &'a TardisContext, bool) -> T + Send + Sync,
        T: Future<Output = TardisResult<SpiBsInst>> + Send;

    fn bs_not_implemented(&self, bs_code: &str) -> TardisError;

    async fn remove_bs_inst_cache<'a>(&self, ctx: &'a TardisContext) -> TardisResult<()>;
}

#[async_trait]
impl SpiBsInstExtractor for TardisFunsInst {
    /// Initialize the backend service instance
    ///
    /// # Arguments
    ///
    /// * `custom_cache_key` - Customize the cache key, if it is none, the default cache key will be used.
    /// * `ctx` - Request Context
    /// * `mgr` - Whether it is a managed request
    /// * `init_fun` - The initialization function called when the backend service instance is not initialized
    ///
    /// # Return
    ///
    /// the backend service instance kind
    async fn init<'a, F, T>(&self, custom_cache_key: Option<String>, ctx: &'a TardisContext, mgr: bool, init_fun: F) -> TardisResult<Arc<SpiBsInst>>
    where
        F: Fn(SpiBsCertResp, &'a TardisContext, bool) -> T + Send + Sync,
        T: Future<Output = TardisResult<SpiBsInst>> + Send,
    {
        let cache_key = if let Some(custom_cache_key) = custom_cache_key {
            format!("{}-{}-{}", self.module_code(), ctx.ak, custom_cache_key)
        } else {
            format!("{}-{}", self.module_code(), ctx.ak)
        };
        {
            let read = get_spi_bs_caches().read().await;
            if let Some(inst) = read.get(&cache_key).cloned() {
                return Ok(inst);
            }
        }
        let spi_bs = SpiBsServ::get_bs_by_rel(&ctx.ak, None, self, ctx).await?;
        info!(
            "[SPI] Init and cache backend service instance [{}]:{}",
            cache_key.clone(),
            TardisFuns::json.obj_to_string(&spi_bs)?
        );
        let kind_code = spi_bs.kind_code.clone();
        let mut spi_bs_inst = init_fun(spi_bs, ctx, mgr).await?;
        {
            let mut write = get_spi_bs_caches().write().await;
            spi_bs_inst.ext.insert(spi_constants::SPI_KIND_CODE_FLAG.to_string(), kind_code);
            return Ok(write.entry(cache_key.clone()).or_insert(Arc::new(spi_bs_inst)).clone());
        }
    }

    /// Fetch the backend service instance
    ///
    /// # Arguments
    ///
    /// * `ctx` - Request Context
    ///
    /// # Return
    ///
    /// the backend service instance
    async fn bs<'a>(&self, ctx: &'a TardisContext) -> TardisResult<Arc<SpiBsInst>> {
        let cache_key = format!("{}-{}", self.module_code(), ctx.ak);
        Ok(get_spi_bs_caches().read().await.get(&cache_key).expect("ignore").clone())
    }

    /// Initialize the backend service instance and fetch it
    ///
    /// # Arguments
    ///
    /// * `ctx` - Request Context
    /// * `mgr` - Whether it is a managed request
    /// * `init_fun` - The initialization function called when the backend service instance is not initialized
    ///
    /// # Return
    ///
    /// the backend service instance
    async fn init_bs<'a, F, T>(&self, ctx: &'a TardisContext, mgr: bool, init_fun: F) -> TardisResult<Arc<SpiBsInst>>
    where
        F: Fn(SpiBsCertResp, &'a TardisContext, bool) -> T + Send + Sync,
        T: Future<Output = TardisResult<SpiBsInst>> + Send,
    {
        self.init(None, ctx, mgr, init_fun).await?;
        self.bs(ctx).await
    }

    fn bs_not_implemented(&self, bs_code: &str) -> TardisError {
        bs_not_implemented(bs_code)
    }

    async fn remove_bs_inst_cache<'a>(&self, ctx: &'a TardisContext) -> TardisResult<()> {
        let cache_key = format!("{}-{}", self.module_code(), ctx.ak);
        info!("[SPI] Remove bs inst cache key:{}", cache_key.clone());
        let mut write = get_spi_bs_caches().write().await;
        write.remove(&cache_key);
        Ok(())
    }
}

pub fn bs_not_implemented(bs_code: &str) -> TardisError {
    TardisError::not_implemented(
        &format!("Backend service kind {bs_code} does not exist or SPI feature is not enabled"),
        "406-rbum-*-enum-init-error",
    )
}
