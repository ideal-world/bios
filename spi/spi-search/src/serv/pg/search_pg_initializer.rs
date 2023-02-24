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
) -> TardisResult<(TardisRelDBlConnection, String)> {
    spi_initializer::common_pg::init_table_and_conn(
        bs_inst,
        ctx,
        mgr,
        Some(tag),
        "search",
        r#"key character varying NOT NULL PRIMARY KEY,
    title character varying NOT NULL,
    title_tsv tsvector,
    content_tsv tsvector,
    owner character varying NOT NULL,
    own_paths character varying NOT NULL,
    create_time timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    update_time timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ext jsonb NOT NULL,
    visit_keys character varying[]"#,
        vec![
            ("key", "btree"),
            ("title_tsv", "gin"),
            ("content_tsv", "gin"),
            ("ext", "gin"),
            ("owner", "btree"),
            ("own_paths", "btree"),
            ("create_time", "btree"),
            ("update_time", "btree"),
            ("visit_keys", "btree"),
        ],
        Some("update_time"),
    )
    .await
}
