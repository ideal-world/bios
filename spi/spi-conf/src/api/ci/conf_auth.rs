use bios_basic::TardisFunInstExtractor;
use tardis::web::{
    context_extractor::TardisContextExtractor,
    poem::Request,
    poem_openapi::{self, payload::Json},
    web_resp::{TardisApiResult, TardisResp},
};

use crate::dto::conf_auth_dto::*;
use crate::serv::*;

#[derive(Default)]
pub struct ConfCiAuthApi;

#[poem_openapi::OpenApi(prefix_path = "/ci/auth", tag = "bios_basic::ApiTag::Interface")]
impl ConfCiAuthApi {
    #[oai(path = "/register", method = "post")]
    async fn register(&self, json: Json<RegisterRequest>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<RegisterResponse> {
        let reg_req = json.0;
        let funs = request.tardis_fun_inst();
        let resp = register(reg_req, &funs, &ctx.0).await?;
        TardisResp::ok(resp)
    }
}
