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
    pub tag: (TardisRelDBlConnection, String),
    pub config_tag_rel: (TardisRelDBlConnection, String),
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
config_tags text NOT NULL DEFAULT '',
tp character varying"#
        ),
        vec![("data_id", "btree"), ("grp", "btree"), ("namespace_id", "btree"), ("md5", "btree"), ("app_name", "btree")],
        None,
        Some("modified_time"),
    )
    .await
}

pub async fn init_table_and_conn_tag(
    bs_inst: (&TardisRelDBClient, &HashMap<String, String>, String),
    ctx: &TardisContext,
    mgr: bool,
) -> TardisResult<(TardisRelDBlConnection, String)> {
    spi_initializer::common_pg::init_table_and_conn(bs_inst, ctx, mgr, None, "conf_tag", r#"id character varying PRIMARY KEY"#, vec![], None, None).await
}

pub async fn init_table_and_conn_tag_config_rel(
    bs_inst: (&TardisRelDBClient, &HashMap<String, String>, String),
    config_table_name: &str,
    tag_table_name: &str,
    ctx: &TardisContext,
    mgr: bool,
) -> TardisResult<(TardisRelDBlConnection, String)> {
    spi_initializer::common_pg::init_table_and_conn(
        bs_inst,
        ctx,
        mgr,
        None,
        "conf_tag_config_rel",
        &format!(
            r#"id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
tag_id character varying NOT NULL REFERENCES {tag_table_name} ON DELETE CASCADE,
config_id uuid NOT NULL REFERENCES {config_table_name} ON DELETE CASCADE"#
        ),
        vec![("tag_id", "btree"), ("config_id", "btree")],
        None,
        None,
    )
    .await
}

pub async fn init_table_and_conn(bs_inst: (&TardisRelDBClient, &HashMap<String, String>, String), ctx: &TardisContext, mgr: bool) -> TardisResult<SpiConfTableAndConns> {
    let (name_space_conn, namespace_table_name) = init_table_and_conn_namespace(bs_inst.clone(), ctx, mgr).await?;
    let (config_conn, config_table_name) = init_table_and_conn_config(bs_inst.clone(), namespace_table_name.as_str(), ctx, mgr).await?;
    let (config_history_conn, history_table_name) = init_table_and_conn_history(bs_inst.clone(), namespace_table_name.as_str(), ctx, mgr).await?;
    let (tag_conn, tag_table_name) = init_table_and_conn_tag(bs_inst.clone(), ctx, mgr).await?;
    let (config_tag_rel_conn, config_tag_rel_table_name) = init_table_and_conn_tag_config_rel(bs_inst, &config_table_name, &tag_table_name, ctx, mgr).await?;
    Ok(SpiConfTableAndConns {
        namespace: (name_space_conn, namespace_table_name),
        config: (config_conn, config_table_name),
        config_history: (config_history_conn, history_table_name),
        tag: (tag_conn, tag_table_name),
        config_tag_rel: (config_tag_rel_conn, config_tag_rel_table_name),
    })
}
