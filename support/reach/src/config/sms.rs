use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HwSmsConfig {
    pub base_url: String,
    pub app_key: String,
    pub app_secret: String,
    pub notice_sender: String,
    pub v_code_sender: String,
    pub status_call_back: Option<String>,

    pub sms_pwd_template_id: String,
    pub sms_general_from: String,
    pub sms_general_signature: Option<String>,
}
