use bios_basic::spi::{spi_funs::TypedSpiBsInst, spi_initializer};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    db::reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
};

pub async fn init_table_and_conn(bs_inst: TypedSpiBsInst<'_, TardisRelDBClient>, ctx: &TardisContext, mgr: bool) -> TardisResult<(TardisRelDBlConnection, String)> {
    spi_initializer::common_pg::init_table_and_conn(
        bs_inst,
        ctx,
        mgr,
        None,
        "kv",
        r#"k character varying NOT NULL PRIMARY KEY,
    v jsonb NOT NULL,
    info character varying  NOT NULL,
    create_time timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    update_time timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP"#,
        vec![("k", "btree"), ("v", "gin")],
        None,
        Some("update_time"),
    )
    .await
}
