use serde::Serialize;
use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::web::poem_openapi::types::{ParseFromJSON, ToJSON};
use tardis::web::web_client::TardisHttpResponse;
use tardis::web::web_resp::TardisResp;
use tardis::{TardisFuns, TardisFunsInst};

use crate::invoke_config::{InvokeConfig, InvokeConfigTrait};
use crate::invoke_constants::TARDIS_CONTEXT;
use crate::invoke_enumeration::InvokeModuleKind;

pub struct BaseSpiClient;

impl BaseSpiClient {
    pub async fn module_url<C: InvokeConfigTrait + 'static>(module: InvokeModuleKind, funs: &TardisFunsInst) -> TardisResult<String> {
        if let Some(uri) = funs.conf::<C>().get_module_opt_url(module) {
            return Ok(uri.to_string());
        }
        Err(funs.err().conflict("spi-module", "spi_module", "spi module uri Not configured yet.", "400-spi-module-not-exist"))
    }

    pub async fn headers(headers: Option<Vec<(String, String)>>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<Vec<(String, String)>>> {
        let spi_ctx = TardisContext {
            owner: funs.conf::<InvokeConfig>().spi_app_id.clone(),
            ..ctx.clone()
        };
        let base_ctx = (TARDIS_CONTEXT.to_string(), TardisFuns::crypto.base64.encode(&TardisFuns::json.obj_to_string(&spi_ctx)?));
        if let Some(mut headers) = headers {
            headers.push(base_ctx);
            return Ok(Some(headers));
        }
        let headers = Some(vec![base_ctx]);
        Ok(headers)
    }

    pub fn package_resp<T>(result: TardisHttpResponse<TardisResp<T>>) -> TardisResult<Option<T>>
    where
        T: ParseFromJSON + ToJSON + Serialize + Send + Sync,
    {
        if result.code != 200 {
            return Err(TardisError::bad_request("Request failure", ""));
        }
        if let Some(body) = result.body {
            if !body.code.starts_with("200") {
                return Err(TardisError::custom(&body.code, &body.msg, ""));
            }
            return Ok(body.data);
        }
        Err(TardisError::bad_request("The requested schema does not exist", ""))
    }
}
