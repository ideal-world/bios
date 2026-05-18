use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::web::web_resp::{TardisPage, TardisResp};
use tardis::TardisFunsInst;

use crate::dto::search_item_dto::{SearchItemAddReq, SearchItemModifyReq, SearchItemSearchReq, SearchItemSearchResp, SearchSaveItemReq};
use crate::invoke_config::InvokeConfigApi;
use crate::invoke_enumeration::InvokeModuleKind;

use super::base_spi_client::{BaseSpiClient, SpiBsAddReq};
use super::spi_kv_client::SpiKvClient;

pub struct SpiSearchClient;

impl SpiSearchClient {
    /// Initialize the Search backend service: create if not exists and bind to app/tenant.
    /// Reads all configuration from `InvokeModuleConfig.bs_init`.
    ///
    /// 初始化 Search 后端服务：不存在则创建，并绑定到应用/租户。配置均来自 invoke 配置，返回后端服务id。
    pub async fn init(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let init_cfg = funs
            .invoke_conf_module_bs_init(InvokeModuleKind::Search)
            .ok_or_else(|| TardisError::bad_request("search module bs_init config not set", ""))?;
        let module_url = BaseSpiClient::module_url(InvokeModuleKind::Search, funs).await?;
        let bs_add_req = SpiBsAddReq {
            name: init_cfg.bs_name.clone(),
            kind_id: init_cfg.bs_kind_id.clone(),
            conn_uri: init_cfg.bs_conn_uri.clone(),
            ak: init_cfg.bs_ak.clone(),
            sk: init_cfg.bs_sk.clone(),
            ext: init_cfg.bs_ext.clone(),
            private: init_cfg.bs_private,
            disabled: init_cfg.bs_disabled,
        };
        let bs_id = BaseSpiClient::add_bs_if_not_exist(&module_url, &bs_add_req, funs, ctx).await?;
        BaseSpiClient::add_bs_rel_if_not_exist(&module_url, &bs_id, &init_cfg.app_tenant_id, funs, ctx).await?;
        Ok(bs_id)
    }

    pub async fn add_item_and_name(add_req: &SearchItemAddReq, name: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let search_url: String = BaseSpiClient::module_url(InvokeModuleKind::Search, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        funs.web_client().put_obj_to_str(&format!("{search_url}/ci/item"), add_req, headers.clone()).await?;
        let name = name.unwrap_or_else(|| add_req.title.clone());
        SpiKvClient::add_or_modify_key_name(&format!("{}:{}", add_req.tag, add_req.key), &name, add_req.kv_disable, funs, ctx).await?;
        Ok(())
    }

    pub async fn modify_item_and_name(tag: &str, key: &str, modify_req: &SearchItemModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let search_url: String = BaseSpiClient::module_url(InvokeModuleKind::Search, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        funs.web_client().put_obj_to_str(&format!("{search_url}/ci/item/{tag}/{key}"), modify_req, headers.clone()).await?;
        if modify_req.title.is_some() || modify_req.name.is_some() {
            let name = modify_req.name.clone().unwrap_or(modify_req.title.clone().unwrap_or("".to_string()));
            SpiKvClient::add_or_modify_key_name(&format!("{tag}:{key}"), &name, modify_req.kv_disable, funs, ctx).await?;
        }
        Ok(())
    }

    pub async fn delete_item_and_name(tag: &str, key: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let search_url = BaseSpiClient::module_url(InvokeModuleKind::Search, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        funs.web_client().delete_to_void(&format!("{search_url}/ci/item/{tag}/{key}"), headers.clone()).await?;
        SpiKvClient::delete_item(&format!("__k_n__:{tag}:{key}"), funs, ctx).await?;
        Ok(())
    }

    pub async fn search(search_req: &SearchItemSearchReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<TardisPage<SearchItemSearchResp>>> {
        let search_url = BaseSpiClient::module_url(InvokeModuleKind::Search, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        let resp =
            funs.web_client().put::<SearchItemSearchReq, TardisResp<TardisPage<SearchItemSearchResp>>>(format!("{search_url}/ci/item/search"), search_req, headers.clone()).await?;
        BaseSpiClient::package_resp(resp)
    }

    pub async fn save(tag: &str, save_req: &SearchSaveItemReq, name: Option<String>, kv_disable: Option<bool>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let search_url = BaseSpiClient::module_url(InvokeModuleKind::Search, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        funs.web_client().put_obj_to_str(&format!("{search_url}/ci/item/{tag}/save"), save_req, headers.clone()).await?;
        if save_req.title.is_some() || name.is_some() {
            let name = name.clone().unwrap_or(save_req.title.clone().unwrap_or("".to_string()));
            SpiKvClient::add_or_modify_key_name(&format!("{tag}:{}", save_req.key), &name, kv_disable, funs, ctx).await?;
        }
        Ok(())
    }

    /// Batch save
    pub async fn batch_save(tag: &str, batch_req: &Vec<SearchSaveItemReq>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let search_url = BaseSpiClient::module_url(InvokeModuleKind::Search, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        funs.web_client().put_obj_to_str(&format!("{search_url}/ci/item/{tag}/batch/save"), batch_req, headers.clone()).await?;
        Ok(())
    }

    /// Batch Delete
    pub async fn batch_delete(tag: &str, batch_req: &Vec<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let search_url = BaseSpiClient::module_url(InvokeModuleKind::Search, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        funs.web_client().put_obj_to_str(&format!("{search_url}/ci/item/{tag}/batch/delete"), batch_req, headers.clone()).await?;
        Ok(())
    }
}
