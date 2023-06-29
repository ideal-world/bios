use std::collections::HashMap;

use bios_basic::spi::spi_initializer;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    db::reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
};

pub async fn init_table_and_conn(
    bs_inst: TypedSpiBsInst<'_, TardisRelDBClient>,
    tag: &str,
    ctx: &TardisContext,
    mgr: bool,
) -> TardisResult<(TardisRelDBlConnection, String)> {
    spi_initializer::common_pg::init_table_and_conn(
        bs_inst,
        ctx,
        mgr,
        Some(tag),
        "log",
        r#"ts timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    key character varying NOT NULL,
    op character varying NOT NULL,
    content text NOT NULL,
    kind character varying NOT NULL,
    owner character varying NOT NULL,
    own_paths character varying NOT NULL,
    ext jsonb NOT NULL,
    rel_key character varying NOT NULL"#,
        vec![
            ("kind", "btree"),
            ("ts", "btree"),
            ("key", "btree"),
            ("op", "btree"),
            ("ext", "gin"),
            ("owner", "btree"),
            ("own_paths", "btree"),
            ("rel_key", "btree"),
        ],
        None,
        None,
    )
    .await
}
