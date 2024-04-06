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
    TardisFunsInst,
};

use crate::{
    conf_constants::error,
    dto::conf_config_dto::{ConfigDescriptor, ConfigHistoryListRequest, ConfigItem, ConfigListResponse},
};

use super::conf_pg_initializer;

#[derive(Debug, Default)]
pub struct HistoryInsertParams<'a> {
    pub data_id: &'a str,
    pub group: &'a str,
    pub namespace: &'a str,
    pub content: &'a str,
    pub md5: &'a str,
    pub app_name: Option<&'a str>,
    pub schema: Option<&'a str>,
    pub config_tags: Vec<&'a str>,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpType {
    Insert,
    Update,
    Delete,
}

impl OpType {
    pub fn as_char(self) -> char {
        match self {
            OpType::Insert => 'I',
            OpType::Update => 'U',
            OpType::Delete => 'D',
        }
    }
}

macro_rules! get {
    ($result:expr => {$($name:ident: $type:ty,)*}) => {
        $(let $name = $result.try_get::<$type>("", stringify!($name))?;)*
    };
}

pub async fn get_history_list_by_namespace(
    req: &mut ConfigHistoryListRequest,
    _funs: &TardisFunsInst,
    ctx: &TardisContext,
    bs_inst: &SpiBsInst,
) -> TardisResult<ConfigListResponse> {
    // query config history list by a ConfigDescriptor
    let ConfigHistoryListRequest { descriptor, page_no, page_size } = req;
    descriptor.fix_namespace_id();
    let limit = (*page_size).min(500).max(1);
    let page_number = (*page_no).max(1);
    let offset = (page_number - 1) * limit;
    let data_id = &descriptor.data_id;
    let group = &descriptor.group;
    let namespace_id = &descriptor.namespace_id;
    let bs_inst = bs_inst.inst::<TardisRelDBClient>();
    let conns = conf_pg_initializer::init_table_and_conn(bs_inst, ctx, true).await?;
    let (conn, table_name) = conns.config_history;
    let qry_result_list = conn
        .query_all(
            &format!(
                r#"SELECT *, count(*) over () as total_count FROM {table_name} cch
WHERE cch.namespace_id=$1 AND cch.grp=$2 AND cch.data_id=$3
ORDER BY created_time DESC
LIMIT {limit}
OFFSET {offset}
"#,
            ),
            vec![Value::from(namespace_id), Value::from(group), Value::from(data_id)],
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
                config_tags: String,
                total_count: i64,
            });
            total = total_count as u32;
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
                config_tags: config_tags.split(',').filter(|s| !s.is_empty()).map(String::from).collect(),
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

pub async fn find_history(descriptor: &mut ConfigDescriptor, id: &Uuid, _funs: &TardisFunsInst, ctx: &TardisContext, bs_inst: &SpiBsInst) -> TardisResult<ConfigItem> {
    descriptor.fix_namespace_id();
    let data_id = &descriptor.data_id;
    let group = &descriptor.group;
    let namespace_id = &descriptor.namespace_id;
    let typed_inst = bs_inst.inst::<TardisRelDBClient>();
    let conns = conf_pg_initializer::init_table_and_conn(typed_inst, ctx, true).await?;
    let (conn, table_name) = conns.config_history;
    let qry_result = conn
        .query_one(
            &format!(
                r#"SELECT id, data_id, namespace_id, md5, content, src_user, op_type, created_time, modified_time, grp, config_tags FROM {table_name} cch
WHERE cch.namespace_id=$1 AND cch.grp=$2 AND cch.data_id=$3 AND cch.id=$4
ORDER BY created_time DESC"#,
            ),
            vec![Value::from(namespace_id), Value::from(group), Value::from(data_id), Value::from(*id)],
        )
        .await?
        .ok_or_else(|| TardisError::not_found("history config not found", error::CONF_NOTFOUND))?;
    get!(qry_result => {
        id: Uuid,
        data_id: String,
        namespace_id: String,
        md5: String,
        content: String,
        op_type: String,
        created_time: DateTimeUtc,
        modified_time: DateTimeUtc,
        src_user: Option<String>,
        grp: String,
        config_tags: String,
    });
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
        src_user,
        config_tags: config_tags.split(',').filter(|s| !s.is_empty()).map(String::from).collect(),
        ..Default::default()
    })
}

pub async fn find_previous_history(descriptor: &mut ConfigDescriptor, id: &Uuid, funs: &TardisFunsInst, ctx: &TardisContext, bs_inst: &SpiBsInst) -> TardisResult<ConfigItem> {
    descriptor.fix_namespace_id();
    // find previous config by id
    // 1. find previous id
    let data_id = &descriptor.data_id;
    let group = &descriptor.group;
    let namespace_id = &descriptor.namespace_id;
    let typed_inst = bs_inst.inst::<TardisRelDBClient>();
    let conns = conf_pg_initializer::init_table_and_conn(typed_inst, ctx, true).await?;
    let (conn, table_name) = conns.config_history;
    let qry_result = conn
        .query_one(
            &format!(
                r#"SELECT prev_id FROM (
    SELECT 
        id,
        LAG(id) OVER (ORDER BY created_time ASC) AS prev_id
    FROM {table_name} cch
    WHERE cch.namespace_id=$1 AND cch.grp=$2 AND cch.data_id=$3
) AS T
WHERE T.id = $4
"#,
            ),
            vec![Value::from(namespace_id), Value::from(group), Value::from(data_id), Value::from(*id)],
        )
        .await?
        .ok_or_else(|| TardisError::not_found("history config not found", error::CONF_NOTFOUND))?;
    get!(qry_result => { prev_id: Option<Uuid>, });
    if let Some(prev_id) = prev_id {
        // 2. find config by id
        self::find_history(descriptor, &prev_id, funs, ctx, bs_inst).await
    } else {
        Err(TardisError::not_found("history config not found", error::CONF_NOTFOUND))
    }
}
pub async fn add_history(param: HistoryInsertParams<'_>, op_type: OpType, _funs: &TardisFunsInst, ctx: &TardisContext, bs_inst: &SpiBsInst) -> TardisResult<bool> {
    let HistoryInsertParams {
        data_id,
        group,
        namespace,
        content,
        md5,
        app_name,
        schema,
        config_tags,
    } = param;
    let src_user = &ctx.owner;

    let params = vec![
        ("data_id", Value::from(data_id)),
        ("grp", Value::from(group)),
        ("namespace_id", Value::from(namespace)),
        ("content", Value::from(content)),
        ("md5", Value::from(md5)),
        ("app_name", Value::from(app_name)),
        ("schema", Value::from(schema)),
        ("op_type", Value::from(op_type.as_char())),
        ("src_user", Value::from(src_user)),
        ("config_tags", Value::from(config_tags.join(","))),
    ];
    let typed_inst = bs_inst.inst::<TardisRelDBClient>();
    let conns = conf_pg_initializer::init_table_and_conn(typed_inst, ctx, true).await?;
    let (mut conn, table_name) = conns.config_history;
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
