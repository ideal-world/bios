use std::collections::HashMap;

use bios_basic::spi::spi_initializer;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    db::reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
};

pub async fn init_table_and_conn(
    bs_inst: (&TardisRelDBClient, &HashMap<String, String>, String),
    tag: &str,
    ctx: &TardisContext,
    mgr: bool,
) -> TardisResult<TardisRelDBlConnection> {
    spi_initializer::common_pg::init_table_and_conn(
        bs_inst,
        tag,
        ctx,
        mgr,
        "log",
        r#"ts timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    key character varying NOT NULL,
    op character varying NOT NULL,
    content text NOT NULL,
    rel_key character varying NOT NULL"#,
        vec![("ts", "btree"), ("key", "btree"), ("op", "btree"), ("rel_key", "btree")],
    )
    .await
}
