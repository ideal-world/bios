use std::collections::HashMap;

use bios_basic::spi::spi_initializer;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    db::reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
};
pub struct SpiConfTableAndConns {
    pub namespace: (TardisRelDBlConnection, String),
    pub config: (TardisRelDBlConnection, String),
    pub config_history: (TardisRelDBlConnection, String),
    // pub config_tag: (TardisRelDBlConnection, String),
}
pub async fn init_table_and_conn_namespace(
    bs_inst: (&TardisRelDBClient, &HashMap<String, String>, String),
    ctx: &TardisContext,
    mgr: bool,
) -> TardisResult<(TardisRelDBlConnection, String)> {
    let (conn, table_name) = spi_initializer::common_pg::init_table_and_conn(
        bs_inst,
        ctx,
        mgr,
        None,
        "conf_namespace",
        r#"id character varying PRIMARY KEY,
    show_name character varying NOT NULL,
    description text,
    tp smallint NOT NULL DEFAULT 0"#,
        vec![("show_name", "btree")],
        None,
        None,
    )
    .await?;
    conn.execute_one(
        format!("INSERT INTO {table_name} (id, show_name, description) VALUES ('public', 'public', 'default public domain') ON CONFLICT (id) DO NOTHING").as_str(),
        vec![],
    )
    .await?;
    Ok((conn, table_name))
}

pub async fn init_table_and_conn_config(
    bs_inst: (&TardisRelDBClient, &HashMap<String, String>, String),
    namespace_table_name: &str,
    ctx: &TardisContext,
    mgr: bool,
) -> TardisResult<(TardisRelDBlConnection, String)> {
    spi_initializer::common_pg::init_table_and_conn(
        bs_inst,
        ctx,
        mgr,
        None,
        "conf_config",
        &format!(
            r#"id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
data_id character varying NOT NULL,
grp character varying NOT NULL DEFAULT 'DEFAULT-GROUP',
namespace_id character varying NOT NULL DEFAULT 'public' REFERENCES {namespace_table_name} ON DELETE CASCADE,
md5 character(32) NOT NULL,
content text NOT NULL,
schema character varying,
app_name character varying,
src_user character varying,
src_ip cidr,
created_time timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
modified_time timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
tp character varying"#
        ),
        vec![("data_id", "btree"), ("grp", "btree"), ("namespace_id", "btree"), ("md5", "btree"), ("app_name", "btree")],
        None,
        Some("modified_time"),
    )
    .await
}

pub async fn init_table_and_conn_history(
    bs_inst: (&TardisRelDBClient, &HashMap<String, String>, String),
    namespace_table_name: &str,
    ctx: &TardisContext,
    mgr: bool,
) -> TardisResult<(TardisRelDBlConnection, String)> {
    spi_initializer::common_pg::init_table_and_conn(
        bs_inst,
        ctx,
        mgr,
        None,
        "conf_config_history",
        &format!(
            r#"id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
data_id character varying NOT NULL,
grp character varying NOT NULL DEFAULT 'DEFAULT-GROUP',
namespace_id character varying NOT NULL DEFAULT 'public' REFERENCES {namespace_table_name} ON DELETE CASCADE,
md5 character(32) NOT NULL,
content text NOT NULL,
schema character varying,
app_name character varying,
src_user character varying,
src_ip cidr,
created_time timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
modified_time timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
op_type character(1) NOT NULL DEFAULT 'I',
tp character varying"#
        ),
        vec![("data_id", "btree"), ("grp", "btree"), ("namespace_id", "btree"), ("md5", "btree"), ("app_name", "btree")],
        None,
        Some("modified_time"),
    )
    .await
}

pub async fn init_table_and_conn(bs_inst: (&TardisRelDBClient, &HashMap<String, String>, String), ctx: &TardisContext, mgr: bool) -> TardisResult<SpiConfTableAndConns> {
    let (name_space_conn, namespace_table_name) = init_table_and_conn_namespace(bs_inst.clone(), ctx, mgr).await?;
    let (config_conn, config_table_name) = init_table_and_conn_config(bs_inst.clone(), namespace_table_name.as_str(), ctx, mgr).await?;
    let (config_history_conn, history_table_name) = init_table_and_conn_history(bs_inst, namespace_table_name.as_str(), ctx, mgr).await?;
    Ok(SpiConfTableAndConns {
        namespace: (name_space_conn, namespace_table_name),
        config: (config_conn, config_table_name),
        config_history: (config_history_conn, history_table_name)
    })
}
