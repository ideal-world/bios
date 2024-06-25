use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use bios_sdk_invoke::clients::spi_kv_client::SpiKvClient;
use tardis::tokio;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    serde_json::Value,
    TardisFunsInst,
};

use crate::iam_constants;

pub struct IamKvClient;

impl IamKvClient {
    pub async fn async_add_or_modify_item(
        key: String,
        value: Value,
        info: Option<String>,
        scope_level: Option<RbumScopeLevelKind>,
        _funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let ctx_clone = ctx.clone();
        ctx.add_async_task(Box::new(|| {
            Box::pin(async move {
                let task_handle = tokio::spawn(async move {
                    let funs = iam_constants::get_tardis_inst();
                    let _ = Self::add_or_modify_item(&key, &value, info, scope_level, &funs, &ctx_clone).await;
                });
                task_handle.await.unwrap();
                Ok(())
            })
        }))
        .await
    }

    pub async fn async_delete_item(key: String, _funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let ctx_clone = ctx.clone();
        ctx.add_async_task(Box::new(|| {
            Box::pin(async move {
                let task_handle = tokio::spawn(async move {
                    let funs = iam_constants::get_tardis_inst();
                    let _ = Self::delete_item(&key, &funs, &ctx_clone).await;
                });
                task_handle.await.unwrap();
                Ok(())
            })
        }))
        .await
    }

    pub async fn add_or_modify_item(
        key: &str,
        value: &Value,
        info: Option<String>,
        scope_level: Option<RbumScopeLevelKind>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        SpiKvClient::add_or_modify_item(key, value, info, scope_level.map(|kind| kind.to_int()), funs, ctx).await
    }

    pub async fn delete_item(key: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        SpiKvClient::delete_item(key, funs, ctx).await
    }
}
