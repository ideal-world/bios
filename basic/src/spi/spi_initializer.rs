use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::{TardisFuns, TardisFunsInst};

use crate::rbum::dto::rbum_domain_dto::RbumDomainAddReq;
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
    funs.db().init(spi_bs::ActiveModel::init(TardisFuns::reldb().backend(), None)).await?;
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

pub async fn add_kind(scheme: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
    RbumKindServ::add_rbum(
        &mut RbumKindAddReq {
            code: TrimString(scheme.to_string()),
            name: TrimString(scheme.to_string()),
            note: None,
            icon: None,
            sort: None,
            ext_table_name: Some("spi_bs".to_lowercase()),
            scope_level: Some(RbumScopeLevelKind::Root),
        },
        funs,
        ctx,
    )
    .await
}

pub mod common_pg {
    use std::collections::HashMap;

    use tardis::{
        basic::{dto::TardisContext, result::TardisResult},
        db::{
            reldb_client::{TardisRelDBClient, TardisRelDBlConnection},
            sea_orm::Value,
        },
        TardisFuns,
    };

    use crate::spi::spi_constants;

    pub async fn init_pg_schema(client: &TardisRelDBClient, ctx: &TardisContext) -> TardisResult<String> {
        // Fix case insensitivity
        let schema_name = format!("spi_{}", TardisFuns::crypto.hex.encode(&ctx.owner));
        let schema = client.conn().query_one("SELECT 1 FROM information_schema.schemata WHERE schema_name = $1", vec![Value::from(schema_name.as_str())]).await?;
        if schema.is_none() {
            client.conn().execute_one(&format!("CREATE SCHEMA {}", schema_name), vec![]).await?;
        }
        Ok(schema_name)
    }

    pub async fn check_table_exit(table_name: &str, conn: &TardisRelDBlConnection, ctx: &TardisContext) -> TardisResult<bool> {
        // Fix case insensitivity
        let schema_name = format!("spi_{}", TardisFuns::crypto.hex.encode(&ctx.owner));
        let table = conn
            .query_one(
                "SELECT 1 FROM information_schema.tables WHERE table_schema = $1 AND table_name = $2",
                vec![Value::from(schema_name.as_str()), Value::from(table_name)],
            )
            .await?;
        Ok(table.is_some())
    }

    pub fn set_pg_schema_to_ext(schema_name: &str, ext: &mut HashMap<String, String>) {
        ext.insert(spi_constants::SPI_PG_SCHEMA_NAME_FLAG.to_string(), schema_name.to_string());
    }

    pub fn get_pg_schema_from_ext(ext: &HashMap<String, String>) -> Option<String> {
        ext.get(spi_constants::SPI_PG_SCHEMA_NAME_FLAG).map(|s| s.to_string())
    }

    pub async fn set_pg_schema_to_session(schema_name: &str, conn: &TardisRelDBlConnection) -> TardisResult<()> {
        conn.execute_one(&format!("SET SCHEMA '{}'", schema_name), vec![]).await?;
        Ok(())
    }
}
