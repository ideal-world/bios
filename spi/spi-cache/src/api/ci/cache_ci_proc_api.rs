use std::collections::HashMap;

use bios_basic::spi::spi_funs::SpiTardisFunInstExtractor;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use crate::dto::cache_proc_dto::{ExpReq, KIncrReq, KReq, KbRagngeReq, KbReq, KbvReq, KfIncrReq, KfReq, KfvReq, KvReq, KvWithExReq};
use crate::serv::cache_proc_serv;

pub struct CacheCiProcApi;

/// Interface Console Cache API
#[poem_openapi::OpenApi(prefix_path = "/ci/proc", tag = "bios_basic::ApiTag::Interface")]
impl CacheCiProcApi {
    /// set
    #[oai(path = "/set", method = "put")]
    async fn set(&self, req: Json<KvReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        cache_proc_serv::set(&req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// set_ex
    #[oai(path = "/set_ex", method = "post")]
    async fn set_ex(&self, req: Json<KvWithExReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        cache_proc_serv::set_ex(&req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// set_nx
    #[oai(path = "/set_nx", method = "put")]
    async fn set_nx(&self, req: Json<KvReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<bool> {
        let funs = request.tardis_fun_inst();
        TardisResp::ok(cache_proc_serv::set_nx(&req.0, &funs, &ctx.0).await?)
    }

    /// get
    #[oai(path = "/get", method = "put")]
    async fn get(&self, req: Json<KReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Option<String>> {
        let funs = request.tardis_fun_inst();
        TardisResp::ok(cache_proc_serv::get(&req.0, &funs, &ctx.0).await?)
    }

    /// getset
    #[oai(path = "/getset", method = "put")]
    async fn getset(&self, req: Json<KvReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Option<String>> {
        let funs = request.tardis_fun_inst();
        TardisResp::ok(cache_proc_serv::getset(&req.0, &funs, &ctx.0).await?)
    }

    /// incr
    #[oai(path = "/incr", method = "post")]
    async fn incr(&self, req: Json<KIncrReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<i64> {
        let funs = request.tardis_fun_inst();
        TardisResp::ok(cache_proc_serv::incr(&req.0, &funs, &ctx.0).await?)
    }

    /// del
    #[oai(path = "/del", method = "put")]
    async fn del(&self, req: Json<KReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        cache_proc_serv::del(&req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// exists
    #[oai(path = "/exists", method = "put")]
    async fn exists(&self, req: Json<KReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<bool> {
        let funs = request.tardis_fun_inst();
        TardisResp::ok(cache_proc_serv::exists(&req.0, &funs, &ctx.0).await?)
    }

    /// expire
    #[oai(path = "/expire", method = "post")]
    async fn expire(&self, req: Json<ExpReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        cache_proc_serv::expire(&req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// ttl
    #[oai(path = "/ttl", method = "put")]
    async fn ttl(&self, req: Json<KReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<u64> {
        let funs = request.tardis_fun_inst();
        TardisResp::ok(cache_proc_serv::ttl(&req.0, &funs, &ctx.0).await?)
    }

    /// lpush
    #[oai(path = "/lpush", method = "put")]
    async fn lpush(&self, req: Json<KvReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        cache_proc_serv::lpush(&req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// lrangeall
    #[oai(path = "/lrangeall", method = "put")]
    async fn lrangeall(&self, req: Json<KReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Vec<String>> {
        let funs = request.tardis_fun_inst();
        TardisResp::ok(cache_proc_serv::lrangeall(&req.0, &funs, &ctx.0).await?)
    }

    /// llen
    #[oai(path = "/llen", method = "put")]
    async fn llen(&self, req: Json<KReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<u64> {
        let funs = request.tardis_fun_inst();
        TardisResp::ok(cache_proc_serv::llen(&req.0, &funs, &ctx.0).await?)
    }

    /// hget
    #[oai(path = "/hget", method = "put")]
    async fn hget(&self, req: Json<KfReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Option<String>> {
        let funs = request.tardis_fun_inst();
        TardisResp::ok(cache_proc_serv::hget(&req.0, &funs, &ctx.0).await?)
    }

    /// hset
    #[oai(path = "/hset", method = "put")]
    async fn hset(&self, req: Json<KfvReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        cache_proc_serv::hset(&req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// hset_nx
    #[oai(path = "/hset_nx", method = "put")]
    async fn hset_nx(&self, req: Json<KfvReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<bool> {
        let funs = request.tardis_fun_inst();
        TardisResp::ok(cache_proc_serv::hset_nx(&req.0, &funs, &ctx.0).await?)
    }

    /// hdel
    #[oai(path = "/hdel", method = "put")]
    async fn hdel(&self, req: Json<KfReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        cache_proc_serv::hdel(&req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// hincr
    #[oai(path = "/hincr", method = "post")]
    async fn hincr(&self, req: Json<KfIncrReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<i64> {
        let funs = request.tardis_fun_inst();
        TardisResp::ok(cache_proc_serv::hincr(&req.0, &funs, &ctx.0).await?)
    }

    /// hexists
    #[oai(path = "/hexists", method = "put")]
    async fn hexists(&self, req: Json<KfReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<bool> {
        let funs = request.tardis_fun_inst();
        TardisResp::ok(cache_proc_serv::hexists(&req.0, &funs, &ctx.0).await?)
    }

    /// hkeys
    #[oai(path = "/hkeys", method = "put")]
    async fn hkeys(&self, req: Json<KReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Vec<String>> {
        let funs = request.tardis_fun_inst();
        TardisResp::ok(cache_proc_serv::hkeys(&req.0, &funs, &ctx.0).await?)
    }

    /// hvals
    #[oai(path = "/hvals", method = "put")]
    async fn hvals(&self, req: Json<KReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Vec<String>> {
        let funs = request.tardis_fun_inst();
        TardisResp::ok(cache_proc_serv::hvals(&req.0, &funs, &ctx.0).await?)
    }

    /// hgetall
    #[oai(path = "/hgetall", method = "put")]
    async fn hgetall(&self, req: Json<KReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<HashMap<String, String>> {
        let funs = request.tardis_fun_inst();
        TardisResp::ok(cache_proc_serv::hgetall(&req.0, &funs, &ctx.0).await?)
    }

    /// hlen
    #[oai(path = "/hlen", method = "put")]
    async fn hlen(&self, req: Json<KReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<u64> {
        let funs = request.tardis_fun_inst();
        TardisResp::ok(cache_proc_serv::hlen(&req.0, &funs, &ctx.0).await?)
    }

    /// setbit
    #[oai(path = "/setbit", method = "put")]
    async fn setbit(&self, req: Json<KbvReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<bool> {
        let funs = request.tardis_fun_inst();
        TardisResp::ok(cache_proc_serv::setbit(&req.0, &funs, &ctx.0).await?)
    }

    /// getbit
    #[oai(path = "/getbit", method = "put")]
    async fn getbit(&self, req: Json<KbReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<bool> {
        let funs = request.tardis_fun_inst();
        TardisResp::ok(cache_proc_serv::getbit(&req.0, &funs, &ctx.0).await?)
    }

    /// bitcount
    #[oai(path = "/bitcount", method = "put")]
    async fn bitcount(&self, req: Json<KReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<u32> {
        let funs = request.tardis_fun_inst();
        TardisResp::ok(cache_proc_serv::bitcount(&req.0, &funs, &ctx.0).await?)
    }

    /// bitcount_range_by_bit
    #[oai(path = "/bitcount_range_by_bit", method = "put")]
    async fn bitcount_range_by_bit(&self, req: Json<KbRagngeReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<u32> {
        let funs = request.tardis_fun_inst();
        TardisResp::ok(cache_proc_serv::bitcount_range_by_bit(&req.0, &funs, &ctx.0).await?)
    }
}
