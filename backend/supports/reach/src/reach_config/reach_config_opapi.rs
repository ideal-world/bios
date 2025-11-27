use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct OpApiConfig {
    pub app_key: String,
    pub app_secret: String,
}




