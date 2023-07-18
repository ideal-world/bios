use serde::{Deserialize, Serialize};
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::chrono::{DateTime, Utc};
use tardis::serde_json::{json, Value};
use tardis::TardisFunsInst;
use tardis::web::web_resp::TardisResp;
use tardis::web::{poem_openapi,web_resp::TardisPage};

use crate::invoke_enumeration::InvokeModuleKind;

use super::base_spi_client::BaseSpiClient;

pub struct SpiKvClient;

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct KvItemSummaryResp {
    #[oai(validator(min_length = "2"))]
    pub key: String,
    pub value: Value,
    pub info: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

impl SpiKvClient {
    pub async fn add_or_modify_item<T: ?Sized + Serialize>(key: &str, value: &T, info: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let kv_url: String = BaseSpiClient::module_url(InvokeModuleKind::Kv, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        let json = json!({
            "key":key.to_string(),
            "value":value,
            "info":info
        });
        funs.web_client().put_obj_to_str(&format!("{kv_url}/ci/item"), &json, headers.clone()).await?;
        Ok(())
    }

    pub async fn add_or_modify_key_name(key: &str, name: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let kv_url = BaseSpiClient::module_url(InvokeModuleKind::Kv, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        funs.web_client()
            .put_obj_to_str(
                &format!("{kv_url}/ci/scene/key-name"),
                &json!({
                    "key":key.to_string(),
                    "name": name.to_string()
                }),
                headers.clone(),
            )
            .await?;
        Ok(())
    }

    pub async fn match_items_by_key_prefix(key_prefix: String,extract: Option<String>,page_number: u32,page_size: u16, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<TardisPage<KvItemSummaryResp>>> {
        let kv_url = BaseSpiClient::module_url(InvokeModuleKind::Kv, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;
        let resp = funs.web_client().get::<TardisResp<TardisPage<KvItemSummaryResp>>>(&format!("{kv_url}/ci/item?=key_prefix={key_prefix}&extract={:?}&page_number={page_number}&page_size={page_size}", extract), headers.clone()).await?;
        BaseSpiClient::package_resp(resp)
    }
}
