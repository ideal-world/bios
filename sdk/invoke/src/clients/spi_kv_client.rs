use std::marker::PhantomData;

use serde::Serialize;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::serde_json::json;
use tardis::TardisFunsInst;

use crate::invoke_config::InvokeConfigTrait;
use crate::invoke_enumeration::InvokeModuleKind;

use super::base_spi_client::BaseSpiClient;

#[derive(Debug, Default)]
pub struct SpiKvClient<C> {
    marker: PhantomData<C>
}

impl<C> SpiKvClient<C>
where C: InvokeConfigTrait + 'static
{
    pub async fn add_or_modify_item<T: ?Sized + Serialize>(key: &str, value: &T, info: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let kv_url: String = BaseSpiClient::module_url::<C>(InvokeModuleKind::Kv, funs).await?;
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
        let kv_url = BaseSpiClient::module_url::<C>(InvokeModuleKind::Kv, funs).await?;
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
}
