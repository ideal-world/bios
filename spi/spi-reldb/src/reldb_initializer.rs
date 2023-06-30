use bios_basic::spi::{
    api::spi_ci_bs_api,
    dto::spi_bs_dto::SpiBsCertResp,
    spi_constants,
    spi_funs::{self, SpiBsInst, TypedSpiBsInst},
    spi_initializer,
};
use tardis::{
    basic::{dto::TardisContext, error::TardisError, result::TardisResult},
    db::reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
    serde_json::Value as JsonValue,
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
    let mut funs = crate::get_tardis_inst();
    let clean_interval_sec = funs.conf::<ReldbConfig>().tx_clean_interval_sec;
    bios_basic::rbum::rbum_initializer::init(funs.module_code(), funs.conf::<ReldbConfig>().rbum.clone()).await?;
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
    web_server.add_module(DOMAIN_CODE, (spi_ci_bs_api::SpiCiBsApi, reldb_ci_exec_api::ReldbCiExecApi), None).await;
    Ok(())
}

pub async fn init_fun(bs_cert: SpiBsCertResp, ctx: &TardisContext, _: bool) -> TardisResult<SpiBsInst> {
    let ext = TardisFuns::json.str_to_json(&bs_cert.ext)?;
    let compatible_type = TardisFuns::json.json_to_obj(ext.get("compatible_type").unwrap_or(&tardis::serde_json::Value::String("None".to_string())).clone())?;
    let client = TardisRelDBClient::init(
        &bs_cert.conn_uri,
        ext.get("max_connections").and_then(JsonValue::as_u64).ok_or(TardisError::bad_request(
            "Tardis context ext expect `max_connections` as an unsigned interger number",
            "400-spi-invalid-tardis-ctx",
        ))? as u32,
        ext.get("min_connections").and_then(JsonValue::as_u64).ok_or(TardisError::bad_request(
            "Tardis context ext expect `min_connections` as an unsigned interger number",
            "400-spi-invalid-tardis-ctx",
        ))? as u32,
        None,
        None,
        compatible_type,
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

pub async fn inst_conn(bs_inst: TypedSpiBsInst<'_, TardisRelDBClient>) -> TardisResult<TardisRelDBlConnection> {
    let conn = bs_inst.0.conn();
    match bs_inst.2 {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => serv::pg::reldb_pg_initializer::init_conn(conn, bs_inst.1).await,
        #[cfg(feature = "spi-mysql")]
        reldb_constants::SPI_MYSQL_KIND_CODE => serv::mysql::reldb_mysql_initializer::init_conn(conn, bs_inst.1).await,
        kind_code => Err(spi_funs::bs_not_implemented(kind_code))?,
    }
}

#[inline]
pub(crate) fn get_tardis_inst() -> TardisFunsInst {
    TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None)
}
