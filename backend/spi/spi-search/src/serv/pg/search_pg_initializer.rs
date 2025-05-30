use bios_basic::spi::{spi_funs::TypedSpiBsInst, spi_initializer};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    db::reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
};

pub async fn init_table_and_conn(bs_inst: TypedSpiBsInst<'_, TardisRelDBClient>, tag: &str, ctx: &TardisContext, mgr: bool) -> TardisResult<(TardisRelDBlConnection, String)> {
    spi_initializer::common_pg::init_table_and_conn(
        bs_inst,
        ctx,
        mgr,
        Some(tag),
        "search",
        r#"kind character varying NOT NULL,
    key character varying NOT NULL PRIMARY KEY,
    title character varying NOT NULL,
    title_tsv tsvector,
    content text,
    content_tsv tsvector,
    data_source character varying NOT NULL,
    owner character varying NOT NULL,
    own_paths character varying NOT NULL,
    create_time timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    update_time timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ext jsonb NOT NULL,
    visit_keys jsonb"#,
        None,
        vec![
            ("kind", "btree"),
            ("key", "btree"),
            ("title_tsv", "gin"),
            ("content_tsv", "gin"),
            ("ext", "gin"),
            ("data_source", "btree"),
            ("owner", "btree"),
            ("own_paths", "btree"),
            ("create_time", "btree"),
            ("update_time", "btree"),
            ("visit_keys", "gin"),
        ],
        None,
        Some("update_time"),
    )
    .await
}
