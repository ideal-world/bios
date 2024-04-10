//! SPI initializer
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::{TardisFuns, TardisFunsInst};

use crate::rbum::dto::rbum_domain_dto::RbumDomainAddReq;
use crate::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumKindFilterReq};
use crate::rbum::dto::rbum_kind_dto::RbumKindAddReq;
use crate::rbum::rbum_enumeration::RbumScopeLevelKind;
use crate::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use crate::rbum::serv::rbum_domain_serv::RbumDomainServ;
use crate::rbum::serv::rbum_kind_serv::RbumKindServ;

use super::domain::spi_bs;

/// SPI service initialization
/// SPI服务初始化
///
/// Initialize ``rbum_domain`` for different SPI services
/// 初始化不同SPI服务的``rbum_domain``
pub async fn init(code: &str, funs: &TardisFunsInst) -> TardisResult<TardisContext> {
    let ctx = TardisContext {
        own_paths: "".to_string(),
        ak: "_".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "".to_string(),
        ..Default::default()
    };
    if RbumDomainServ::get_rbum_domain_id_by_code(code, funs).await?.is_some() {
        return Ok(ctx);
    }
    // Initialize spi component RBUM item table and indexes
    funs.db().init(spi_bs::ActiveModel::init(TardisFuns::reldb().backend(), None, TardisFuns::reldb().compatible_type())).await?;
    // Initialize spi component RBUM domain data
    RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: TrimString(code.to_string()),
            name: TrimString(code.to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: Some(RbumScopeLevelKind::Root),
        },
        funs,
        &ctx,
    )
    .await?;
    Ok(ctx)
}

/// Add the type of the SPI service backend implementation
/// 添加SPI服务后端实现的类型
pub async fn add_kind(scheme: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    if !RbumKindServ::exist_rbum(
        &RbumKindFilterReq {
            basic: RbumBasicFilterReq {
                code: Some(scheme.to_string()),
                ..Default::default()
            },
            ..Default::default()
        },
        funs,
        ctx,
    )
    .await?
    {
        RbumKindServ::add_rbum(
            &mut RbumKindAddReq {
                code: TrimString(scheme.to_string()),
                name: TrimString(scheme.to_string()),
                note: None,
                icon: None,
                sort: None,
                module: None,
                ext_table_name: Some("spi_bs".to_lowercase()),
                scope_level: Some(RbumScopeLevelKind::Root),
            },
            funs,
            ctx,
        )
        .await?;
    }
    Ok(())
}

/// Some common initialization helper methods
/// 一些公共的初始化辅助方法
///
/// # Mainly includes the following functions:
/// 1. Isolation flag processing.
///    The isolation identifier is the data used to distinguish the subject of the request (tenant or application).
///    For example, application a and application b use the same cache service instance and both use the same cache key.
///    To avoid data confusion, you can use the isolation flag as part of the cache key to distinguish different application data.
///    For the private [`crate::spi::domain::spi_bs::Model::private`] backend implementation instance, the isolation flag is not required,
///    because the private instance will only be used by one subject of the request.
///
/// # 主要包含如下功能：
/// 1. 隔离标识处理。隔离标识是用于区分请求主体（租户或应用）的数据。
///    比如应用a与应用b使用相同的缓存服务实例且都使用了相同的缓存key，为了避免数据混乱，可以通过隔离标识作为缓存key的一部分来区分不同的应用数据。
///    对于私有 [`crate::spi::domain::spi_bs::Model::private`] 的后端实现实例不需要隔离标识，因为私有实例只会被一个请求主体使用。
pub mod common {
    use std::collections::HashMap;

    use tardis::{basic::dto::TardisContext, TardisFuns};

    use crate::spi::spi_constants;

    /// Get the context's ``owner`` as the isolation flag
    /// 获取上下文的``owner``作为隔离标识
    pub fn get_isolation_flag_from_context(ctx: &TardisContext) -> String {
        // Fix case insensitivity
        format!("spi{}", TardisFuns::crypto.hex.encode(&ctx.owner))
    }

    /// Set the isolation flag to the extension
    /// 将隔离标识设置到扩展中
    pub fn set_isolation_flag_to_ext(isolation_flag: &str, ext: &mut HashMap<String, String>) {
        ext.insert(spi_constants::SPI_ISOLATION_FLAG.to_string(), isolation_flag.to_string());
    }

    /// Get the isolation flag from the extension
    /// 从扩展中获取隔离标识
    pub fn get_isolation_flag_from_ext(ext: &HashMap<String, String>) -> Option<String> {
        ext.get(spi_constants::SPI_ISOLATION_FLAG).map(|s| s.to_string())
    }
}

/// Some common postgresql initialization helper methods
/// 一些公共的PostgreSQL初始化辅助方法
pub mod common_pg {
    use std::collections::HashMap;

    use tardis::{
        basic::{dto::TardisContext, error::TardisError, result::TardisResult},
        config::config_dto::DBModuleConfig,
        db::{
            reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
            sea_orm::Value,
        },
        TardisFuns,
    };

    use crate::spi::{
        dto::spi_bs_dto::SpiBsCertResp,
        spi_constants::GLOBAL_STORAGE_FLAG,
        spi_funs::{SpiBsInst, TypedSpiBsInst},
    };

    use super::common;

    /// Get the schema name from the context
    /// 从上下文中获取schema名称
    ///
    /// Each subject of the request (tenant or application) data will be isolated into different schemas, and the schema name is determined by the isolation flag
    /// 每个请求主体（租户或应用）的数据都会被隔离到不同的schema中，schema的名称由隔离标识决定
    pub fn get_schema_name_from_context(ctx: &TardisContext) -> String {
        common::get_isolation_flag_from_context(ctx)
    }

    /// Set the schema name to the extension
    /// 将schema名称设置到扩展中
    ///
    /// Each subject of the request (tenant or application) data will be isolated into different schemas, and the schema name is determined by the isolation flag
    /// 每个请求主体（租户或应用）的数据都会被隔离到不同的schema中，schema的名称由隔离标识决定
    pub fn set_schema_name_to_ext(schema_name: &str, ext: &mut HashMap<String, String>) {
        common::set_isolation_flag_to_ext(schema_name, ext);
    }

    /// Get the schema name from the extension
    /// 从扩展中获取schema名称
    ///
    /// Each subject of the request (tenant or application) data will be isolated into different schemas, and the schema name is determined by the isolation flag
    /// 每个请求主体（租户或应用）的数据都会被隔离到不同的schema中，schema的名称由隔离标识决定
    pub fn get_schema_name_from_ext(ext: &HashMap<String, String>) -> Option<String> {
        common::get_isolation_flag_from_ext(ext)
    }

    /// Check if the schema exists
    /// 检查schema是否存在
    pub async fn check_schema_exit(client: &TardisRelDBClient, ctx: &TardisContext) -> TardisResult<bool> {
        let schema_name = get_schema_name_from_context(ctx);
        let schema = client.conn().count_by_sql("SELECT 1 FROM information_schema.schemata WHERE schema_name = $1", vec![Value::from(schema_name.as_str())]).await?;
        Ok(schema != 0)
    }

    /// Create schema
    /// 创建schema
    pub async fn create_schema(client: &TardisRelDBClient, ctx: &TardisContext) -> TardisResult<String> {
        let schema_name = get_schema_name_from_context(ctx);
        if !check_schema_exit(client, ctx).await? {
            client.conn().execute_one(&format!("CREATE SCHEMA {schema_name}"), vec![]).await?;
        }
        Ok(schema_name)
    }

    /// Check if the table exists
    /// 检查表是否存在
    ///
    /// When checking, the schema corresponding to the request will be added
    /// 检查时会加上与请求对应的schema
    pub async fn check_table_exit(table_name: &str, conn: &TardisRelDBlConnection, ctx: &TardisContext) -> TardisResult<bool> {
        let schema_name = get_schema_name_from_context(ctx);
        let table = conn
            .count_by_sql(
                "SELECT 1 FROM information_schema.tables WHERE table_schema = $1 AND table_name = $2",
                vec![Value::from(schema_name.as_str()), Value::from(format!("{GLOBAL_STORAGE_FLAG}_{table_name}"))],
            )
            .await?;
        Ok(table != 0)
    }

    /// Set the schema to the session
    /// 将schema设置到会话中
    ///
    /// After setting, the operation of the current connection does not need to add the schema prefix
    /// 设置后当前连接的操作不需要再加上schema前缀
    pub async fn set_schema_to_session(schema_name: &str, conn: &mut TardisRelDBlConnection) -> TardisResult<()> {
        conn.begin().await?;
        conn.execute_one(&format!("SET SCHEMA '{schema_name}'"), vec![]).await?;
        Ok(())
    }

    /// Get the formatted table name
    /// 获取格式化后的表名
    ///
    /// Format: {schema_name}.{GLOBAL_STORAGE_FLAG}_{table_name}
    /// 格式为：{schema_name}.{GLOBAL_STORAGE_FLAG}_{table_name}
    pub fn package_table_name(table_name: &str, ctx: &TardisContext) -> String {
        let schema_name = get_schema_name_from_context(ctx);
        format!("{schema_name}.{GLOBAL_STORAGE_FLAG}_{table_name}")
    }

    /// Initialize the backend implementation instance of PostgreSQL
    /// 初始化PostgreSQL的后端实现实例
    pub async fn init(bs_cert: &SpiBsCertResp, ctx: &TardisContext, mgr: bool) -> TardisResult<SpiBsInst> {
        let ext = TardisFuns::json.str_to_json(&bs_cert.ext)?;
        let compatible_type = TardisFuns::json.json_to_obj(ext.get("compatible_type").unwrap_or(&tardis::serde_json::Value::String("None".to_string())).clone())?;
        let client = TardisRelDBClient::init(&DBModuleConfig {
            url: bs_cert.conn_uri.parse().expect("invalid url"),
            max_connections: ext.get("max_connections").unwrap().as_u64().unwrap() as u32,
            min_connections: ext.get("min_connections").unwrap().as_u64().unwrap() as u32,
            connect_timeout_sec: None,
            idle_timeout_sec: None,
            compatible_type,
        })
        .await?;
        let mut ext = HashMap::new();
        // If the connection is private, the isolation (schema) does not need to be processed, so use public directly.
        // 如果连接为私有的，不需要处理隔离（schema），故直接使用public
        let schema_name = if bs_cert.private {
            "public".to_string()
        } else if mgr {
            // Only in management mode can the schema be created
            // 仅管理模式下才能创建schema
            create_schema(&client, ctx).await?
        } else if check_schema_exit(&client, ctx).await? {
            get_schema_name_from_context(ctx)
        } else {
            return Err(TardisError::bad_request("The requested schema does not exist", ""));
        };
        set_schema_name_to_ext(&schema_name, &mut ext);
        Ok(SpiBsInst { client: Box::new(client), ext })
    }

    /// Initialize the table and connection
    /// 初始化表和连接
    pub async fn init_table_and_conn(
        // Database connection client instance
        bs_inst: TypedSpiBsInst<'_, TardisRelDBClient>,
        ctx: &TardisContext,
        // If it is management mode
        mgr: bool,
        // Tag, as a table name suffix
        tag: Option<&str>,
        // Table flag, as part of the table name
        table_flag: &str,
        // Create table DDL
        table_create_content: &str,
        // Table index
        // Format: field name -> index type
        indexes: Vec<(&str, &str)>,
        // Primary keys
        primary_keys: Option<Vec<&str>>,
        // Update time field, used to create a trigger that automatically updates the time
        update_time_field: Option<&str>,
    ) -> TardisResult<(TardisRelDBlConnection, String)> {
        let tag = tag.map(|t| format!("_{t}")).unwrap_or_default();
        let conn = bs_inst.0.conn();
        let schema_name = get_schema_name_from_ext(bs_inst.1).unwrap();
        if check_table_exit(&format!("{table_flag}{tag}"), &conn, ctx).await? {
            return Ok((conn, format!("{schema_name}.{GLOBAL_STORAGE_FLAG}_{table_flag}{tag}")));
        } else if !mgr {
            return Err(TardisError::bad_request("The requested tag does not exist", ""));
        }
        do_init_table(&schema_name, &conn, &tag, table_flag, table_create_content, indexes, primary_keys, update_time_field).await?;
        Ok((conn, format!("{schema_name}.{GLOBAL_STORAGE_FLAG}_{table_flag}{tag}")))
    }

    /// Initialize connection
    /// 初始化连接
    pub async fn init_conn(bs_inst: TypedSpiBsInst<'_, TardisRelDBClient>) -> TardisResult<(TardisRelDBlConnection, String)> {
        let conn = bs_inst.0.conn();
        let schema_name = get_schema_name_from_ext(bs_inst.1).unwrap();
        Ok((conn, schema_name))
    }

    /// Initialize table
    /// 初始化表
    pub async fn init_table(
        conn: &TardisRelDBlConnection,
        tag: Option<&str>,
        table_flag: &str,
        table_create_content: &str,
        // field_name_or_fun -> index type
        indexes: Vec<(&str, &str)>,
        primary_keys: Option<Vec<&str>>,
        update_time_field: Option<&str>,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let tag = tag.map(|t| format!("_{t}")).unwrap_or_default();
        let schema_name = get_schema_name_from_context(ctx);
        do_init_table(&schema_name, conn, &tag, table_flag, table_create_content, indexes, primary_keys, update_time_field).await
    }

    async fn do_init_table(
        schema_name: &str,
        conn: &TardisRelDBlConnection,
        tag: &str,
        table_flag: &str,
        table_create_content: &str,
        // field_name_or_fun -> index type
        indexes: Vec<(&str, &str)>,
        primary_keys: Option<Vec<&str>>,
        update_time_field: Option<&str>,
    ) -> TardisResult<()> {
        conn.execute_one(
            &format!(
                r#"CREATE TABLE {schema_name}.{GLOBAL_STORAGE_FLAG}_{table_flag}{tag}
(
    {table_create_content}
)"#
            ),
            vec![],
        )
        .await?;
        for (idx, (field_name_or_fun, index_type)) in indexes.into_iter().enumerate() {
            // index name shouldn't be longer than 63 characters
            // [4 ][     18    ][ 12 ][     26    ][ 3 ]
            // idx_{schema_name}{tag}_{table_flag}_{idx}
            #[inline]
            fn truncate_str(s: &str, max_size: usize) -> &str {
                &s[..max_size.min(s.len())]
            }
            let index_name = format!(
                "idx_{schema_name}{tag}_{table_flag}_{idx}",
                schema_name = truncate_str(schema_name, 18),
                tag = truncate_str(tag, 11),
                table_flag = truncate_str(table_flag, 25),
                idx = truncate_str(idx.to_string().as_str(), 3),
            );
            conn.execute_one(
                &format!("CREATE INDEX {index_name} ON {schema_name}.{GLOBAL_STORAGE_FLAG}_{table_flag}{tag} USING {index_type}({field_name_or_fun})"),
                vec![],
            )
            .await?;
        }
        if let Some(primary_keys) = primary_keys {
            let pks = primary_keys.join(", ");
            conn.execute_one(
                &format!(r#"ALTER TABLE {schema_name}.{GLOBAL_STORAGE_FLAG}_{table_flag}{tag} ADD PRIMARY KEY ({pks})"#),
                vec![],
            )
            .await?;
        }
        if let Some(update_time_field) = update_time_field {
            conn.execute_one(
                &format!(
                    r###"CREATE OR REPLACE FUNCTION TARDIS_AUTO_UPDATE_TIME_{}()
RETURNS TRIGGER AS $$
BEGIN
    NEW.{} = now();
    RETURN NEW;
END;
$$ language 'plpgsql';"###,
                    update_time_field.replace('-', "_"),
                    update_time_field
                ),
                vec![],
            )
            .await?;
            conn.execute_one(
                &format!(
                    r###"CREATE OR REPLACE TRIGGER TARDIS_AUTO_UPDATE_TIME_ON
    BEFORE UPDATE
    ON
        {}.{GLOBAL_STORAGE_FLAG}_{}{}
    FOR EACH ROW
EXECUTE PROCEDURE TARDIS_AUTO_UPDATE_TIME_{}();"###,
                    schema_name,
                    table_flag,
                    tag,
                    update_time_field.replace('-', "_")
                ),
                vec![],
            )
            .await?;
        }
        Ok(())
    }
}
