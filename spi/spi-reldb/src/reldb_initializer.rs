use std::any::Any;

use bios_basic::spi::{api::spi_ci_bs_api, spi_initializer, dto::spi_bs_dto::SpiBsCertResp};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    web::web_server::TardisWebServer,
    TardisFuns, TardisFunsInst, db::reldb_client::TardisRelDBClient,
};

use crate::{api::ci::api::reldb_exec_api, reldb_constants::DOMAIN_CODE};

pub async fn init(web_server: &TardisWebServer) -> TardisResult<()> {
    let mut funs = TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None);
    funs.begin().await?;
    let ctx = spi_initializer::init(DOMAIN_CODE, &funs).await?;
    init_db(&funs, &ctx).await?;
    funs.commit().await?;
    init_api(web_server).await
}

async fn init_db(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    spi_initializer::add_kind("spi-pg", funs, ctx).await?;
    spi_initializer::add_kind("spi-mysql", funs, ctx).await?;
    Ok(())
}

async fn init_api(web_server: &TardisWebServer) -> TardisResult<()> {
    web_server.add_module(DOMAIN_CODE, (spi_ci_bs_api::SpiCiBsApi, reldb_exec_api::ReldbCiExecApi)).await;
    Ok(())
}

pub async fn init_fun(cert: SpiBsCertResp) -> TardisResult<Box<dyn Any + Send>> {
    let ext = TardisFuns::json.str_to_json(&cert.ext)?;
    let client = TardisRelDBClient::init(
        &cert.conn_uri,
        ext.get("max_connections").unwrap().as_u64().unwrap() as u32,
        ext.get("min_connections").unwrap().as_u64().unwrap() as u32,
        None,
        None,
    )
    .await?;
    Ok(Box::new(client))
}