use tardis::serde_json::Value;
use tardis::web::context_extractor::TardisContextExtractor;

use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use crate::dto::reldb_exec_dto::{ReldbDdlReq, ReldbDmlReq, ReldbDmlResp, ReldbDqlReq, ReldbTxResp};
use crate::serv::reldb_exec_serv;

#[derive(Clone)]
pub struct ReldbCiExecApi;

/// Interface Console RelDB Execute API
#[poem_openapi::OpenApi(prefix_path = "/ci/exec", tag = "bios_basic::ApiTag::Interface")]
impl ReldbCiExecApi {
    /// Fetch Transaction ID
    #[oai(path = "/tx", method = "get")]
    async fn tx_begin(&self, auto_commit: Query<bool>, exp_sec: Query<Option<u8>>, ctx: TardisContextExtractor) -> TardisApiResult<ReldbTxResp> {
        let funs = crate::get_tardis_inst();
        let resp = reldb_exec_serv::tx_begin(auto_commit.0, exp_sec.0, &funs, &ctx.0).await?;
        TardisResp::ok(resp)
    }

    /// Commit Transaction
    #[oai(path = "/tx", method = "put")]
    async fn tx_commit(&self, tx_id: Query<String>) -> TardisApiResult<Void> {
        reldb_exec_serv::tx_commit(tx_id.0).await?;
        TardisResp::ok(Void {})
    }

    /// Rollack Transaction
    #[oai(path = "/tx", method = "delete")]
    async fn tx_rollback(&self, tx_id: Query<String>) -> TardisApiResult<Void> {
        reldb_exec_serv::tx_rollback(tx_id.0).await?;
        TardisResp::ok(Void {})
    }

    /// DDL
    #[oai(path = "/ddl", method = "post")]
    async fn ddl(&self, mut ddl_req: Json<ReldbDdlReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        reldb_exec_serv::ddl(&mut ddl_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// DML
    #[oai(path = "/dml", method = "post")]
    async fn dml(&self, mut dml_req: Json<ReldbDmlReq>, tx_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<ReldbDmlResp> {
        let funs = crate::get_tardis_inst();
        let resp = reldb_exec_serv::dml(&mut dml_req.0, tx_id.0, &funs, &ctx.0).await?;
        TardisResp::ok(resp)
    }

    /// DQL
    #[oai(path = "/dql", method = "put")]
    async fn dql(&self, mut dql_req: Json<ReldbDqlReq>, tx_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<Vec<Value>> {
        let funs = crate::get_tardis_inst();
        let resp = reldb_exec_serv::dql(&mut dql_req.0, tx_id.0, &funs, &ctx.0).await?;
        TardisResp::ok(resp)
    }
}
