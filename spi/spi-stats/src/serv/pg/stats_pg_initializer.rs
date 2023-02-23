use std::collections::HashMap;

use bios_basic::spi::spi_initializer;
use tardis::{basic::{dto::TardisContext, result::TardisResult}, db::reldb_client::{TardisRelDBlConnection, TardisRelDBClient}};

pub async fn init_conf_dim_table_and_conn(bs_inst: (&TardisRelDBClient, &HashMap<String, String>, String), ctx: &TardisContext, mgr: bool) -> TardisResult<TardisRelDBlConnection> {
    spi_initializer::common_pg::init_table_and_conn(
        bs_inst,
        ctx,
        mgr,
        Some("conf_dim"),
        "stats",
        r#"key character varying NOT NULL,
        show_name character varying NOT NULL,
        stable_ds boolean DEFAULT FALSE,
        data_type character varying NOT NULL,
        hierarchy character varying[] NOT NULL,
        remark character varying NOT NULL,
        create_time timestamp without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
        update_time timestamp without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP"#,
        vec![],
        Some("update_time"),
    )
    .await
}

pub async fn init_conf_fact_table_and_conn(bs_inst: (&TardisRelDBClient, &HashMap<String, String>, String), ctx: &TardisContext, mgr: bool) -> TardisResult<TardisRelDBlConnection> {
    spi_initializer::common_pg::init_table_and_conn(
        bs_inst,
        ctx,
        mgr,
        Some("conf_fact"),
        "stats",
        r#"key character varying NOT NULL,
        show_name character varying NOT NULL,
        query_limit integer DEFAULT 1000,
        remark character varying NOT NULL,
        create_time timestamp without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
        update_time timestamp without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP"#,
        vec![],
        Some("update_time"),
    )
    .await
}

pub async fn init_conf_fact_col_table_and_conn(bs_inst: (&TardisRelDBClient, &HashMap<String, String>, String), ctx: &TardisContext, mgr: bool) -> TardisResult<TardisRelDBlConnection> {
    spi_initializer::common_pg::init_table_and_conn(
        bs_inst,
        ctx,
        mgr,
        Some("conf_fact_col"),
        "stats",
        r#"key character varying NOT NULL,
        show_name character varying NOT NULL,
        kind character varying NOT NULL,
        dim_rel_conf_dim_key character varying,
        dim_multi_values boolean,
        dim_exclusive_rec boolean,
        mes_data_type character varying,
        mes_frequency character varying,
        mes_act_by_dim_conf_keys character varying[],
        rel_conf_fact_key character varying NOT NULL,
        rel_conf_fact_and_col_key character varying,
        remark character varying NOT NULL,
        create_time timestamp without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
        update_time timestamp without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
        unique (key, rel_conf_fact_key)"#,
        vec![("rel_conf_fact_key","btree")],
        Some("update_time"),
    )
    .await
}