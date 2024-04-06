use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_basic::rbum::serv::rbum_kind_serv::RbumKindServ;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::serde_json::Value;
use tardis::web::web_client::TardisHttpResponse;
use tardis::{TardisFuns, TardisFunsInst};

use super::plugin_api_serv::PluginApiServ;
use super::plugin_bs_serv::PluginBsServ;
use crate::dto::plugin_exec_dto::PluginExecReq;
pub struct PluginExecServ;

impl PluginExecServ {
    pub async fn exec(kind_code: &str, api_code: &str, exec_req: PluginExecReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<TardisHttpResponse<String>> {
        let kind_id = RbumKindServ::get_rbum_kind_id_by_code(kind_code, funs).await?;
        let Some(kind_id) = kind_id else {
            return Err(funs.err().not_found(&PluginApiServ::get_obj_name(), "exec", "exec kind is not fond", ""));
        };
        let spi_api = PluginApiServ::get_by_code(&kind_id, api_code, funs, ctx).await?;
        let result;
        if let Some(spi_api) = &spi_api {
            let spi_bs = PluginBsServ::get_bs_by_rel_up(kind_code, funs, ctx).await?;
            let url = format!(
                "{}{}",
                &spi_bs.conn_uri,
                Self::build_url(
                    &format!("{}{}", if spi_api.path_and_query.starts_with('/') { "" } else { "/" }, &spi_api.path_and_query),
                    exec_req.body.clone(),
                    funs,
                )?
            );
            let mut headers: Vec<(String, String)> = exec_req.header.unwrap_or_default().iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
            if let Some(rel) = spi_bs.rel {
                rel.attrs.iter().for_each(|attr| {
                    headers.push((attr.name.to_string(), attr.value.to_string()));
                });
            }
            headers.push(("Content-Type".to_string(), spi_api.content_type.to_string()));
            headers.push(("Callback-Url".to_string(), spi_api.callback.to_string()));
            let headers = headers;
            info!("url: {}", url);
            match spi_api.http_method {
                crate::plugin_enumeration::PluginApiMethodKind::GET => {
                    result = funs.web_client().get_to_str(url.as_str(), headers.clone()).await?;
                }
                crate::plugin_enumeration::PluginApiMethodKind::PUT => {
                    result = funs.web_client().put_str_to_str(url.as_str(), &TardisFuns::json.obj_to_string(&exec_req.body.clone())?, headers.clone()).await?;
                }
                crate::plugin_enumeration::PluginApiMethodKind::POST => {
                    result = funs.web_client().post_str_to_str(url.as_str(), &TardisFuns::json.obj_to_string(&exec_req.body.clone())?, headers.clone()).await?;
                }
                crate::plugin_enumeration::PluginApiMethodKind::DELETE => {
                    let delete_result = funs.web_client().delete_to_void(url.as_str(), headers.clone()).await?;
                    result = TardisHttpResponse {
                        code: delete_result.code,
                        headers: delete_result.headers,
                        body: Some("".to_string()),
                    }
                }
                crate::plugin_enumeration::PluginApiMethodKind::PATCH => {
                    result = funs.web_client().patch_str_to_str(url.as_str(), &TardisFuns::json.obj_to_string(&exec_req.body)?, headers.clone()).await?;
                }
            }
            if spi_api.save_message {
                // todo 日志记录 至 spi-log 暂存疑
            }
            return Ok(result);
        }
        return Err(funs.err().not_found(&PluginApiServ::get_obj_name(), "exec", "exec api is not fond", ""));
    }

    fn build_url(path: &str, body: Option<Value>, funs: &TardisFunsInst) -> TardisResult<String> {
        if !path.contains(':') {
            return Ok(path.to_string());
        }
        if let Some(body) = body {
            let mut is_ok = true;
            let new_path = path
                .split('/')
                .map(|r| {
                    if !r.starts_with(':') {
                        return r;
                    }
                    let new_r = r.replace(':', "");
                    if let Some(new_r) = body.get(&new_r) {
                        return new_r.as_str().unwrap_or("");
                    }
                    is_ok = false;
                    r
                })
                .collect::<Vec<&str>>()
                .join("/");
            if !new_path.contains(':') && is_ok {
                return Ok(new_path);
            }
        }
        Err(funs.err().not_found("build_url", "exec", "param is not found", ""))
    }
}
