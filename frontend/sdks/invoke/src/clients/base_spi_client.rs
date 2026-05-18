use serde::{Deserialize, Serialize};
use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::serde_json::json;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::types::{ParseFromJSON, ToJSON};
use tardis::web::web_client::TardisHttpResponse;
use tardis::web::web_resp::{TardisPage, TardisResp};
use tardis::{TardisFuns, TardisFunsInst};

use crate::invoke_config::InvokeConfigApi;
use crate::invoke_constants::TARDIS_CONTEXT;
use crate::invoke_enumeration::InvokeModuleKind;

/// Add request for SPI backend service (SDK side)
///
/// SPI后端服务添加请求（SDK侧）
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SpiBsAddReq {
    pub name: String,
    pub kind_id: String,
    pub conn_uri: String,
    pub ak: String,
    pub sk: String,
    pub ext: String,
    pub private: bool,
    pub disabled: Option<bool>,
}

/// Summary response for SPI backend service (SDK side)
///
/// SPI后端服务概要响应（SDK侧）
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct SpiBsSummaryResp {
    pub id: String,
    pub name: String,
}

/// Detail response for SPI backend service (SDK side)
///
/// SPI后端服务详细响应（SDK侧）
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct SpiBsDetailResp {
    pub id: String,
    pub name: String,
    pub rel_app_tenant_ids: Vec<String>,
}

pub struct BaseSpiClient;

impl BaseSpiClient {
    pub async fn module_url(module: InvokeModuleKind, funs: &TardisFunsInst) -> TardisResult<String> {
        if let Some(uri) = funs.invoke_conf_module_url().get(&module.to_string()) {
            return Ok(uri.as_str().to_string());
        }
        Err(funs.err().conflict("spi-module", "spi_module", "spi module uri Not configured yet.", "400-spi-module-not-exist"))
    }

    pub async fn headers(headers: Option<Vec<(String, String)>>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<(String, String)>> {
        let spi_ctx = TardisContext {
            ak: funs.invoke_conf_spi_app_id(),
            ..ctx.clone()
        };
        let base_ctx = (TARDIS_CONTEXT.to_string(), TardisFuns::crypto.base64.encode(TardisFuns::json.obj_to_string(&spi_ctx)?));
        if let Some(mut headers) = headers {
            headers.push(base_ctx);
            return Ok(headers);
        }
        let headers = vec![base_ctx];
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

    /// Add a backend service instance if one with the same name does not already exist.
    /// Returns the existing or newly created backend service id.
    ///
    /// 如果同名后端服务不存在则创建，已存在则直接返回已有实例的id
    pub async fn add_bs_if_not_exist(module_url: &str, add_req: &SpiBsAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        let url = format!("{module_url}/ci/manage/bs?name={}&page_number=1&page_size=10", add_req.name);
        let resp = funs.web_client().get::<TardisResp<TardisPage<SpiBsSummaryResp>>>(&url, headers.clone()).await?;
        if let Some(page) = BaseSpiClient::package_resp(resp)? {
            if let Some(existing) = page.records.into_iter().find(|bs| bs.name == add_req.name) {
                return Ok(existing.id);
            }
        }
        let resp = funs.web_client().post::<_, TardisResp<String>>(&format!("{module_url}/ci/manage/bs"), add_req, headers).await?;
        BaseSpiClient::package_resp(resp)?.ok_or_else(|| TardisError::bad_request("Failed to create backend service", ""))
    }

    /// Bind a backend service to an app/tenant if not already bound.
    ///
    /// 如果后端服务未绑定到指定应用/租户则进行绑定，已绑定则跳过
    pub async fn add_bs_rel_if_not_exist(module_url: &str, bs_id: &str, app_tenant_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        let url = format!("{module_url}/ci/manage/bs/{bs_id}");
        let resp = funs.web_client().get::<TardisResp<SpiBsDetailResp>>(&url, headers.clone()).await?;
        if let Some(detail) = BaseSpiClient::package_resp(resp)? {
            if detail.rel_app_tenant_ids.contains(&app_tenant_id.to_string()) {
                return Ok(());
            }
        }
        let url = format!("{module_url}/ci/manage/bs/{bs_id}/rel/{app_tenant_id}");
        funs.web_client().put_obj_to_str(&url, &json!({}), headers).await?;
        Ok(())
    }
}
