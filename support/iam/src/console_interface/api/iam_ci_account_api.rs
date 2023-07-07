use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::{param::Path, param::Query};
use tardis::web::web_resp::{TardisApiResult, TardisResp};
use tardis::TardisFuns;

use crate::basic::serv::iam_key_cache_serv::IamIdentCacheServ;
use crate::iam_constants;

#[derive(Clone, Default)]
pub struct IamCiAccountApi;

/// Interface Console Account API
#[poem_openapi::OpenApi(prefix_path = "/ci/account", tag = "bios_basic::ApiTag::Interface")]
impl IamCiAccountApi {
    /// Get Context By Account Id
    #[oai(path = "/:id/ctx", method = "get")]
    async fn get_account_context(&self, id: Path<String>, app_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<String> {
        let funs = iam_constants::get_tardis_inst();
        let mut ctx_resp = IamIdentCacheServ::get_account_context(&id.0, &app_id.0.unwrap_or((&"").to_string()), &funs).await?;
        ctx_resp.own_paths = ctx.0.own_paths;
        TardisResp::ok(TardisFuns::crypto.base64.encode(&TardisFuns::json.obj_to_string(&ctx_resp).unwrap_or_default()))
    }
}
