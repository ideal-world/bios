use bios_basic::spi::spi_constants;
use bios_basic::spi::spi_funs::SpiBsInstExtractor;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::TardisFunsInst;

use crate::dto::graph_dto::{GraphNodeVersionResp, GraphRelAddReq, GraphRelDetailResp, GraphRelUpgardeVersionReq};
use crate::graph_initializer;

use super::pg;

pub async fn add_rel(add_req: &GraphRelAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    match funs.init(ctx, true, graph_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::graph_pg_basic_serv::add_rel(add_req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn upgrade_version(upgrade_version_req: &GraphRelUpgardeVersionReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    match funs.init(ctx, true, graph_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::graph_pg_basic_serv::upgrade_version(upgrade_version_req, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn find_versions(tag: String, key: String, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<GraphNodeVersionResp>> {
    match funs.init(ctx, true, graph_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::graph_pg_basic_serv::find_versions(tag, key, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn find_rels(from_key: String, from_version: String, depth: Option<u8>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<GraphRelDetailResp> {
    match funs.init(ctx, true, graph_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::graph_pg_basic_serv::find_rels(from_key, from_version, depth, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}

pub async fn delete_rels(
    tag: String,
    from_key: Option<String>,
    to_key: Option<String>,
    from_version: Option<String>,
    to_version: Option<String>,
    funs: &TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<()> {
    if from_key.is_none() && to_key.is_none() {
        return Err(funs.err().bad_request(
            "spi-graph-rel",
            "delete-rel",
            "at least one of [from_key] and [to_key] cannot be empty",
            "400-spi-graph-key-require",
        ));
    }
    match funs.init(ctx, true, graph_initializer::init_fun).await?.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::graph_pg_basic_serv::delete_rels(tag, from_key, to_key, from_version, to_version, funs, ctx).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}
