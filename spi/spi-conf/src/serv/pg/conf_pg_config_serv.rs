use bios_basic::spi::spi_funs::SpiBsInstExtractor;
use tardis::{
    basic::{dto::TardisContext, error::TardisError, result::TardisResult},
    db::{reldb_client::TardisRelDBClient, sea_orm::Value},
    web::web_resp::TardisPage,
    TardisFunsInst,
};

use crate::{
    conf_constants::*,
    dto::{conf_config_dto::*, conf_namespace_dto::NamespaceItem},
};

use super::{conf_pg_initializer, get_namespace_id, OpType, add_history, HistoryInsertParams};

fn md5(content: &str) -> String {
    use tardis::crypto::rust_crypto::{digest::Digest, md5::Md5};
    let mut md5 = Md5::new();
    md5.input_str(content);
    md5.result_str()
}

pub async fn get_config(descriptor: &mut ConfigDescriptor, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
    let data_id = &descriptor.data_id;
    let group = &descriptor.group;
    let namespace = get_namespace_id(&descriptor.namespace_id);
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let conns = conf_pg_initializer::init_table_and_conn(bs_inst, ctx, true).await?;
    let (conn, table_name) = conns.config;
    let qry_result = conn
        .query_one(
            &format!(
                r#"SELECT (content) FROM {table_name} cc
WHERE cc.namespace_id='{namespace}' AND cc.grp='{group}' AND cc.data_id='{data_id}'
	"#,
            ),
            vec![Value::from(data_id), Value::from(group), Value::from(namespace)],
        )
        .await?
        .ok_or(TardisError::not_found("config not found", error::NAMESPACE_NOTFOUND))?;
    let content = qry_result.try_get::<String>("", "content")?;
    Ok(content)
}

pub async fn get_md5(descriptor: &mut ConfigDescriptor, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
    let data_id = &descriptor.data_id;
    let group = &descriptor.group;
    let namespace = get_namespace_id(&descriptor.namespace_id);
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let conns = conf_pg_initializer::init_table_and_conn(bs_inst, ctx, true).await?;
    let (conn, table_name) = conns.config;
    let qry_result = conn
        .query_one(
            &format!(
                r#"SELECT (md5) FROM {table_name} cc
WHERE cc.namespace_id='{namespace}' AND cc.grp='{group}' AND cc.data_id='{data_id}'
	"#,
            ),
            vec![Value::from(data_id), Value::from(group), Value::from(namespace)],
        )
        .await?
        .ok_or(TardisError::not_found("config not found", error::NAMESPACE_NOTFOUND))?;
    let md5 = qry_result.try_get::<String>("", "md5")?;
    Ok(md5)
}

pub async fn publish_config(req: &mut ConfigPublishRequest, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
    let data_id = &req.descriptor.data_id;
    let group = &req.descriptor.group;
    let namespace = &get_namespace_id(&req.descriptor.namespace_id);
    let content = &req.content;
    let md5 = &md5(content);
    let app_name = req.app_name.as_deref();
    let schema = req.schema.as_deref();
    let history = HistoryInsertParams { data_id, group, namespace, content, md5, app_name, schema};
    let params = vec![
        ("content", Value::from(content)),
        ("md5", Value::from(md5)),
        ("app_name", Value::from(app_name)),
        ("schema", Value::from(schema)),
    ];
    let key_params = vec![
        ("data_id", Value::from(data_id)),
        ("grp", Value::from(group)),
        ("namespace_id", Value::from(namespace)),
    ];
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let conns = conf_pg_initializer::init_table_and_conn(bs_inst, ctx, true).await?;
    let (mut conn, table_name) = conns.config;
    // check if exists
    let qry_result = conn
        .query_one(
            &format!(
                r#"SELECT (data_id) FROM {table_name} cc
WHERE cc.namespace_id='{namespace}' AND cc.grp='{group}' AND cc.data_id='{data_id}'
                    "#,
            ),
            vec![Value::from(data_id), Value::from(group), Value::from(namespace)],
        )
        .await?;
    let op_type: OpType;

    conn.begin().await?;
    if qry_result.is_some() {
        // if exists, update
        op_type = OpType::Update;
        let (set_caluse, where_caluse, values) = super::gen_update_sql_stmt(params, key_params);
        add_history(history, op_type, funs, ctx).await?;
        conn.execute_one(
            &format!(
                r#"UPDATE {table_name} 
SET {set_caluse}
WHERE {where_caluse}
        "#,
            ),
            values,
        )
        .await?;
    } else {
        // if not exists, insert
        op_type = OpType::Insert;
        add_history(history, op_type, funs, ctx).await?;
        let mut fields_and_values = params;
        fields_and_values.extend(key_params);
        let (fields, placeholders, values) = super::gen_insert_sql_stmt(fields_and_values);
        conn.begin().await?;
        conn.execute_one(
            &format!(
                r#"INSERT INTO {table_name} 
        ({fields})
    VALUES
        ({placeholders})
        "#,
            ),
            values,
        )
        .await?;
    }
    conn.commit().await?;
    Ok(true)
}

pub async fn delete_config(descriptor: &mut ConfigDescriptor, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
    let data_id = &descriptor.data_id;
    let group = &descriptor.group;
    let namespace = &get_namespace_id(&descriptor.namespace_id);
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let conns = conf_pg_initializer::init_table_and_conn(bs_inst, ctx, true).await?;
    let history = HistoryInsertParams { data_id, group, namespace, ..Default::default()};
    let (mut conn, table_name) = conns.config;
    conn.begin().await?;
    add_history(history, OpType::Delete, funs, ctx).await?;
    conn.execute_one(
        &format!(
            r#"DELETE FROM {table_name} cc
WHERE cc.namespace_id='{namespace}' AND cc.grp='{group}' AND cc.data_id='{data_id}'
    "#,
        ),
        vec![Value::from(data_id), Value::from(group), Value::from(namespace)],
    )
    .await?;
    conn.commit().await?;
    Ok(true)
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
