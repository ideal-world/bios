use std::{collections::HashMap, time::Duration};

use bios_basic::spi::spi_funs::SpiBsInstExtractor;
use tardis::{
    basic::{dto::TardisContext, error::TardisError, result::TardisResult},
    db::{reldb_client::TardisRelDBClient, sea_orm::Value},
    tokio::{sync::RwLock, time::Instant},
    TardisFunsInst,
};

use crate::{
    conf_constants::*,
    dto::{conf_config_dto::*, conf_namespace_dto::*},
};

// local memory cached md5
lazy_static::lazy_static! {
    static ref MD5_CACHE: RwLock<HashMap<ConfigDescriptor, (String, Instant)>> = RwLock::new(HashMap::new());
}

macro_rules! get {
    ($result:expr => {$($name:ident: $type:ty,)*}) => {
        $(let $name = $result.try_get::<$type>("", stringify!($name))?;)*
    };
}

use super::{add_history, conf_pg_initializer, HistoryInsertParams, OpType};

fn md5(content: &str) -> String {
    use tardis::crypto::rust_crypto::{digest::Digest, md5::Md5};
    let mut md5 = Md5::new();
    md5.input_str(content);
    md5.result_str()
}

pub async fn get_config(descriptor: &mut ConfigDescriptor, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
    descriptor.fix_namespace_id();
    let data_id = &descriptor.data_id;
    let group = &descriptor.group;
    let namespace = &descriptor.namespace_id;
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
    descriptor.fix_namespace_id();
    const EXPIRE: Duration = Duration::from_secs(1);
    // try get from cache
    {
        if let Some((md5, register_time)) = MD5_CACHE.read().await.get(descriptor) {
            if register_time.elapsed() < EXPIRE {
                return Ok(md5.clone());
            }
        }
    }
    let data_id = &descriptor.data_id;
    let group = &descriptor.group;
    let namespace = &descriptor.namespace_id;
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
    // set cache
    {
        let mut cache = MD5_CACHE.write().await;
        cache.insert(descriptor.clone(), (md5.clone(), Instant::now()));
    }
    Ok(md5)
}

pub async fn publish_config(req: &mut ConfigPublishRequest, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
    // clear cache
    req.descriptor.fix_namespace_id();
    {
        let mut cache = MD5_CACHE.write().await;
        cache.remove(&req.descriptor);
    }
    let data_id = &req.descriptor.data_id;
    let group = &req.descriptor.group;
    let namespace = &req.descriptor.namespace_id;
    let content = &req.content;
    let md5 = &md5(content);
    let app_name = req.app_name.as_deref();
    let schema = req.schema.as_deref();
    let src_user = &ctx.owner;
    let history = HistoryInsertParams {
        data_id,
        group,
        namespace,
        content,
        md5,
        app_name,
        schema,
    };
    let params = vec![
        ("content", Value::from(content)),
        ("md5", Value::from(md5)),
        ("app_name", Value::from(app_name)),
        ("schema", Value::from(schema)),
        ("src_user", Value::from(src_user)),
    ];
    let key_params = vec![("data_id", Value::from(data_id)), ("grp", Value::from(group)), ("namespace_id", Value::from(namespace))];
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
    descriptor.fix_namespace_id();
    // clear cache
    {
        let mut cache = MD5_CACHE.write().await;
        cache.remove(descriptor);
    }
    let data_id = &descriptor.data_id;
    let group = &descriptor.group;
    let namespace = &descriptor.namespace_id;
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let conns = conf_pg_initializer::init_table_and_conn(bs_inst, ctx, true).await?;
    let history = HistoryInsertParams {
        data_id,
        group,
        namespace,
        ..Default::default()
    };
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

pub async fn get_configs_by_namespace(namespace_id: &NamespaceId, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<ConfigItemDigest>> {
    let namespace_id = if namespace_id.is_empty() { "public" } else { namespace_id };

    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let conns = conf_pg_initializer::init_table_and_conn(bs_inst, ctx, true).await?;
    let (conn, table_name) = conns.config;
    let qry_result = conn
        .query_all(
            &format!(
                r#"SELECT data_id, grp, namespace_id, app_name, tp FROM {table_name} cc
WHERE cc.namespace_id=$1
ORDER BY created_time DESC
	"#,
            ),
            vec![Value::from(namespace_id)],
        )
        .await?;
    let list = qry_result
        .iter()
        .map(|result| {
            get!(result => {
                data_id: String,
                grp: String,
                namespace_id: String,
                app_name: Option<String>,
                tp: Option<String>,
            });
            Ok(ConfigItemDigest {
                data_id,
                group: grp,
                namespace: namespace_id,
                app_name,
                r#type: tp,
            })
        })
        .collect::<TardisResult<Vec<_>>>()?;
    Ok(list)
}
