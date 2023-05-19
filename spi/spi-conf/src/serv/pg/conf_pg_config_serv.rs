use bios_basic::spi::spi_funs::SpiBsInstExtractor;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult, error::TardisError},
    db::{reldb_client::TardisRelDBClient, sea_orm::Value},
    web::web_resp::TardisPage,
    TardisFunsInst,
};

use crate::{dto::conf_config_dto::*, conf_constants::*};

use super::conf_pg_initializer;

fn md5(content: &str) -> String {
    use tardis::crypto::rust_crypto::{digest::Digest, md5::Md5};
    let mut md5 = Md5::new();
    md5.input_str(content);
    md5.result_str()
}

fn get_namespace_id(namespace: &String) -> String {
    if namespace.is_empty() {
        "public".to_string()
    } else {
        namespace.to_string()
    }
}

pub async fn get_config(descriptor: &mut ConfigDescriptor, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
    let data_id = &descriptor.data_id;
    let group = &descriptor.group;
    let namespace = get_namespace_id(&descriptor.namespace_id);
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let conns = conf_pg_initializer::init_table_and_conn(bs_inst, ctx, true).await?;
    let (conn, table_name) = conns.config;
    let qry_result = conn.query_one(
        &format!(
            r#"SELECT (content) FROM {table_name} cc
WHERE cc.namespace_id='{namespace}' AND cc.grp='{group}' AND cc.data_id='{data_id}'
	"#,
        ),
        vec![Value::from(data_id), Value::from(group), Value::from(namespace)],
    )
    .await?.ok_or(TardisError::not_found("config not found", error::NAMESPACE_NOTFOUND))?;
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
    let qry_result = conn.query_one(
        &format!(
            r#"SELECT (md5) FROM {table_name} cc
WHERE cc.namespace_id='{namespace}' AND cc.grp='{group}' AND cc.data_id='{data_id}'
	"#,
        ),
        vec![Value::from(data_id), Value::from(group), Value::from(namespace)],
    )
    .await?.ok_or(TardisError::not_found("config not found", error::NAMESPACE_NOTFOUND))?;
    let md5 = qry_result.try_get::<String>("", "md5")?;
    Ok(md5)
}

pub async fn publish_config(req: &mut ConfigPublishRequest, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
    let data_id = &req.descriptor.data_id;
    let group = &req.descriptor.group;
    let namespace = get_namespace_id(&req.descriptor.namespace_id);
    let content = &req.content;
    let md5 = md5(content);
    let app_name = &req.app_name;
    let schema = &req.schema;
    let params = vec![
        ("data_id", Value::from(data_id)),
        ("grp", Value::from(group)),
        ("namespace_id", Value::from(namespace)),
        ("content", Value::from(content)),
        ("md5", Value::from(md5)),
        ("app_name", Value::from(app_name.clone())),
        ("schema", Value::from(schema.clone())),
    ];
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let conns = conf_pg_initializer::init_table_and_conn(bs_inst, ctx, true).await?;
    let (mut conn, table_name) = conns.config;
    conn.begin().await?;
    let (fields, placeholders, values) = super::gen_insert_sql_stmt(params);
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
    conn.commit().await?;
    Ok(true)
}

pub async fn delete_config(descriptor: &mut ConfigDescriptor, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
    let data_id = &descriptor.data_id;
    let group = &descriptor.group;
    let namespace = get_namespace_id(&descriptor.namespace_id);
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let conns = conf_pg_initializer::init_table_and_conn(bs_inst, ctx, true).await?;
    let (mut conn, table_name) = conns.config;
    conn.begin().await?;
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