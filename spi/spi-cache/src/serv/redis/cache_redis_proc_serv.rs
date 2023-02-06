use std::collections::HashMap;

use bios_basic::spi::{spi_funs::SpiBsInstExtractor, spi_initializer::common};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    cache::cache_client::TardisCacheClient,
    TardisFunsInst,
};

use crate::dto::cache_proc_dto::{ExpReq, KIncrReq, KReq, KbRagngeReq, KbReq, KbvReq, KfIncrReq, KfReq, KfvReq, KvReq, KvWithExReq};

fn format_key(req_key: &str, ext: &HashMap<String, String>) -> String {
    if let Some(key_prefix) = common::get_isolation_flag_from_ext(ext) {
        format!("{key_prefix}{req_key}" )
    } else {
        req_key.to_string()
    }
}

pub async fn set(req: &KvReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisCacheClient>();
    Ok(bs_inst.0.set(&format_key(&req.key, bs_inst.1), &req.value).await?)
}

pub async fn set_ex(req: &KvWithExReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisCacheClient>();
    Ok(bs_inst.0.set_ex(&format_key(&req.key, bs_inst.1), &req.value, req.exp_sec as usize).await?)
}

pub async fn set_nx(req: &KvReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisCacheClient>();
    Ok(bs_inst.0.set_nx(&format_key(&req.key, bs_inst.1), &req.value).await?)
}

pub async fn get(req: &KReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<String>> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisCacheClient>();
    Ok(bs_inst.0.get(&format_key(&req.key, bs_inst.1)).await?)
}

pub async fn getset(req: &KvReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<String>> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisCacheClient>();
    Ok(bs_inst.0.getset(&format_key(&req.key, bs_inst.1), &req.value).await?)
}

pub async fn incr(req: &KIncrReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<i64> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisCacheClient>();
    Ok(bs_inst.0.incr(&format_key(&req.key, bs_inst.1), req.delta as isize).await? as i64)
}

pub async fn del(req: &KReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisCacheClient>();
    Ok(bs_inst.0.del(&format_key(&req.key, bs_inst.1)).await?)
}

pub async fn exists(req: &KReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisCacheClient>();
    Ok(bs_inst.0.exists(&format_key(&req.key, bs_inst.1)).await?)
}

pub async fn expire(req: &ExpReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisCacheClient>();
    Ok(bs_inst.0.expire(&format_key(&req.key, bs_inst.1), req.exp_sec as usize).await?)
}

pub async fn ttl(req: &KReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u64> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisCacheClient>();
    Ok(bs_inst.0.ttl(&format_key(&req.key, bs_inst.1)).await? as u64)
}

// list operations

pub async fn lpush(req: &KvReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisCacheClient>();
    Ok(bs_inst.0.lpush(&format_key(&req.key, bs_inst.1), &req.value).await?)
}

pub async fn lrangeall(req: &KReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<String>> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisCacheClient>();
    Ok(bs_inst.0.lrangeall(&format_key(&req.key, bs_inst.1)).await?)
}

pub async fn llen(req: &KReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u64> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisCacheClient>();
    Ok(bs_inst.0.llen(&format_key(&req.key, bs_inst.1)).await? as u64)
}

// hash operations

pub async fn hget(req: &KfReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<String>> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisCacheClient>();
    Ok(bs_inst.0.hget(&format_key(&req.key, bs_inst.1), &req.field).await?)
}

pub async fn hset(req: &KfvReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisCacheClient>();
    Ok(bs_inst.0.hset(&format_key(&req.key, bs_inst.1), &req.field, &req.value).await?)
}

pub async fn hset_nx(req: &KfvReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisCacheClient>();
    Ok(bs_inst.0.hset_nx(&format_key(&req.key, bs_inst.1), &req.field, &req.value).await?)
}

pub async fn hdel(req: &KfReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisCacheClient>();
    Ok(bs_inst.0.hdel(&format_key(&req.key, bs_inst.1), &req.field).await?)
}

pub async fn hincr(req: &KfIncrReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<i64> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisCacheClient>();
    Ok(bs_inst.0.hincr(&format_key(&req.key, bs_inst.1), &req.field, req.delta as isize).await? as i64)
}

pub async fn hexists(req: &KfReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisCacheClient>();
    Ok(bs_inst.0.hexists(&format_key(&req.key, bs_inst.1), &req.field).await?)
}

pub async fn hkeys(req: &KReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<String>> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisCacheClient>();
    Ok(bs_inst.0.hkeys(&format_key(&req.key, bs_inst.1)).await?)
}

pub async fn hvals(req: &KReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<String>> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisCacheClient>();
    Ok(bs_inst.0.hvals(&format_key(&req.key, bs_inst.1)).await?)
}

pub async fn hgetall(req: &KReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<HashMap<String, String>> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisCacheClient>();
    Ok(bs_inst.0.hgetall(&format_key(&req.key, bs_inst.1)).await?)
}

pub async fn hlen(req: &KReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u64> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisCacheClient>();
    Ok(bs_inst.0.hlen(&format_key(&req.key, bs_inst.1)).await? as u64)
}

// bitmap operations

pub async fn setbit(req: &KbvReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisCacheClient>();
    Ok(bs_inst.0.setbit(&format_key(&req.key, bs_inst.1), req.offset as usize, req.value).await?)
}

pub async fn getbit(req: &KbReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisCacheClient>();
    Ok(bs_inst.0.getbit(&format_key(&req.key, bs_inst.1), req.offset as usize).await?)
}

pub async fn bitcount(req: &KReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u32> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisCacheClient>();
    Ok(bs_inst.0.bitcount(&format_key(&req.key, bs_inst.1)).await? as u32)
}

pub async fn bitcount_range_by_bit(req: &KbRagngeReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<u32> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisCacheClient>();
    Ok(bs_inst.0.bitcount_range_by_bit(&format_key(&req.key, bs_inst.1), req.start as usize, req.end as usize).await? as u32)
}
