use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    db::reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
};

use bios_basic::spi::{spi_funs::TypedSpiBsInst, spi_initializer};

use crate::log_constants;

pub async fn init_table_and_conn(bs_inst: TypedSpiBsInst<'_, TardisRelDBClient>, tag: &str, ctx: &TardisContext, mgr: bool) -> TardisResult<(TardisRelDBlConnection, String)> {
    spi_initializer::common_pg::init_table_and_conn(
        bs_inst,
        ctx,
        mgr,
        Some(tag),
        log_constants::TABLE_LOG_FLAG_V2,
        r#""#,
        Some(crate::log_constants::PARENT_TABLE_NAME.to_string()),
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
        ],
        None,
        None,
    )
    .await
}
