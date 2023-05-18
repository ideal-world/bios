use std::collections::HashMap;

use bios_basic::spi::spi_initializer;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    db::reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
};

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
const CONFIG_TABLE_CREATE_CONTENT: &str = r#"id character varying NOT NULL PRIMARY KEY,
group character varying NOT NULL,
namespace character varying NOT NULL,
md5 character varying NOT NULL,
content text NOT NULL,
app_name character varying,
src_user character varying,
src_ip cidr varying,
create_time timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
last_modify_time timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
tag character varying,
type smallint NOT NULL"#;

pub async fn init_table_and_conn_config(
    bs_inst: (&TardisRelDBClient, &HashMap<String, String>, String),
    ctx: &TardisContext,
    mgr: bool,
) -> TardisResult<(TardisRelDBlConnection, String)> {
    spi_initializer::common_pg::init_table_and_conn(
        bs_inst,
        ctx,
        mgr,
        None,
        "conf_config",
        CONFIG_TABLE_CREATE_CONTENT,
        vec![("show_name", "btree")],
        None,
        Some("last_modify_time"),
    )
    .await
}

//    /// 命名空间，默认为public与 ''相同
//    pub namespace_id: Option<NamespaceId>,
//    /// 配置分组名
//    pub group: String,
//    /// 配置名
//    pub data_id: String,
//    /// 标签
//    pub tag: Option<String>,
//
//
//
//
//
