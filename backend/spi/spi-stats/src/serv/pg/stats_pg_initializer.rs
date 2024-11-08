use bios_basic::spi::{spi_funs::TypedSpiBsInst, spi_initializer};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    db::reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
};

pub async fn init_conf_dim_group_table_and_conn(bs_inst: TypedSpiBsInst<'_, TardisRelDBClient>, ctx: &TardisContext, mgr: bool) -> TardisResult<(TardisRelDBlConnection, String)> {
    spi_initializer::common_pg::init_table_and_conn(
        bs_inst,
        ctx,
        mgr,
        None,
        "stats_conf_dim_group",
        r#"key character varying NOT NULL,
    show_name character varying NOT NULL,
    data_type character varying NOT NULL,
    remark character varying NOT NULL,
    dynamic_url character varying NOT NULL,
    rel_attribute_code character varying[],
    rel_attribute_url character varying,
    create_time timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    update_time timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP"#,
        None,
        vec![],
        None,
        Some("update_time"),
    )
    .await
}
pub async fn init_conf_dim_table_and_conn(bs_inst: TypedSpiBsInst<'_, TardisRelDBClient>, ctx: &TardisContext, mgr: bool) -> TardisResult<(TardisRelDBlConnection, String)> {
    spi_initializer::common_pg::init_table_and_conn(
        bs_inst,
        ctx,
        mgr,
        None,
        "stats_conf_dim",
        r#"key character varying NOT NULL,
    show_name character varying NOT NULL,
    stable_ds boolean DEFAULT FALSE,
    data_type character varying NOT NULL,
    hierarchy character varying[] NOT NULL,
    remark character varying NOT NULL,
    dynamic_url character varying,
    is_tree boolean NOT NULL DEFAULT FALSE,
    dim_group_key character varying NOT NULL,
    tree_dynamic_url character varying,
    rel_attribute_code character varying[],
    rel_attribute_url character varying,
    create_time timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    update_time timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP"#,
        None,
        vec![],
        None,
        Some("update_time"),
    )
    .await
}

pub async fn init_conf_fact_table_and_conn(bs_inst: TypedSpiBsInst<'_, TardisRelDBClient>, ctx: &TardisContext, mgr: bool) -> TardisResult<(TardisRelDBlConnection, String)> {
    spi_initializer::common_pg::init_table_and_conn(
        bs_inst,
        ctx,
        mgr,
        None,
        "stats_conf_fact",
        r#"key character varying NOT NULL,
    redirect_path character varying,
    is_online boolean NOT NULL DEFAULT FALSE,
    show_name character varying NOT NULL,
    query_limit integer DEFAULT 10000,
    remark character varying NOT NULL,
    create_time timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    update_time timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    rel_cert_id character varying,
    sync_sql character varying,
    sync_cron character varying,
    is_sync boolean NOT NULL DEFAULT FALSE"#,
        None,
        vec![],
        None,
        Some("update_time"),
    )
    .await
}

pub async fn init_conf_fact_col_table_and_conn(bs_inst: TypedSpiBsInst<'_, TardisRelDBClient>, ctx: &TardisContext, mgr: bool) -> TardisResult<(TardisRelDBlConnection, String)> {
    spi_initializer::common_pg::init_table_and_conn(
        bs_inst,
        ctx,
        mgr,
        None,
        "stats_conf_fact_col",
        r#"key character varying NOT NULL,
    show_name character varying NOT NULL,
    kind character varying NOT NULL,
    dim_rel_conf_dim_key character varying,
    dim_multi_values boolean,
    dim_exclusive_rec boolean,
    dim_data_type character varying,
    dim_dynamic_url character varying,
    mes_data_distinct boolean,
    mes_data_type character varying,
    mes_frequency character varying,
    mes_unit character varying,
    mes_act_by_dim_conf_keys character varying[],
    rel_cert_id character varying,
    rel_field character varying,
    rel_sql character varying,
    rel_conf_fact_key character varying NOT NULL,
    rel_conf_fact_and_col_key character varying,
    rel_external_id character varying NOT NULL,
    remark character varying NOT NULL,
    create_time timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    update_time timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    unique (key, rel_conf_fact_key, kind, rel_external_id)"#,
        None,
        vec![("rel_conf_fact_key", "btree")],
        None,
        Some("update_time"),
    )
    .await
}
