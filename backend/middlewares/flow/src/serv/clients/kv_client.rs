use bios_sdk_invoke::clients::spi_kv_client::SpiKvClient;
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
}
