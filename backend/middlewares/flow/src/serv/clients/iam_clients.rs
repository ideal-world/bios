use bios_sdk_invoke::clients::base_spi_client::BaseSpiClient;
use bios_sdk_invoke::clients::iam_client::{IamCiTempProductGrantReq, IamClient};
use bios_sdk_invoke::invoke_config::InvokeConfigApi;
use bios_sdk_invoke::invoke_enumeration::InvokeModuleKind;
use tardis::basic::{dto::TardisContext, result::TardisResult};
use tardis::TardisFunsInst;

pub struct FlowIamClient;

impl FlowIamClient {
    pub async fn grant_temp_product(account_id: &str, product_id: &str, tenant_id: &str, expire_at: i64, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let url = BaseSpiClient::module_url(InvokeModuleKind::Iam, funs).await?;
        let spi_ctx = funs.invoke_conf_inject_context(ctx);
        IamClient::new("", funs, &spi_ctx, Some(ctx.clone()), &url)
            .grant_temp_product(&IamCiTempProductGrantReq {
                account_id: account_id.to_string(),
                product_id: product_id.to_string(),
                tenant_id: tenant_id.to_string(),
                expire_at,
            })
            .await?;
        Ok(())
    }

    pub async fn revoke_temp_product(account_id: &str, product_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let url = BaseSpiClient::module_url(InvokeModuleKind::Iam, funs).await?;
        let spi_ctx = funs.invoke_conf_inject_context(ctx);
        IamClient::new("", funs, &spi_ctx, Some(ctx.clone()), &url).revoke_temp_product(account_id, product_id).await?;
        Ok(())
    }
}
