use async_trait::async_trait;
use std::any::Any;
use std::collections::HashMap;
use std::future::Future;
use std::ptr::replace;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::TardisFuns;
use tardis::TardisFunsInst;

use crate::spi::dto::spi_bs_dto::SpiBsCertResp;

use super::serv::spi_bs_serv::SpiBsServ;
use super::spi_constants;

pub trait SpiTardisFunInstExtractor {
    fn tardis_fun_inst(&self) -> TardisFunsInst;
}

#[cfg(feature = "default")]
impl SpiTardisFunInstExtractor for tardis::web::poem::Request {
    fn tardis_fun_inst(&self) -> TardisFunsInst {
        let serv_domain = self.original_uri().path().split('/').collect::<Vec<&str>>()[1];
        TardisFuns::inst_with_db_conn(serv_domain.to_string(), None)
    }
}

pub struct SpiBsInst {
    pub client: Box<dyn Any + Send>,
    pub ext: HashMap<String, String>,
}

impl SpiBsInst {
    pub fn inst<T>(&'static self) -> (&'static T, &'static HashMap<String, String>, String) {
        let c = self.client.as_ref().downcast_ref::<T>().unwrap();
        (c, &self.ext, self.kind_code())
    }

    pub fn kind_code(&self) -> String {
        self.ext.get(spi_constants::SPI_KIND_CODE_FLAG).unwrap().to_string()
    }
}

static mut SPI_BS_CACHES: Option<HashMap<String, SpiBsInst>> = None;

#[async_trait]
pub trait SpiBsInstExtractor {
    async fn bs<'a, F, T>(&self, ctx: &'a TardisContext, init_funs: F) -> TardisResult<&'static SpiBsInst>
    where
        F: Fn(SpiBsCertResp, &'a TardisContext) -> T + Send + Sync,
        T: Future<Output = TardisResult<SpiBsInst>> + Send;
}

#[async_trait]
impl SpiBsInstExtractor for TardisFunsInst {
    async fn bs<'a, F, T>(&self, ctx: &'a TardisContext, init_funs: F) -> TardisResult<&'static SpiBsInst>
    where
        F: Fn(SpiBsCertResp, &'a TardisContext) -> T + Send + Sync,
        T: Future<Output = TardisResult<SpiBsInst>> + Send,
    {
        let cache_key = format!("{}-{}", self.module_code(), ctx.own_paths);
        unsafe {
            if SPI_BS_CACHES.is_none() {
                replace(&mut SPI_BS_CACHES, Some(HashMap::new()));
            }
            match &mut SPI_BS_CACHES {
                None => panic!("[SPI] CACHE instance doesn't exist"),
                Some(caches) => {
                    if !caches.contains_key(&cache_key) {
                        let spi_bs = SpiBsServ::get_bs_by_rel(&ctx.owner, self, ctx).await?;
                        let kind_code = spi_bs.kind_code.clone();
                        let mut spi_bs_inst = init_funs(spi_bs, ctx).await?;
                        spi_bs_inst.ext.insert(spi_constants::SPI_KIND_CODE_FLAG.to_string(), kind_code);
                        caches.insert(cache_key.clone(), spi_bs_inst);
                    }
                    Ok(caches.get(&cache_key).unwrap())
                }
            }
        }
    }
}
