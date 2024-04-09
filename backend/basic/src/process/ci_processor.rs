//! CI (Interface Console) Processor
//!
//! The CI type interface is mostly used for calls between systems, and the interface is authenticated by Ak/Sk to ensure the security of the interface.
//! CI类型的接口多用于系统之间的调用，通过Ak/Sk进行签名认证，保证接口的安全性。
use serde::{Deserialize, Serialize};
use tardis::{basic::result::TardisResult, chrono::Utc, TardisFuns};

use crate::helper::request_helper::sort_query;

/// Application key configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct AppKeyConfig {
    pub head_key_date_flag: String,
    pub ak: String,
    pub sk: String,
}

impl Default for AppKeyConfig {
    fn default() -> Self {
        AppKeyConfig {
            head_key_date_flag: "Bios-Date".to_string(),
            ak: "".to_string(),
            sk: "".to_string(),
        }
    }
}

/// Generate signature
///
/// Generate a signature for the request and return the request header with the signature
pub fn signature(app_key_config: &AppKeyConfig, method: &str, path: &str, query: &str, mut header: Vec<(String, String)>) -> TardisResult<Vec<(String, String)>> {
    let sorted_req_query = sort_query(query);
    let date = Utc::now().format("%a, %d %b %Y %T GMT").to_string();
    let signature =
        TardisFuns::crypto.base64.encode(TardisFuns::crypto.digest.hmac_sha256(format!("{method}\n{date}\n{path}\n{sorted_req_query}").to_lowercase(), &app_key_config.sk)?);
    header.push(("Authorization".to_string(), format!("{}:{signature}", app_key_config.ak)));
    header.push((app_key_config.head_key_date_flag.to_string(), date));
    Ok(header)
}
