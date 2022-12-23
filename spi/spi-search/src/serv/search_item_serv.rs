use bios_basic::spi::spi_constants;
use bios_basic::spi::spi_funs::SpiBsInstExtractor;
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::db::sea_orm::{FromQueryResult, Value};
use tardis::web::web_resp::TardisPage;
use tardis::{basic::dto::TardisContext, db::reldb_client::TardisRelDBClient};
use tardis::{serde_json, TardisFunsInst};

use crate::dto::search_item_dto::{SearchItemAddOrModifyReq, SearchItemQueryReq, SearchItemQueryResp};
use crate::search_initializer;

use super::pg;

pub struct SearchItemServ;

impl SearchItemServ {
    pub async fn add_or_modify(add_or_modify_req: &mut SearchItemAddOrModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let kind_code = funs.init(ctx, search_initializer::init_fun).await?;
        match kind_code.as_str() {
            #[cfg(feature = "spi-pg")]
            spi_constants::SPI_PG_KIND_CODE => pg::search_pg_item_serv::add_or_modify(add_or_modify_req, funs, ctx).await,
            _ => Err(TardisError::not_implemented(
                &format!("Backend service kind {} does not exist or SPI feature is not enabled", kind_code),
                "406-rbum-*-enum-init-error",
            )),
        }
    }

    pub async fn delete(tag: &str, key: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let kind_code = funs.init(ctx, search_initializer::init_fun).await?;
        match kind_code.as_str() {
            #[cfg(feature = "spi-pg")]
            spi_constants::SPI_PG_KIND_CODE => pg::search_pg_item_serv::delete(tag, key, funs, ctx).await,
            _ => Err(TardisError::not_implemented(
                &format!("Backend service kind {} does not exist or SPI feature is not enabled", kind_code),
                "406-rbum-*-enum-init-error",
            )),
        }
    }

    pub async fn query(query_req: &mut SearchItemQueryReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<TardisPage<SearchItemQueryResp>> {
        let kind_code = funs.init(ctx, search_initializer::init_fun).await?;
        match kind_code.as_str() {
            #[cfg(feature = "spi-pg")]
            spi_constants::SPI_PG_KIND_CODE => pg::search_pg_item_serv::query(query_req, funs, ctx).await,
            _ => Err(TardisError::not_implemented(
                &format!("Backend service kind {} does not exist or SPI feature is not enabled", kind_code),
                "406-rbum-*-enum-init-error",
            )),
        }
    }
}
