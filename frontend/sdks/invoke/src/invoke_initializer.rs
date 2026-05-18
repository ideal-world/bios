use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::TardisFunsInst;

use crate::invoke_config::InvokeConfig;
use crate::invoke_config::InvokeConfigManager;

pub fn init(code: &str, config: InvokeConfig) -> TardisResult<()> {
    InvokeConfigManager::add(code, config)?;
    Ok(())
}

/// If any module has `bs_init` configured, call its `init()` to create and bind the backend service.
///
/// 如果某个模块配置了 `bs_init`，则自动调用对应客户端的 `init()` 创建并绑定后端服务实例。
pub async fn init_bs(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    #[cfg(feature = "spi_kv")]
    {
        use crate::invoke_config::InvokeConfigApi;
        use crate::invoke_enumeration::InvokeModuleKind;
        if funs.invoke_conf_module_bs_init(InvokeModuleKind::Kv).is_some() {
            crate::clients::spi_kv_client::SpiKvClient::init(funs, ctx).await?;
        }
    }
    #[cfg(feature = "spi_log")]
    {
        use crate::invoke_config::InvokeConfigApi;
        use crate::invoke_enumeration::InvokeModuleKind;
        if funs.invoke_conf_module_bs_init(InvokeModuleKind::Log).is_some() {
            crate::clients::spi_log_client::SpiLogClient::init(funs, ctx).await?;
        }
    }
    #[cfg(feature = "spi_search")]
    {
        use crate::invoke_config::InvokeConfigApi;
        use crate::invoke_enumeration::InvokeModuleKind;
        if funs.invoke_conf_module_bs_init(InvokeModuleKind::Search).is_some() {
            crate::clients::spi_search_client::SpiSearchClient::init(funs, ctx).await?;
        }
    }
    #[cfg(feature = "spi_stats")]
    {
        use crate::invoke_config::InvokeConfigApi;
        use crate::invoke_enumeration::InvokeModuleKind;
        if funs.invoke_conf_module_bs_init(InvokeModuleKind::Stats).is_some() {
            crate::clients::spi_stats_client::SpiStatsClient::init(funs, ctx).await?;
        }
    }
    #[cfg(feature = "spi_object")]
    {
        use crate::invoke_config::InvokeConfigApi;
        use crate::invoke_enumeration::InvokeModuleKind;
        if funs.invoke_conf_module_bs_init(InvokeModuleKind::Object).is_some() {
            crate::clients::spi_object_client::SpiObjectClient::init(funs, ctx).await?;
        }
    }
    Ok(())
}
