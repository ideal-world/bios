use std::{
    collections::{HashMap, HashSet},
    net::IpAddr,
    time::Duration,
};

use bios_basic::spi::spi_funs::SpiBsInst;
use tardis::{
    basic::{dto::TardisContext, error::TardisError, result::TardisResult},
    db::{
        reldb_client::TardisRelDBClient,
        sea_orm::{
            prelude::{DateTimeUtc, Uuid},
            Value,
        },
    },
    log::warn,
    tokio::{sync::RwLock, time::Instant},
    TardisFunsInst,
};

use crate::{
    conf_constants::*,
    dto::{conf_config_dto::*, conf_namespace_dto::*},
    serv::{gen_md5, placeholder::render_content_for_ip},
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
    use tardis::crypto::crypto_digest::TardisCryptoDigest;
    TardisCryptoDigest.md5(content).expect("md5 digest shouldn't fail")
}
pub async fn fix_md5(descriptor: &ConfigDescriptor, md5: &str, ctx: &TardisContext, bs_inst: &SpiBsInst) -> TardisResult<()> {
    let data_id = &descriptor.data_id;
    let group = &descriptor.group;
    let namespace = &descriptor.namespace_id;
    let typed_inst = bs_inst.inst::<TardisRelDBClient>();
    let conns = conf_pg_initializer::init_table_and_conn(typed_inst, ctx, true).await?;
    let (conn, table_name) = conns.config;
    let fields_and_values = vec![("md5", Value::from(md5))];
    let key_params = vec![("data_id", Value::from(data_id)), ("grp", Value::from(group)), ("namespace_id", Value::from(namespace))];
    let (fields, placeholders, values) = super::gen_update_sql_stmt(fields_and_values, key_params);
    conn.execute_one(
        &format!(
            r#"UPDATE {table_name}
SET {fields}
WHERE {placeholders}"#,
        ),
        values,
    )
    .await?;
    MD5_CACHE.write().await.insert(descriptor.clone(), (md5.to_string(), Instant::now()));
    Ok(())
}
pub async fn get_config(descriptor: &mut ConfigDescriptor, _funs: &TardisFunsInst, ctx: &TardisContext, bs_inst: &SpiBsInst) -> TardisResult<String> {
    descriptor.fix_namespace_id();
    let data_id = &descriptor.data_id;
    let group = &descriptor.group;
    let namespace = &descriptor.namespace_id;
    let typed_inst = bs_inst.inst::<TardisRelDBClient>();
    let conns = conf_pg_initializer::init_table_and_conn(typed_inst, ctx, true).await?;
    let (conn, table_name) = conns.config;
    let qry_result = conn
        .query_one(
            &format!(
                r#"SELECT "content", "md5" FROM {table_name} cc
WHERE cc.namespace_id=$1 AND cc.grp=$2 AND cc.data_id=$3
	"#,
            ),
            vec![Value::from(namespace), Value::from(group), Value::from(data_id)],
        )
        .await?
        .ok_or_else(|| TardisError::not_found("config not found", error::NAMESPACE_NOTFOUND))?;
    let content = qry_result.try_get::<String>("", "content")?;
    Ok(content)
}

pub async fn get_config_detail(descriptor: &mut ConfigDescriptor, _funs: &TardisFunsInst, ctx: &TardisContext, bs_inst: &SpiBsInst) -> TardisResult<ConfigItem> {
    use md5 as gen_md5;
    descriptor.fix_namespace_id();
    let data_id = &descriptor.data_id;
    let group = &descriptor.group;
    let namespace = &descriptor.namespace_id;
    let typed_inst = bs_inst.inst::<TardisRelDBClient>();
    let conns = conf_pg_initializer::init_table_and_conn(typed_inst, ctx, true).await?;
    let (conn, table_name) = conns.config;
    let config_tag_rel_table_name = conns.config_tag_rel.1;

    let qry_result = conn
        .query_one(
            &format!(
                r#"SELECT 
    *, 
    ARRAY_TO_STRING(
        ARRAY(select tag_id from {config_tag_rel_table_name} tcr where tcr.config_id = c.id), ','
    ) as tags 
FROM {table_name} c
WHERE c.namespace_id=$1 AND c.grp=$2 AND c.data_id=$3"#,
            ),
            vec![Value::from(namespace), Value::from(group), Value::from(data_id)],
        )
        .await?
        .ok_or_else(|| TardisError::not_found("config not found", error::NAMESPACE_NOTFOUND))?;
    get!(qry_result => {
        id: Uuid,
        md5: String,
        content: String,
        created_time: DateTimeUtc,
        modified_time: DateTimeUtc,
        src_user: Option<String>,
        tags: Option<String>,
    });
    let config_tags = tags.map(|tags| tags.split(',').filter(|s| !s.is_empty()).map(String::from).collect()).unwrap_or_default();
    // fix md5 automatically
    let real_md5 = gen_md5(&content);
    if md5 != real_md5 {
        let update_result = fix_md5(descriptor, &real_md5, ctx, bs_inst).await;
        if let Err(e) = update_result {
            warn!("[Bios.spi-conf] update md5 failed: {}", e);
        }
    }
    Ok(ConfigItem {
        data_id: data_id.clone(),
        namespace: namespace.clone(),
        group: group.clone(),
        id: id.to_string(),
        md5: real_md5,
        content,
        created_time,
        last_modified_time: modified_time,
        config_tags,
        src_user,
        ..Default::default()
    })
}
pub async fn get_md5(descriptor: &mut ConfigDescriptor, source_addr: Option<IpAddr>, funs: &TardisFunsInst, ctx: &TardisContext, bs_inst: &SpiBsInst) -> TardisResult<String> {
    let content = get_config(descriptor, funs, ctx, bs_inst).await?;
    let content = render_content_for_ip(descriptor, content, source_addr, funs, ctx).await?;
    Ok(gen_md5(&content))
}

pub async fn get_raw_md5(descriptor: &mut ConfigDescriptor, _funs: &TardisFunsInst, ctx: &TardisContext, bs_inst: &SpiBsInst) -> TardisResult<String> {
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
    let typed_inst = bs_inst.inst::<TardisRelDBClient>();
    let conns = conf_pg_initializer::init_table_and_conn(typed_inst, ctx, true).await?;
    let (conn, table_name) = conns.config;
    let qry_result = conn
        .query_one(
            &format!(
                r#"SELECT (md5) FROM {table_name} cc
WHERE cc.namespace_id='{namespace}' AND cc.grp='{group}' AND cc.data_id='{data_id}'"#,
            ),
            vec![Value::from(data_id), Value::from(group), Value::from(namespace)],
        )
        .await?
        .ok_or_else(|| TardisError::not_found("config not found", error::NAMESPACE_NOTFOUND))?;
    let md5 = qry_result.try_get::<String>("", "md5")?;
    // set cache
    {
        let mut cache = MD5_CACHE.write().await;
        cache.insert(descriptor.clone(), (md5.clone(), Instant::now()));
    }
    Ok(md5)
}

pub async fn publish_config(req: &mut ConfigPublishRequest, funs: &TardisFunsInst, ctx: &TardisContext, bs_inst: &SpiBsInst) -> TardisResult<bool> {
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
        config_tags: config_tags.iter().map(String::as_str).collect(),
    };
    let params = vec![
        ("content", Value::from(content)),
        ("md5", Value::from(md5)),
        ("app_name", Value::from(app_name)),
        ("schema", Value::from(schema)),
        ("src_user", Value::from(src_user)),
    ];

    let key_params = vec![("data_id", Value::from(data_id)), ("grp", Value::from(group)), ("namespace_id", Value::from(namespace))];
    let typed_inst = bs_inst.inst::<TardisRelDBClient>();
    let conns = conf_pg_initializer::init_table_and_conn(typed_inst, ctx, true).await?;
    let (mut conn, config_table_name) = conns.config;
    let (_, tag_table_name) = conns.tag;
    let (_, config_tag_rel_table_name) = conns.config_tag_rel;
    // check if exists
    let qry_result = conn
        .query_one(
            &format!(
                r#"SELECT id FROM {config_table_name} cc
WHERE cc.grp=$1 AND cc.namespace_id=$2 AND cc.data_id=$3"#,
            ),
            vec![Value::from(group), Value::from(namespace), Value::from(data_id)],
        )
        .await?
        .and_then(|r| r.try_get::<Uuid>("", "id").ok());
    let op_type: OpType;

    conn.begin().await?;
    // if has config tags, insert tags first
    if !config_tags.is_empty() {
        let placeholders = (1..=config_tags.len()).map(|idx| format!("(${idx})")).collect::<Vec<String>>().join(", ");
        let config_values = config_tags.iter().map(Value::from).collect();
        // 1. insert tags
        conn.execute_one(
            &format!("INSERT INTO {tag_table_name} (id) VALUES {placeholders} ON CONFLICT (id) DO NOTHING",),
            config_values,
        )
        .await?;
    }
    // update or insert config, get config id
    let config_id = if let Some(uuid) = qry_result {
        // if exists, update
        op_type = OpType::Update;
        let (set_caluse, where_caluse, values) = super::gen_update_sql_stmt(params, key_params);
        add_history(history, op_type, funs, ctx, bs_inst).await?;
        conn.execute_one(
            &format!(
                r#"UPDATE {config_table_name} 
SET {set_caluse}
WHERE {where_caluse}"#,
            ),
            values,
        )
        .await?;
        uuid
    } else {
        // if not exists, insert
        op_type = OpType::Insert;
        add_history(history, op_type, funs, ctx, bs_inst).await?;
        let mut fields_and_values = params;
        fields_and_values.extend(key_params);
        let (fields, placeholders, values) = super::gen_insert_sql_stmt(fields_and_values);
        let _result = conn
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
                r#"SELECT id FROM {config_table_name} cc
    WHERE cc.data_id=$1 AND cc.grp=$2 AND cc.namespace_id=$3
                    "#,
            ),
            vec![Value::from(data_id), Value::from(group), Value::from(namespace)],
        )
        .await?
        .map(|r| r.try_get::<Uuid>("", "id").expect("query result no id (primary key) which is impossible"))
        .expect("no such config after insert")
    };
    // get existed tags
    let mut exsisted_tags = conn
        .query_all(
            &format!("SELECT tag_id FROM {config_tag_rel_table_name} t WHERE t.config_id=$1"),
            vec![Value::from(config_id)],
        )
        .await?
        .iter()
        .map(|r| r.try_get::<String>("", "tag_id"))
        .collect::<Result<HashSet<String>, _>>()?;

    let mut insert_values = vec![];

    for tag in config_tags {
        // for tags existed, do nothing
        if exsisted_tags.take(tag).is_none() {
            // insert rel
            insert_values.push(vec![Value::from(tag), Value::from(config_id)]);
        }
    }

    // insert new tags
    if !insert_values.is_empty() {
        let placeholders = (1..=insert_values.len()).map(|idx| format!("(${ptag}, ${pconfig})", ptag = idx * 2 - 1, pconfig = idx * 2)).collect::<Vec<String>>().join(", ");
        conn.execute_one(
            &format!("INSERT INTO {config_tag_rel_table_name} (tag_id, config_id) VALUES {placeholders}"),
            insert_values.concat(),
        )
        .await?;
    }

    // delete removed tags
    if !exsisted_tags.is_empty() {
        let placeholders = (1..=exsisted_tags.len()).map(|idx| format!("${idx}")).collect::<Vec<String>>().join(", ");
        let mut delete_values = exsisted_tags.iter().map(Value::from).collect::<Vec<_>>();
        let cfg_id_ph = delete_values.len() + 1;
        delete_values.push(Value::from(config_id));
        conn.execute_one(
            &format!(
                r#"
            DELETE FROM {config_tag_rel_table_name} ctr
            WHERE ctr.config_id=${cfg_id_ph} AND ctr.tag_id IN ({placeholders})
            "#
            ),
            delete_values,
        )
        .await?;
    }
    conn.commit().await?;
    Ok(true)
}

pub async fn delete_config(descriptor: &mut ConfigDescriptor, funs: &TardisFunsInst, ctx: &TardisContext, bs_inst: &SpiBsInst) -> TardisResult<bool> {
    descriptor.fix_namespace_id();
    // clear cache
    {
        let mut cache = MD5_CACHE.write().await;
        cache.remove(descriptor);
    }
    let data_id = &descriptor.data_id;
    let group = &descriptor.group;
    let namespace = &descriptor.namespace_id;
    let typed_inst = bs_inst.inst::<TardisRelDBClient>();
    let conns = conf_pg_initializer::init_table_and_conn(typed_inst, ctx, true).await?;
    let history = HistoryInsertParams {
        data_id,
        group,
        namespace,
        ..Default::default()
    };
    let (mut conn, table_name) = conns.config;
    conn.begin().await?;
    add_history(history, OpType::Delete, funs, ctx, bs_inst).await?;
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

pub async fn get_configs_by_namespace(namespace_id: &NamespaceId, _funs: &TardisFunsInst, ctx: &TardisContext, bs_inst: &SpiBsInst) -> TardisResult<Vec<ConfigItemDigest>> {
    let namespace_id = if namespace_id.is_empty() { "public" } else { namespace_id };

    let typed_inst = bs_inst.inst::<TardisRelDBClient>();
    let conns = conf_pg_initializer::init_table_and_conn(typed_inst, ctx, true).await?;
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

pub async fn get_configs(req: ConfigListRequest, mode: SearchMode, _funs: &TardisFunsInst, ctx: &TardisContext, bs_inst: &SpiBsInst) -> TardisResult<ConfigListResponse> {
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
    let typed_inst = bs_inst.inst::<TardisRelDBClient>();
    let conns = conf_pg_initializer::init_table_and_conn(typed_inst, ctx, true).await?;
    let (conn, config_table_name) = conns.config;
    let (_, config_tag_rel_table_name) = conns.config_tag_rel;
    let tag_condition_clause = if !tags.is_empty() {
        let tag_count = tags.len();
        let tags_placeholder = (values.len() + 1..=values.len() + tag_count).map(|idx| format!("${}", idx)).collect::<Vec<String>>().join(", ");
        values.extend(tags.iter().map(Value::from));
        format!(
            r#"id IN (
        SELECT config_id FROM {config_tag_rel_table_name} tcr
        WHERE tcr.tag_id IN ({tags_placeholder})
        GROUP BY config_id
        HAVING COUNT(config_id) = {tag_count})"#
        )
    } else {
        "".into()
    };
    let qry_result_list = conn
        .query_all(
            &format!(
                r#"SELECT 
    *, 
    COUNT(*) OVER () as total_count,
    ARRAY_TO_STRING(
		ARRAY(select tag_id from {config_tag_rel_table_name} tcr where tcr.config_id = c.id), ','
	) as tags
    FROM {config_table_name} c
    WHERE {cond}
    ORDER BY created_time DESC
    LIMIT {limit}
    OFFSET {offset}
    "#,
                cond = [where_clause.as_deref().unwrap_or("1=1"), tag_condition_clause.as_str()].join(" AND "),
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
                created_time: DateTimeUtc,
                modified_time: DateTimeUtc,
                grp: String,
                total_count: i64,
                src_user: Option<String>,
                tags: Option<String>,
            });
            total = total_count;
            Ok(ConfigItem {
                id: id.to_string(),
                data_id,
                namespace: namespace_id,
                md5,
                content,
                created_time,
                last_modified_time: modified_time,
                group: grp,
                src_user,
                config_tags: tags.map(|tags| tags.split(',').filter(|s| !s.is_empty()).map(String::from).collect()).unwrap_or_default(),
                ..Default::default()
            })
        })
        .collect::<TardisResult<Vec<_>>>()?;
    let total = total as u32;
    Ok(ConfigListResponse {
        total_count: total,
        page_number,
        pages_available: total / limit + 1,
        page_items: list,
    })
}
