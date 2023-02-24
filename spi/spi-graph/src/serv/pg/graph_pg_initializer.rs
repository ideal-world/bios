use std::collections::HashMap;

use bios_basic::spi::spi_initializer;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    db::reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
};

pub async fn init_table_and_conn(
    bs_inst: (&TardisRelDBClient, &HashMap<String, String>, String),
    ctx: &TardisContext,
    mgr: bool,
) -> TardisResult<(TardisRelDBlConnection, String)> {
    spi_initializer::common_pg::init_table_and_conn(
        bs_inst,
        ctx,
        mgr,
        None,
        "graph",
        r#"tag character varying NOT NULL,
    from_key character varying NOT NULL,
    from_version character varying NOT NULL,
    to_key character varying NOT NULL,
    to_version character varying NOT NULL,
    reverse bool DEFAULT false NOT NULL, 
    ts timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    check (from_key <> to_key),  
    unique (from_key, from_version, to_key, to_version, tag)"#,
        vec![
            ("tag", "btree"),
            ("from_key", "btree"),
            ("from_version", "btree"),
            ("to_key", "btree"),
            ("to_version", "btree"),
        ],
        None,
    )
    .await
}
