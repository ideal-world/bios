use std::collections::HashMap;

use bios_basic::spi::{
    api::spi_ci_bs_api,
    dto::spi_bs_dto::SpiBsCertResp,
    spi_constants,
    spi_funs::{self, SpiBsInst},
    spi_initializer,
};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    db::reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
    web::web_server::TardisWebServer,
    TardisFuns, TardisFunsInst,
};

use crate::{
    api::ci::reldb_ci_exec_api,
    reldb_config::ReldbConfig,
    reldb_constants::{self, DOMAIN_CODE},
    serv::{self, reldb_exec_serv},
};

pub async fn init(web_server: &TardisWebServer) -> TardisResult<()> {
    let mut funs = TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None);
    let clean_interval_sec = funs.conf::<ReldbConfig>().tx_clean_interval_sec;

    funs.begin().await?;
    let ctx = spi_initializer::init(DOMAIN_CODE, &funs).await?;
    init_db(&funs, &ctx).await?;
    funs.commit().await?;
    init_api(web_server).await?;
    reldb_exec_serv::clean(clean_interval_sec).await;
    Ok(())
}

async fn init_db(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    spi_initializer::add_kind(spi_constants::SPI_PG_KIND_CODE, funs, ctx).await?;
    spi_initializer::add_kind(reldb_constants::SPI_MYSQL_KIND_CODE, funs, ctx).await?;
    Ok(())
}

async fn init_api(web_server: &TardisWebServer) -> TardisResult<()> {
    web_server.add_module(DOMAIN_CODE, (spi_ci_bs_api::SpiCiBsApi, reldb_ci_exec_api::ReldbCiExecApi)).await;
    Ok(())
}

pub async fn init_fun(bs_cert: SpiBsCertResp, ctx: &TardisContext, _: bool) -> TardisResult<SpiBsInst> {
    let ext = TardisFuns::json.str_to_json(&bs_cert.ext)?;
    let client = TardisRelDBClient::init(
        &bs_cert.conn_uri,
        ext.get("max_connections").unwrap().as_u64().unwrap() as u32,
        ext.get("min_connections").unwrap().as_u64().unwrap() as u32,
        None,
        None,
    )
    .await?;
    let ext = match bs_cert.kind_code.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => serv::pg::reldb_pg_initializer::init(&bs_cert, &client, ctx).await?,
        #[cfg(feature = "spi-mysql")]
        reldb_constants::SPI_MYSQL_KIND_CODE => serv::mysql::reldb_mysql_initializer::init(&bs_cert, &client, ctx).await?,
        _ => Err(bs_cert.bs_not_implemented())?,
    };
    Ok(SpiBsInst { client: Box::new(client), ext })
}

pub async fn inst_conn(bs_inst: (&TardisRelDBClient, &HashMap<String, String>, String)) -> TardisResult<TardisRelDBlConnection> {
    let conn = bs_inst.0.conn();
    match bs_inst.2.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => serv::pg::reldb_pg_initializer::init_conn(conn, bs_inst.1).await,
        #[cfg(feature = "spi-mysql")]
        reldb_constants::SPI_MYSQL_KIND_CODE => serv::mysql::reldb_mysql_initializer::init_conn(conn, bs_inst.1).await,
        kind_code => Err(spi_funs::bs_not_implemented(kind_code))?,
    }
}
