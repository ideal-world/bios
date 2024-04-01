use self::audit_log_param::{AuditLogParam, LogParamContent};

pub mod audit_log_param;
pub mod before_encrypt_body;
pub mod cert_info;
pub mod request_crypto_status;

pub enum ExtensionPackEnum {
    AuditLogParam(AuditLogParam),
    LogParamContent(LogParamContent),
    None,
}

impl From<String> for ExtensionPackEnum {
    fn from(value: String) -> Self {
        match value.as_str() {
            "log_param" => ExtensionPackEnum::AuditLogParam(AuditLogParam::new()),
            "log_content" => ExtensionPackEnum::LogParamContent(LogParamContent::new()),
            _ => ExtensionPackEnum::None,
        }
    }
}
