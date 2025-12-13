use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct SmsConfig {
    pub base_url: String,
    // 实际服务的地址（当 base_url为转发地址时，签名需要用到实际的服务地址）
    pub real_url: Option<String>,
    pub app_key: String,
    pub app_secret: String,
    pub status_call_back: Option<String>,

    pub sms_pwd_template_id: String,
    pub sms_general_from: String,
    pub sms_general_signature: Option<String>,
}
