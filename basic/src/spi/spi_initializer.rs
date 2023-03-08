use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::{TardisFuns, TardisFunsInst};

use crate::rbum::dto::rbum_domain_dto::RbumDomainAddReq;
use crate::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use crate::rbum::dto::rbum_kind_dto::RbumKindAddReq;
use crate::rbum::rbum_enumeration::RbumScopeLevelKind;
use crate::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use crate::rbum::serv::rbum_domain_serv::RbumDomainServ;
use crate::rbum::serv::rbum_kind_serv::RbumKindServ;

use super::domain::spi_bs;

pub async fn init(code: &str, funs: &TardisFunsInst) -> TardisResult<TardisContext> {
    let ctx = TardisContext {
        own_paths: "".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "".to_string(),
        ..Default::default()
    };
    if RbumDomainServ::get_rbum_domain_id_by_code(code, funs).await?.is_some() {
        return Ok(ctx);
    }
    // Initialize spi component RBUM item table and indexs
    funs.db().init(spi_bs::ActiveModel::init(TardisFuns::reldb().backend(), None)).await?;
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

pub async fn add_kind(scheme: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    if !RbumKindServ::exist_rbum(
        &RbumBasicFilterReq {
            code: Some(scheme.to_string()),
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

pub mod common {
    use std::collections::HashMap;

    use tardis::{basic::dto::TardisContext, TardisFuns};

    use crate::spi::spi_constants;

    pub fn get_isolation_flag_from_context(ctx: &TardisContext) -> String {
        // Fix case insensitivity
        format!("spi{}", TardisFuns::crypto.hex.encode(&ctx.owner))
    }

    pub fn set_isolation_flag_to_ext(isolation_flag: &str, ext: &mut HashMap<String, String>) {
        ext.insert(spi_constants::SPI_ISOLATION_FLAG.to_string(), isolation_flag.to_string());
    }

    pub fn get_isolation_flag_from_ext(ext: &HashMap<String, String>) -> Option<String> {
        ext.get(spi_constants::SPI_ISOLATION_FLAG).map(|s| s.to_string())
    }
}

pub mod common_pg {
    use std::collections::HashMap;

    use tardis::{
        basic::{dto::TardisContext, error::TardisError, result::TardisResult},
        db::{
            reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
            sea_orm::Value,
        },
        TardisFuns,
    };

    use crate::spi::{dto::spi_bs_dto::SpiBsCertResp, spi_constants::GLOBAL_STORAGE_FLAG, spi_funs::SpiBsInst};

    use super::common;

    pub fn get_schema_name_from_context(ctx: &TardisContext) -> String {
        common::get_isolation_flag_from_context(ctx)
    }

    pub fn set_schema_name_to_ext(schema_name: &str, ext: &mut HashMap<String, String>) {
        common::set_isolation_flag_to_ext(schema_name, ext);
    }

    pub fn get_schema_name_from_ext(ext: &HashMap<String, String>) -> Option<String> {
        common::get_isolation_flag_from_ext(ext)
    }

    pub async fn check_schema_exit(client: &TardisRelDBClient, ctx: &TardisContext) -> TardisResult<bool> {
        let schema_name = get_schema_name_from_context(ctx);
        let schema = client.conn().count_by_sql("SELECT 1 FROM information_schema.schemata WHERE schema_name = $1", vec![Value::from(schema_name.as_str())]).await?;
        Ok(schema != 0)
    }

    pub async fn create_schema(client: &TardisRelDBClient, ctx: &TardisContext) -> TardisResult<String> {
        let schema_name = get_schema_name_from_context(ctx);
        if !check_schema_exit(client, ctx).await? {
            client.conn().execute_one(&format!("CREATE SCHEMA {schema_name}"), vec![]).await?;
        }
        Ok(schema_name)
    }

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

    pub async fn set_schema_to_session(schema_name: &str, conn: &mut TardisRelDBlConnection) -> TardisResult<()> {
        conn.begin().await?;
        conn.execute_one(&format!("SET SCHEMA '{schema_name}'"), vec![]).await?;
        Ok(())
    }

    pub fn package_table_name(table_name: &str, ctx: &TardisContext) -> String {
        let schema_name = get_schema_name_from_context(ctx);
        format!("{schema_name}.{GLOBAL_STORAGE_FLAG}_{table_name}")
    }

    pub async fn init(bs_cert: &SpiBsCertResp, ctx: &TardisContext, mgr: bool) -> TardisResult<SpiBsInst> {
        let ext = TardisFuns::json.str_to_json(&bs_cert.ext)?;
        let client = TardisRelDBClient::init(
            &bs_cert.conn_uri,
            ext.get("max_connections").unwrap().as_u64().unwrap() as u32,
            ext.get("min_connections").unwrap().as_u64().unwrap() as u32,
            None,
            None,
        )
        .await?;
        let mut ext = HashMap::new();
        let schema_name = if bs_cert.private {
            "public".to_string()
        } else if mgr {
            create_schema(&client, ctx).await?
        } else if check_schema_exit(&client, ctx).await? {
            get_schema_name_from_context(ctx)
        } else {
            return Err(TardisError::bad_request("The requested schema does not exist", ""));
        };
        set_schema_name_to_ext(&schema_name, &mut ext);
        Ok(SpiBsInst { client: Box::new(client), ext })
    }

    /// return db connection and table name
    pub async fn init_table_and_conn(
        bs_inst: (&TardisRelDBClient, &HashMap<String, String>, String),
        ctx: &TardisContext,
        mgr: bool,
        tag: Option<&str>,
        table_flag: &str,
        table_create_content: &str,
        // field name -> index type
        indexes: Vec<(&str, &str)>,
        primary_keys: Option<Vec<&str>>,
        update_time_field: Option<&str>,
    ) -> TardisResult<(TardisRelDBlConnection, String)> {
        let tag = tag.map(|t| format!("_{t}")).unwrap_or_else(|| "".to_string());
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

    /// return db connection and schema name
    pub async fn init_conn(bs_inst: (&TardisRelDBClient, &HashMap<String, String>, String)) -> TardisResult<(TardisRelDBlConnection, String)> {
        let conn = bs_inst.0.conn();
        let schema_name = get_schema_name_from_ext(bs_inst.1).unwrap();
        Ok((conn, schema_name))
    }

    pub async fn init_table(
        conn: &TardisRelDBlConnection,
        tag: Option<&str>,
        table_flag: &str,
        table_create_content: &str,
        // field name -> index type
        indexes: Vec<(&str, &str)>,
        primary_keys: Option<Vec<&str>>,
        update_time_field: Option<&str>,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let tag = tag.map(|t| format!("_{t}")).unwrap_or_else(|| "".to_string());
        let schema_name = get_schema_name_from_context(ctx);
        do_init_table(&schema_name, conn, &tag, table_flag, table_create_content, indexes, primary_keys, update_time_field).await
    }

    async fn do_init_table(
        schema_name: &str,
        conn: &TardisRelDBlConnection,
        tag: &str,
        table_flag: &str,
        table_create_content: &str,
        // field name -> index type
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
        for (field_name_or_fun, index_type) in indexes {
            let index_part = field_name_or_fun.replace(|c: char| !c.is_ascii_alphanumeric(), "_");
            conn.execute_one(
                &format!("CREATE INDEX idx_{schema_name}{tag}_{table_flag}_{index_part} ON {schema_name}.{GLOBAL_STORAGE_FLAG}_{table_flag}{tag} USING {index_type}({field_name_or_fun})"),
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
                    r###"CREATE OR REPLACE FUNCTION TARDIS_AUTO_UPDATE_ITME_{}()
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
                    r###"CREATE OR REPLACE TRIGGER TARDIS_ATUO_UPDATE_TIME_ON
    BEFORE UPDATE
    ON
        {}.{GLOBAL_STORAGE_FLAG}_{}{}
    FOR EACH ROW
EXECUTE PROCEDURE TARDIS_AUTO_UPDATE_ITME_{}();"###,
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
