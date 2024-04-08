use http::Extensions;
use spacegate_shell::{kernel::extension::ExtensionPack as _, BoxError};
use tardis::serde_json::{self, Value};

use self::audit_log_param::LogParamContent;

pub mod audit_log_param;
pub mod before_encrypt_body;
pub mod cert_info;
pub mod request_crypto_status;

pub enum ExtensionPackEnum {
    LogParamContent(),
    None,
}

impl From<String> for ExtensionPackEnum {
    fn from(value: String) -> Self {
        match value.as_str() {
            "log_content" => ExtensionPackEnum::LogParamContent(),
            _ => ExtensionPackEnum::None,
        }
    }
}
impl ExtensionPackEnum {
    pub fn _to_value(&self, ext: &Extensions) -> Result<Option<Value>, BoxError> {
        match self {
            ExtensionPackEnum::LogParamContent() => {
                if let Some(ext) = LogParamContent::get(ext) {
                    return Ok(Some(serde_json::to_value(ext)?));
                }
            }
            ExtensionPackEnum::None => (),
        }
        Ok(None)
    }
}
