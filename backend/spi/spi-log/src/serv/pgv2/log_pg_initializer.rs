use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    db::reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
};

use bios_basic::spi::{spi_funs::TypedSpiBsInst, spi_initializer};

use crate::log_constants::{self, CONFIG_TABLE_NAME};

pub async fn init_table_and_conn(bs_inst: TypedSpiBsInst<'_, TardisRelDBClient>, tag: &str, ctx: &TardisContext, mgr: bool) -> TardisResult<(TardisRelDBlConnection, String)> {
    //添加父表
    let schema_name = spi_initializer::common_pg::get_schema_name_from_context(ctx);
    bs_inst
        .0
        .conn()
        .execute_one(
            &format!(
                r#"CREATE TABLE IF NOT EXISTS {schema_name}.{}(
                  idempotent_id varchar NOT NULL,
                  ts            timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
                  key           varchar NOT NULL,
                  kind          varchar NOT NULL,
                  tag           varchar NOT NULL,
                  op            varchar NOT NULL,
                  content       jsonb NOT NULL,
                  owner         varchar NOT NULL,
                  owner_name    varchar NOT NULL,
                  own_paths     varchar NOT NULL,
                  push          boolean NOT NULL DEFAULT false,
                  rel_key       varchar NOT NULL,
                  ext           jsonb NOT NULL,
                  disable       boolean NOT NULL DEFAULT false,
                  msg           varchar NOT NULL
                );"#,
                log_constants::PARENT_TABLE_NAME
            ),
            vec![],
        )
        .await?;

    //添加配置表
    bs_inst
        .0
        .conn()
        .execute_one(
            &format!(
                r#"CREATE TABLE IF NOT EXISTS {schema_name}.{CONFIG_TABLE_NAME}(
                  table_name VARCHAR NOT NULL,
                  ref_field VARCHAR NOT NULL
                );"#
            ),
            vec![],
        )
        .await?;

    //添加配置表索引
    bs_inst
        .0
        .conn()
        .execute_one(
            &format!(
                r#"
    CREATE INDEX IF NOT EXISTS {CONFIG_TABLE_NAME}_index1 ON {schema_name}.{CONFIG_TABLE_NAME} USING btree (table_name);
    "#
            ),
            vec![],
        )
        .await?;

    spi_initializer::common_pg::init_table_and_conn(
        bs_inst,
        ctx,
        mgr,
        Some(tag),
        log_constants::TABLE_LOG_FLAG_V2,
        "",
        Some(format!("{schema_name}.{}", crate::log_constants::PARENT_TABLE_NAME)),
        vec![
            ("kind", "btree"),
            ("ts", "btree"),
            ("key", "btree"),
            ("content", "gin"),
            ("ext", "gin"),
            ("owner", "btree"),
            ("own_paths", "btree"),
            ("rel_key", "btree"),
            ("idempotent_id", "btree"),
            ("disable", "btree"),
            ("tag", "btree"),
            ("push", "btree"),
        ],
        None,
        None,
    )
    .await
}
