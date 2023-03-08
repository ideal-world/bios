use std::collections::HashMap;

use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_basic::spi::serv::spi_bs_serv::SpiBsServ;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::web::web_client::TardisHttpResponse;
use tardis::{TardisFuns, TardisFunsInst};

use super::plugin_api_serv::PluginApiServ;
use crate::dto::plugin_exec_dto::PluginExecReq;
pub struct PluginExecServ;

impl PluginExecServ {
    pub async fn exec(kind_code: &str, api_code: &str, exec_req: PluginExecReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<TardisHttpResponse<String>> {
        let spi_api = PluginApiServ::get_by_code(api_code, funs, ctx).await?;
        let spi_bs = SpiBsServ::get_bs_by_rel_up(Some(kind_code.to_owned()), funs, ctx).await?;
        let result;
        if let Some(spi_api) = &spi_api {
            let url = Self::build_url(&format!("{}/{}", &spi_bs.conn_uri, &spi_api.path_and_query), exec_req.body.clone(), funs)?;
            let headers = Some(exec_req.header.unwrap_or_default().iter().map(|(k, v)| (k.to_string(), v.to_string())).collect());
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
                    result = funs.web_client().delete(url.as_str(), headers.clone()).await?;
                }
                crate::plugin_enumeration::PluginApiMethodKind::PATCH => {
                    result = funs.web_client().patch_str_to_str(url.as_str(), &TardisFuns::json.obj_to_string(&exec_req.body)?, headers).await?;
                }
            }
            if spi_api.save_message {
                // todo 日志记录 至 spi-log 暂存疑
            }
            return Ok(result);
        }
        return Err(funs.err().not_found(&PluginApiServ::get_obj_name(), "exec", "exec api is not fond", ""));
    }

    fn build_url(path: &str, body: Option<HashMap<String, String>>, funs: &TardisFunsInst) -> TardisResult<String> {
        if !path.contains(':') {
            return Ok(path.to_string());
        }
        if let Some(body) = body {
            let mut is_ok = true;
            let new_path = path
                .split('/')
                .into_iter()
                .map(|r| {
                    if !r.starts_with(':') {
                        return r;
                    }
                    let new_r = r.replace(':', "");
                    if body.contains_key(&new_r) {
                        return body.get(&new_r).unwrap();
                    }
                    is_ok = false;
                    r
                })
                .collect::<Vec<&str>>()
                .join("/");
            if is_ok {
                return Ok(new_path);
            }
        }
        Err(funs.err().not_found("build_url", "exec", "param is not found", ""))
    }
}
