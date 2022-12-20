use bios_basic::spi::spi_funs::SpiTardisFunInstExtractor;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use crate::dto::reldb_exec_dto::{ReldbDmlReq, ReldbDmlResp};
use crate::serv::reldb_exec_serv::ReldbExecServ;

pub struct ReldbCiExecApi;

/// Interface Console RelDB Execute API
#[poem_openapi::OpenApi(prefix_path = "/ci/exec", tag = "bios_basic::ApiTag::Interface")]
impl ReldbCiExecApi {
    /// DML
    #[oai(path = "/dml", method = "post")]
    async fn dml(&self, mut dml_req: Json<ReldbDmlReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<ReldbDmlResp> {
        let mut funs = request.tardis_fun_inst();
        let resp = ReldbExecServ::dml(&mut dml_req.0, &mut funs, &ctx.0).await?;
        TardisResp::ok(resp)
    }
}
