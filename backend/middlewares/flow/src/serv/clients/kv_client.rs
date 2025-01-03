use bios_basic::rbum::helper::rbum_scope_helper;
use bios_sdk_invoke::clients::spi_kv_client::SpiKvClient;
use itertools::Itertools;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    TardisFunsInst,
};

pub struct FlowKvClient;

impl FlowKvClient {
    pub async fn get_account_name(account_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let account_name = SpiKvClient::get_item(format!("__k_n__:iam_account:{}", account_id), None, funs, ctx)
            .await?
            .map(|resp| resp.value.as_str().unwrap_or("").to_string())
            .unwrap_or_default();
        Ok(account_name)
    }

    pub async fn get_role_id(original_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let mut role_id = "".to_string();
        if let Some(role_id_prefix) = original_id.split(':').collect_vec().first() {
            role_id = SpiKvClient::match_items_by_key_prefix(format!("__k_n__:iam_role:{}", role_id_prefix), None, 1, 999, None, funs, ctx).await?
            .map(|resp| {
                resp.records.into_iter().filter(|record| ctx.own_paths.contains(&record.own_paths)).collect_vec()
            })
            .map(|records| {
                if let Some(item) = records.iter().find(|r| r.own_paths == ctx.own_paths) {
                    return item.key.clone();
                }
                if let Some(item) = records.iter().find(|r| r.own_paths == rbum_scope_helper::get_path_item(1, &ctx.own_paths).unwrap_or_default()) {
                    return item.key.clone();
                }
                if let Some(item) = records.iter().find(|r| r.own_paths.is_empty()) {
                    return item.key.clone();
                }
                "".to_string()
            })
            .unwrap_or_default();
        }

        Ok(role_id)
    }
}
