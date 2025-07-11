use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    db::reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
};

use bios_basic::spi::{spi_funs::TypedSpiBsInst, spi_initializer};

pub async fn init_table_and_conn(bs_inst: TypedSpiBsInst<'_, TardisRelDBClient>, tag: &str, ctx: &TardisContext, mgr: bool) -> TardisResult<(TardisRelDBlConnection, String)> {
    spi_initializer::common_pg::init_table_and_conn(
        bs_inst,
        ctx,
        mgr,
        Some(tag),
        crate::log_constants::TABLE_LOG_FLAG,
        r#"ts timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    id character varying NOT NULL,
    key character varying NOT NULL,
    op character varying NOT NULL,
    content text NOT NULL,
    kind character varying[] NOT NULL,
    data_source character varying NOT NULL,
    owner character varying NOT NULL,
    own_paths character varying NOT NULL,
    ext jsonb NOT NULL,
    rel_key character varying NOT NULL"#,
        None,
        vec![
            ("kind", "gin"),
            ("ts", "btree"),
            ("key", "btree"),
            ("op", "btree"),
            ("ext", "gin"),
            ("data_source", "btree"),
            ("owner", "btree"),
            ("own_paths", "btree"),
            ("rel_key", "btree"),
            ("id", "btree"),
        ],
        None,
        None,
    )
    .await
}
