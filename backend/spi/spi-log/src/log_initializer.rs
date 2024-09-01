use bios_basic::spi::{api::spi_ci_bs_api, dto::spi_bs_dto::SpiBsCertResp, spi_constants, spi_funs::SpiBsInst, spi_initializer};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    log::info,
    web::web_server::TardisWebServer,
    TardisFuns, TardisFunsInst,
};

use crate::{
    api::ci::log_ci_item_api,
    log_config::LogConfig,
    log_constants::{self, CONFIG_TABLE_NAME, DOMAIN_CODE},
};

pub async fn init(web_server: &TardisWebServer) -> TardisResult<()> {
    info!("[BIOS.Log] Module initializing");
    let mut funs = crate::get_tardis_inst();
    bios_basic::rbum::rbum_initializer::init(funs.module_code(), funs.conf::<LogConfig>().rbum.clone()).await?;
    funs.begin().await?;
    let ctx = spi_initializer::init(DOMAIN_CODE, &funs).await?;
    init_db(&funs, &ctx).await?;
    funs.commit().await?;
    crate::event::handle_events().await?;
    init_api(web_server).await?;
    info!("[BIOS.Log] Module initialized");
    Ok(())
}

async fn init_db(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    spi_initializer::add_kind(spi_constants::SPI_PG_KIND_CODE, funs, ctx).await?;
    spi_initializer::add_kind(log_constants::SPI_PG_V2_KIND_CODE, funs, ctx).await?;
    //添加父表
    let schema_name = spi_initializer::common_pg::get_schema_name_from_context(ctx);
    funs.db()
        .execute_one(
            &format!(
                r#"CREATE TABLE IF NOT EXISTS {schema_name}.{}(
                    idempotent_id varchar NOT NULL,
                    ts            timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    key           varchar NOT NULL,
                    kind          varchar NOT NULL,
                    op            varchar NOT NULL,
                    content       jsonb NOT NULL,
                    owner         varchar NOT NULL,
                    own_paths     varchar NOT NULL,
                    rel_key       varchar NOT NULL,
                    ext           jsonb NOT NULL,
                    disable       boolean NOT NULL DEFAULT false,
                  );"#,
                log_constants::PARENT_TABLE_NAME
            ),
            vec![],
        )
        .await?;

    //添加配置表
    funs.db()
        .execute_one(
            &format!(
                r#"CREATE TABLE IF NOT EXISTS {schema_name}.{CONFIG_TABLE_NAME}(
                    table_name VARCHAR NOT NULL,
                    ref_field VARCHAR NOT NULL,
                  );"#
            ),
            vec![],
        )
        .await?;

    //添加配置表索引
    funs.db()
        .execute_one(
            &format!(
                r#"
      CREATE INDEX IF NOT EXISTS {CONFIG_TABLE_NAME}_index1 ON {schema_name}.{CONFIG_TABLE_NAME} USING btree (table_name);
      "#
            ),
            vec![],
        )
        .await?;
    Ok(())
}

async fn init_api(web_server: &TardisWebServer) -> TardisResult<()> {
    web_server.add_module(DOMAIN_CODE, (spi_ci_bs_api::SpiCiBsApi, log_ci_item_api::LogCiItemApi)).await;
    Ok(())
}

pub async fn init_fun(bs_cert: SpiBsCertResp, ctx: &TardisContext, mgr: bool) -> TardisResult<SpiBsInst> {
    info!("[BIOS.Log] Fun [{}]({}) initializing", bs_cert.kind_code, bs_cert.conn_uri);
    let inst = match bs_cert.kind_code.as_str() {
        #[cfg(feature = "spi-pg")]
        spi_constants::SPI_PG_KIND_CODE => spi_initializer::common_pg::init(&bs_cert, ctx, mgr).await,
        log_constants::SPI_PG_V2_KIND_CODE => spi_initializer::common_pg::init(&bs_cert, ctx, mgr).await,
        _ => Err(bs_cert.bs_not_implemented())?,
    }?;
    info!("[BIOS.Log] Fun [{}]({}) initialized", bs_cert.kind_code, bs_cert.conn_uri);
    Ok(inst)
}

#[inline]
pub(crate) fn get_tardis_inst() -> TardisFunsInst {
    TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None)
}
