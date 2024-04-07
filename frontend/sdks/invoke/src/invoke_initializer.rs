use tardis::basic::result::TardisResult;

use crate::invoke_config::InvokeConfig;
use crate::invoke_config::InvokeConfigManager;

pub fn init(code: &str, config: InvokeConfig) -> TardisResult<()> {
    InvokeConfigManager::add(code, config)?;
    Ok(())
}
