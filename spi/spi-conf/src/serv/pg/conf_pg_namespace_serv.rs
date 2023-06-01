use bios_basic::spi::spi_funs::SpiBsInstExtractor;
use tardis::{
    basic::{dto::TardisContext, error::TardisError, result::TardisResult},
    db::{reldb_client::TardisRelDBClient, sea_orm::Value},
    TardisFunsInst,
};

use crate::{conf_constants::error, dto::conf_namespace_dto::*, serv::pg::conf_pg_initializer};
pub async fn get_namespace(discriptor: &mut NamespaceDescriptor, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<NamespaceItem> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let conns = conf_pg_initializer::init_table_and_conn(bs_inst, ctx, true).await?;
    let (conn, table_name_namespace) = conns.namespace;
    let (_, table_name_config) = conns.config;
    let namespace = conn
        .query_one(
            &format!(
                r#"SELECT id, show_name, description
FROM {table_name}
WHERE id = $1"#,
                table_name = table_name_namespace
            ),
            vec![Value::from(&discriptor.namespace_id)],
        )
        .await?
        .ok_or(TardisError::not_found("namespace not found", error::NAMESPACE_NOTFOUND))?;
    let mut namespace_item = NamespaceItem {
        namespace: namespace.try_get("", "id").unwrap(),
        namespace_show_name: namespace.try_get("", "show_name").unwrap(),
        namespace_desc: namespace.try_get("", "description").unwrap(),
        ..Default::default()
    };
    let count = conn
        .count_by_sql(
            format!(
                "SELECT namespace_id FROM {table} WHERE namespace_id='{id}'",
                table = table_name_config,
                id = namespace_item.namespace,
            )
            .as_str(),
            vec![],
        )
        .await?;
    namespace_item.config_count = count as u32;
    Ok(namespace_item)
}
pub async fn create_namespace(attribute: &mut NamespaceAttribute, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let mut params = vec![Value::from(&attribute.namespace), Value::from(&attribute.namespace_show_name)];
    params.extend(attribute.namespace_desc.as_ref().map(Value::from));
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = conf_pg_initializer::init_table_and_conn_namespace(bs_inst, ctx, true).await?;
    conn.begin().await?;
    conn.execute_one(
        &format!(
            r#"INSERT INTO {table_name} 
    (id, show_name{})
VALUES
    ($1, $2{})
	"#,
            if attribute.namespace_desc.is_some() { ", description" } else { "" },
            if attribute.namespace_desc.is_some() { ", $3" } else { "" },
        ),
        params,
    )
    .await?;
    conn.commit().await?;
    Ok(())
}
pub async fn edit_namespace(attribute: &mut NamespaceAttribute, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let mut params = vec![Value::from(&attribute.namespace), Value::from(&attribute.namespace_show_name)];
    params.extend(attribute.namespace_desc.as_ref().map(Value::from));
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = conf_pg_initializer::init_table_and_conn_namespace(bs_inst, ctx, true).await?;
    conn.begin().await?;
    conn.execute_one(
        &format!(
            r#"UPDATE {table_name} 
SET show_name = $2{}
WHERE 
    id = $1
	"#,
            if attribute.namespace_desc.is_some() { ", description=$3" } else { "" },
        ),
        params,
    )
    .await?;
    conn.commit().await?;
    Ok(())
}

pub async fn delete_namespace(discriptor: &mut NamespaceDescriptor, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = conf_pg_initializer::init_table_and_conn_namespace(bs_inst, ctx, true).await?;
    if discriptor.namespace_id.is_empty() || discriptor.namespace_id == "public" {
        return Err(TardisError::bad_request("default namespace(public) can not be deleted", error::NAMESPACE_DEFAULT_CANNOT_DELETE));
    }
    conn.begin().await?;
    conn.execute_one(
        &format!(
            r#"DELETE FROM {table_name} 
WHERE 
    id = $1
    "#,
            table_name = table_name
        ),
        vec![Value::from(&discriptor.namespace_id)],
    )
    .await?;
    conn.commit().await?;
    Ok(())
}

// this could be slow
pub async fn get_namespace_list(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<NamespaceItem>> {
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let conns = conf_pg_initializer::init_table_and_conn(bs_inst, ctx, true).await?;
    let (conn, table_name_namespace) = conns.namespace;
    let (_, table_name_config) = conns.config;
    let namespaces = conn
        .query_all(
            &format!(
                r#"SELECT id, show_name, description
FROM {table_name}
"#,
                table_name = table_name_namespace
            ),
            vec![],
        )
        .await?;
    let mut namespace_items = vec![];
    for namespace in namespaces {
        let mut namespace_item = NamespaceItem {
            namespace: namespace.try_get("", "id").unwrap(),
            namespace_show_name: namespace.try_get("", "show_name").unwrap(),
            namespace_desc: namespace.try_get("", "description").unwrap(),
            ..Default::default()
        };
        let count = conn
            .count_by_sql(
                format!(
                    "SELECT namespace_id FROM {table} WHERE namespace_id='{id}'",
                    table = table_name_config,
                    id = namespace_item.namespace,
                )
                .as_str(),
                vec![],
            )
            .await?;
        namespace_item.config_count = count as u32;
        namespace_items.push(namespace_item);
    }
    Ok(namespace_items)
}
