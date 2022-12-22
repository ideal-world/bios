use bios_basic::spi::spi_funs::SpiBsInstExtractor;
use tardis::basic::result::TardisResult;
use tardis::db::sea_orm::{FromQueryResult, Value};
use tardis::{basic::dto::TardisContext, db::reldb_client::TardisRelDBClient};
use tardis::{serde_json, TardisFunsInst};

use crate::dto::reldb_exec_dto::{ReldbDdlReq, ReldbDmlReq, ReldbDmlResp, ReldbDqlReq};
use crate::reldb_initializer;

pub struct ReldbExecServ;

impl ReldbExecServ {
    fn parse_params(params: &serde_json::Value) -> Vec<Value> {
        params
            .as_array()
            .unwrap()
            .iter()
            .map(|item| {
                if item.is_string() {
                    Value::from(item.as_str().unwrap())
                } else if item.is_boolean() {
                    Value::from(item.as_bool().unwrap())
                } else if item.is_u64() {
                    Value::from(item.as_u64().unwrap())
                } else if item.is_f64() {
                    Value::from(item.as_f64().unwrap())
                } else if item.is_i64() {
                    Value::from(item.as_i64().unwrap())
                } else {
                    // TODO
                    Value::from(item.as_str().unwrap())
                }
            })
            .collect::<Vec<Value>>()
    }

    pub async fn ddl(ddl_req: &mut ReldbDdlReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let (client, ext) = funs.bs(ctx, reldb_initializer::init_fun).await?.inst::<TardisRelDBClient>();
        let params = Self::parse_params(&ddl_req.params);
        client.conn().execute_one(&ddl_req.sql, params).await?;
        Ok(())
    }

    pub async fn dml(dml_req: &mut ReldbDmlReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<ReldbDmlResp> {
        let (client, ext) = funs.bs(ctx, reldb_initializer::init_fun).await?.inst::<TardisRelDBClient>();
        let params = Self::parse_params(&dml_req.params);
        let resp = client.conn().execute_one(&dml_req.sql, params).await?;
        Ok(ReldbDmlResp {
            affected_rows: resp.rows_affected(),
        })
    }

    pub async fn dql(dql_req: &mut ReldbDqlReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<serde_json::Value>> {
        let (client, ext) = funs.bs(ctx, reldb_initializer::init_fun).await?.inst::<TardisRelDBClient>();
        let params = Self::parse_params(&dql_req.params);
        let resp = client.conn().query_all(&dql_req.sql, params).await?;
        let mut result: Vec<serde_json::Value> = Vec::new();
        for row in resp {
            let json = serde_json::Value::from_query_result_optional(&row, "")?.unwrap();
            result.push(json);
        }
        Ok(result)
    }
}
