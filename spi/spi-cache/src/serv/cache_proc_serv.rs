use std::collections::HashMap;

use bios_basic::spi::spi_funs::SpiBsInstExtractor;

use tardis::basic::result::TardisResult;

use crate::dto::cache_proc_dto::*;
use crate::{cache_constants, cache_initializer};
use bios_basic::spi_dispatch_service;

use super::redis;
spi_dispatch_service! {
    @mgr: true,
    @init: cache_initializer::init_fun,
    @dispatch: {
        #[cfg(feature = "spi-redis")]
        cache_constants::SPI_REDIS_KIND_CODE => redis::cache_redis_proc_serv,
    },
    @method: {
        set(req: &KvReq) -> TardisResult<()>;
        set_ex(req: &KvWithExReq) -> TardisResult<()>;
        set_nx(req: &KvReq) -> TardisResult<bool>;
        get(req: &KReq) -> TardisResult<Option<String>>;
        getset(req: &KvReq) -> TardisResult<Option<String>>;
        incr(req: &KIncrReq) -> TardisResult<i64>;
        del(req: &KReq) -> TardisResult<()>;
        exists(req: &KReq) -> TardisResult<bool>;
        expire(req: &ExpReq) -> TardisResult<()>;
        ttl(req: &KReq) -> TardisResult<u64>;
        lpush(req: &KvReq) -> TardisResult<()>;
        lrangeall(req: &KReq) -> TardisResult<Vec<String>>;
        llen(req: &KReq) -> TardisResult<u64>;
        hget(req: &KfReq) -> TardisResult<Option<String>>;
        hset(req: &KfvReq) -> TardisResult<()>;
        hset_nx(req: &KfvReq) -> TardisResult<bool>;
        hdel(req: &KfReq) -> TardisResult<()>;
        hincr(req: &KfIncrReq) -> TardisResult<i64>;
        hexists(req: &KfReq) -> TardisResult<bool>;
        hkeys(req: &KReq) -> TardisResult<Vec<String>>;
        hvals(req: &KReq) -> TardisResult<Vec<String>>;
        hgetall(req: &KReq) -> TardisResult<HashMap<String, String>>;
        hlen(req: &KReq) -> TardisResult<u64>;
        setbit(req: &KbvReq) -> TardisResult<bool>;
        getbit(req: &KbReq) -> TardisResult<bool>;
        bitcount(req: &KReq) -> TardisResult<u32>;
        bitcount_range_by_bit(req: &KbRangeReq) -> TardisResult<u32>;
    }
}
