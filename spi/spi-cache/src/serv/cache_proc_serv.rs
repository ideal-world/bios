use std::collections::HashMap;

use bios_basic::spi::spi_funs::SpiBsInstExtractor;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::TardisFunsInst;

use crate::dto::cache_proc_dto::{ExpReq, KIncrReq, KReq, KbRagngeReq, KbReq, KbvReq, KfIncrReq, KfReq, KfvReq, KvReq, KvWithExReq};
use crate::{cache_constants, cache_initializer};

use super::redis;

pub async fn set(req: &KvReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.init(ctx, true, cache_initializer::init_fun).await?;
    match bs_inst.kind_code() {
        #[cfg(feature = "spi-redis")]
        cache_constants::SPI_REDIS_KIND_CODE => redis::cache_redis_proc_serv::set(req, funs, ctx, bs_inst).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn set_ex(req: &KvWithExReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    match funs.init(ctx, true, cache_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-redis")]
        cache_constants::SPI_REDIS_KIND_CODE => redis::cache_redis_proc_serv::set_ex(req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn set_nx(req: &KvReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
    match funs.init(ctx, true, cache_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-redis")]
        cache_constants::SPI_REDIS_KIND_CODE => redis::cache_redis_proc_serv::set_nx(req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn get(req: &KReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<String>> {
    match funs.init(ctx, true, cache_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-redis")]
        cache_constants::SPI_REDIS_KIND_CODE => redis::cache_redis_proc_serv::get(req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn getset(req: &KvReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<String>> {
    match funs.init(ctx, true, cache_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-redis")]
        cache_constants::SPI_REDIS_KIND_CODE => redis::cache_redis_proc_serv::getset(req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn incr(req: &KIncrReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<i64> {
    match funs.init(ctx, true, cache_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-redis")]
        cache_constants::SPI_REDIS_KIND_CODE => redis::cache_redis_proc_serv::incr(req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn del(req: &KReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    match funs.init(ctx, true, cache_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-redis")]
        cache_constants::SPI_REDIS_KIND_CODE => redis::cache_redis_proc_serv::del(req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn exists(req: &KReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
    match funs.init(ctx, true, cache_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-redis")]
        cache_constants::SPI_REDIS_KIND_CODE => redis::cache_redis_proc_serv::exists(req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn expire(req: &ExpReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    match funs.init(ctx, true, cache_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-redis")]
        cache_constants::SPI_REDIS_KIND_CODE => redis::cache_redis_proc_serv::expire(req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn ttl(req: &KReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u64> {
    match funs.init(ctx, true, cache_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-redis")]
        cache_constants::SPI_REDIS_KIND_CODE => redis::cache_redis_proc_serv::ttl(req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

// list operations

pub async fn lpush(req: &KvReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    match funs.init(ctx, true, cache_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-redis")]
        cache_constants::SPI_REDIS_KIND_CODE => redis::cache_redis_proc_serv::lpush(req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn lrangeall(req: &KReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<String>> {
    match funs.init(ctx, true, cache_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-redis")]
        cache_constants::SPI_REDIS_KIND_CODE => redis::cache_redis_proc_serv::lrangeall(req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn llen(req: &KReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u64> {
    match funs.init(ctx, true, cache_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-redis")]
        cache_constants::SPI_REDIS_KIND_CODE => redis::cache_redis_proc_serv::llen(req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

// hash operations

pub async fn hget(req: &KfReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<String>> {
    match funs.init(ctx, true, cache_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-redis")]
        cache_constants::SPI_REDIS_KIND_CODE => redis::cache_redis_proc_serv::hget(req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn hset(req: &KfvReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    match funs.init(ctx, true, cache_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-redis")]
        cache_constants::SPI_REDIS_KIND_CODE => redis::cache_redis_proc_serv::hset(req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn hset_nx(req: &KfvReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
    match funs.init(ctx, true, cache_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-redis")]
        cache_constants::SPI_REDIS_KIND_CODE => redis::cache_redis_proc_serv::hset_nx(req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn hdel(req: &KfReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    match funs.init(ctx, true, cache_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-redis")]
        cache_constants::SPI_REDIS_KIND_CODE => redis::cache_redis_proc_serv::hdel(req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn hincr(req: &KfIncrReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<i64> {
    match funs.init(ctx, true, cache_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-redis")]
        cache_constants::SPI_REDIS_KIND_CODE => redis::cache_redis_proc_serv::hincr(req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn hexists(req: &KfReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
    match funs.init(ctx, true, cache_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-redis")]
        cache_constants::SPI_REDIS_KIND_CODE => redis::cache_redis_proc_serv::hexists(req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn hkeys(req: &KReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<String>> {
    match funs.init(ctx, true, cache_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-redis")]
        cache_constants::SPI_REDIS_KIND_CODE => redis::cache_redis_proc_serv::hkeys(req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn hvals(req: &KReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<String>> {
    match funs.init(ctx, true, cache_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-redis")]
        cache_constants::SPI_REDIS_KIND_CODE => redis::cache_redis_proc_serv::hvals(req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn hgetall(req: &KReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<HashMap<String, String>> {
    match funs.init(ctx, true, cache_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-redis")]
        cache_constants::SPI_REDIS_KIND_CODE => redis::cache_redis_proc_serv::hgetall(req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn hlen(req: &KReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u64> {
    match funs.init(ctx, true, cache_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-redis")]
        cache_constants::SPI_REDIS_KIND_CODE => redis::cache_redis_proc_serv::hlen(req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

// bitmap operations

pub async fn setbit(req: &KbvReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
    match funs.init(ctx, true, cache_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-redis")]
        cache_constants::SPI_REDIS_KIND_CODE => redis::cache_redis_proc_serv::setbit(req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn getbit(req: &KbReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
    match funs.init(ctx, true, cache_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-redis")]
        cache_constants::SPI_REDIS_KIND_CODE => redis::cache_redis_proc_serv::getbit(req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn bitcount(req: &KReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u32> {
    match funs.init(ctx, true, cache_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-redis")]
        cache_constants::SPI_REDIS_KIND_CODE => redis::cache_redis_proc_serv::bitcount(req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn bitcount_range_by_bit(req: &KbRagngeReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u32> {
    match funs.init(ctx, true, cache_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-redis")]
        cache_constants::SPI_REDIS_KIND_CODE => redis::cache_redis_proc_serv::bitcount_range_by_bit(req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}
