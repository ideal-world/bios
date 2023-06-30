use bios_basic::spi::spi_constants;
use bios_basic::spi::spi_funs::SpiBsInstExtractor;
use bios_basic::spi_dispatch_service;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::TardisFunsInst;

use crate::dto::graph_dto::{GraphNodeVersionResp, GraphRelAddReq, GraphRelDetailResp, GraphRelUpgardeVersionReq};
use crate::graph_initializer;

use super::pg;

spi_dispatch_service! {
    @mgr: true,
    @init: graph_initializer::init_fun,
    @dispatch: {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::graph_pg_basic_serv,
    },
    @method: {
        add_rel(add_req: &GraphRelAddReq) -> TardisResult<()>;
        upgrade_version(upgrade_version_req: &GraphRelUpgardeVersionReq) -> TardisResult<()>;
        find_versions(tag: String, key: String) -> TardisResult<Vec<GraphNodeVersionResp>>;
        find_rels(from_key: String, from_version: String, depth: Option<u8>) -> TardisResult<GraphRelDetailResp>;
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
    let inst = funs.init(ctx, true, graph_initializer::init_fun).await?;
    match inst.kind_code() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => pg::graph_pg_basic_serv::delete_rels(tag, from_key, to_key, from_version, to_version, funs, ctx, inst).await,
        kind_code => Err(funs.bs_not_implemented(kind_code)),
    }
}
