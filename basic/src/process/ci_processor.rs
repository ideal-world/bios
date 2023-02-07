use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct AppKeyConfig {
    pub ak: String,
    pub sk: String,
}

pub fn signature()
