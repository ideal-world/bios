use std::{collections::{HashMap, HashSet}, time::Duration};

use bios_basic::spi::spi_funs::SpiBsInstExtractor;
use tardis::{
    basic::{dto::TardisContext, error::TardisError, result::TardisResult},
    db::{
        reldb_client::TardisRelDBClient,
        sea_orm::{
            prelude::{DateTimeUtc, Uuid},
            Value,
        },
    },
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

use super::{add_history, conf_pg_initializer, gen_select_sql_stmt, HistoryInsertParams, OpType};

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
    let config_tags = &req.config_tags;
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
    let (mut conn, config_table_name) = conns.config;
    let (_, tag_table_name) = conns.tag;
    let (_, config_tag_rel_table_name) = conns.config_tag_rel;
    // check if exists
    let qry_result = conn
        .query_one(
            &format!(
                r#"SELECT (data_id) FROM {config_table_name} cc
WHERE cc.namespace_id='{namespace}' AND cc.grp='{group}' AND cc.data_id='{data_id}'
                    "#,
            ),
            vec![Value::from(data_id), Value::from(group), Value::from(namespace)],
        )
        .await?
        .and_then(|r| r.try_get::<Uuid>("", "data_id").ok());
    let op_type: OpType;

    conn.begin().await?;
    if !config_tags.is_empty() {
        let placeholders = (1..=config_tags.len()).map(|idx| format!("${idx}")).collect::<Vec<String>>().join(", ");
        let config_values = config_tags.iter().map(Value::from).collect();
        // 1. insert tags
        conn.execute_one(
            &format!("INSERT INTO {tag_table_name} (id) VALUES ({placeholders}) ON CONFLICT (id) DO NOTHING",),
            config_values,
        )
        .await?;
    }
    let config_id = if let Some(uuid) = qry_result {
        // if exists, update
        op_type = OpType::Update;
        let (set_caluse, where_caluse, values) = super::gen_update_sql_stmt(params, key_params);
        add_history(history, op_type, funs, ctx).await?;
        conn.execute_one(
            &format!(
                r#"UPDATE {config_table_name} 
SET {set_caluse}
WHERE {where_caluse}
        "#,
            ),
            values,
        )
        .await?;
        uuid
    } else {
        // if not exists, insert
        op_type = OpType::Insert;
        add_history(history, op_type, funs, ctx).await?;
        let mut fields_and_values = params;
        fields_and_values.extend(key_params);
        let (fields, placeholders, values) = super::gen_insert_sql_stmt(fields_and_values);
        conn.begin().await?;
        let result = conn
            .execute_one(
                &format!(
                    r#"INSERT INTO {config_table_name} 
        ({fields})
    VALUES
        ({placeholders})
    RETURNING id
        "#
                ),
                values,
            )
            .await?;
        conn.query_one(
            &format!(
                r#"SELECT (data_id) FROM {config_table_name} cc
    WHERE cc.namespace_id='{namespace}' AND cc.grp='{group}' AND cc.data_id='{data_id}'
                    "#,
            ),
            vec![Value::from(data_id), Value::from(group), Value::from(namespace)],
        )
        .await?
        .and_then(|r| r.try_get::<Uuid>("", "data_id").ok())
        .unwrap()
    };
    // get existed tags
    let mut exsisted_tags = conn
    .query_all(&format!("SELECT tag_id FROM {tag_table_name} t WHERE t.data_id=$1"), vec![Value::from(config_id)]).await?
    .iter().map(|r| r.try_get::<String>("", "tag_id")).collect::<Result<HashSet<String>, _>>()?;
    let mut insert_values = vec![];
    for tag in config_tags {
        if exsisted_tags.take(tag).is_none() {
            // insert rel
            insert_values.push(vec!(Value::from(tag), Value::from(config_id)));
        }
    }
    conn.insert_raw_many(&format!("INSERT INTO {config_tag_rel_table_name} (tag_id, config_id) VALUES $1"), insert_values).await?;
    let placeholders = (1..=exsisted_tags.len()).map(|idx| format!("${idx}")).collect::<Vec<String>>().join(", ");
    let mut delete_values = exsisted_tags.iter().map(Value::from).collect::<Vec<_>>();
    let cfg_id_ph = delete_values.len() + 1;
    delete_values.push(Value::from(config_id));
    conn.execute_one(&format!(
        r#"
        DELETE FROM {config_tag_rel_table_name} ctr
        WHERE ctr.config_id=${cfg_id_ph} AND ctr.tag_id NOT IN ({placeholders})
        "#
    ), delete_values).await?;
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

pub async fn get_configs(req: ConfigListRequest, mode: SearchMode, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<ConfigListResponse> {
    // query config history list by a ConfigDescriptor
    let ConfigListRequest {
        page_no,
        page_size,
        namespace_id,
        group,
        data_id,
        tags,
        tp,
    } = req;
    let limit = page_size.min(500).max(1);
    let page_number = page_no.max(1);
    let offset = (page_number - 1) * limit;
    let mut keys = vec![];
    let op = match mode {
        SearchMode::Fuzzy => "~",
        SearchMode::Exact => "=",
    };
    keys.extend(data_id.as_deref().map(|data_id| ("data_id", op, Value::from(data_id))));
    keys.extend(namespace_id.as_deref().map(|namespace_id| ("namespace_id", op, Value::from(if namespace_id.is_empty() { "public" } else { namespace_id }))));
    keys.extend(group.as_deref().map(|group| ("grp", op, Value::from(group))));
    keys.extend(tp.as_deref().map(|tp| ("tp", op, Value::from(tp))));
    let (where_clause, mut values) = gen_select_sql_stmt(keys);
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let conns = conf_pg_initializer::init_table_and_conn(bs_inst, ctx, true).await?;
    let (conn, config_table_name) = conns.config;
    let (_, config_tag_rel_table_name) = conns.config_tag_rel;
    let inner_join_clause = if !tags.is_empty() {
        values.push(Value::from(tags));
        format!(
            r#"INNER JOIN (
            SELECT data_id FROM {config_tag_rel_table_name} ctr
            WHERE ctr.tag_id IN (${tags_placeholder})
        ) AS ctrd ON cch.data_id=ctrd.data_id"#,
            tags_placeholder = values.len()
        )
    } else {
        "".into()
    };
    let qry_result_list = conn
        .query_all(
            &format!(
                r#"SELECT id, data_id, namespace_id, md5, content, src_user, op_type, created_time, modified_time, grp, count(*) AS total_count FROM {config_table_name} cch
    WHERE {where_clause}
    {inner_join_clause}
    ORDER BY created_time DESC
    LIMIT {limit}
    OFFSET {offset}
    "#,
            ),
            values,
        )
        .await?;
    let mut total = 0;
    let list = qry_result_list
        .into_iter()
        .map(|qry_result| {
            get!(qry_result => {
                id: Uuid,
                data_id: String,
                namespace_id: String,
                md5: String,
                content: String,
                op_type: String,
                created_time: DateTimeUtc,
                modified_time: DateTimeUtc,
                grp: String,
                total_count: u32,
            });
            total = total_count;
            Ok(ConfigItem {
                id: id.to_string(),
                data_id,
                namespace: namespace_id,
                md5,
                content,
                op_type,
                created_time,
                last_modified_time: modified_time,
                group: grp,
                ..Default::default()
            })
        })
        .collect::<TardisResult<Vec<_>>>()?;
    Ok(ConfigListResponse {
        total_count: total,
        page_number,
        pages_available: total / limit + 1,
        page_items: list,
    })
}
