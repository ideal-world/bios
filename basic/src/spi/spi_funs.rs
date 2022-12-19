use async_trait::async_trait;
use std::collections::HashMap;
use std::ptr::replace;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::TardisFuns;
use tardis::TardisFunsInst;

use crate::spi::dto::spi_bs_dto::SpiBsCertResp;

use super::serv::spi_bs_serv::SpiBsServ;

pub trait SpiTardisFunInstExtractor {
    fn tardis_fun_inst(&self) -> TardisFunsInst;
}

#[cfg(feature = "default")]
impl SpiTardisFunInstExtractor for tardis::web::poem::Request {
    fn tardis_fun_inst(&self) -> TardisFunsInst {
        let serv_domain = self.uri().path().split('/').collect::<Vec<&str>>()[0];
        TardisFuns::inst_with_db_conn(serv_domain.to_string(), None)
    }
}

static mut SPI_BS_CACHES: Option<HashMap<String, SpiBsCertResp>> = None;

pub trait SpiBsInst {}

#[async_trait]
pub trait SpiBsInstExtractor {
    async fn bs(&self, ctx: &TardisContext) -> TardisResult<Option<&'static SpiBsCertResp>>;
}

#[async_trait]
impl SpiBsInstExtractor for TardisFunsInst {
    async fn bs(&self, ctx: &TardisContext) -> TardisResult<Option<&'static SpiBsCertResp>> {
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
                        caches.insert(cache_key.clone(), spi_bs);
                    }
                    Ok(caches.get(&cache_key))
                }
            }
        }
    }
}
