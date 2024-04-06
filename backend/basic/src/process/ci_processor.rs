use serde::{Deserialize, Serialize};
use tardis::{basic::result::TardisResult, chrono::Utc, TardisFuns};

use crate::helper::url_helper::sort_query;

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

pub fn signature(app_key_config: &AppKeyConfig, method: &str, path: &str, query: &str, mut header: Vec<(String, String)>) -> TardisResult<Vec<(String, String)>> {
    let sorted_req_query = sort_query(query);
    let date = Utc::now().format("%a, %d %b %Y %T GMT").to_string();
    let signature =
        TardisFuns::crypto.base64.encode(TardisFuns::crypto.digest.hmac_sha256(format!("{method}\n{date}\n{path}\n{sorted_req_query}").to_lowercase(), &app_key_config.sk)?);
    header.push(("Authorization".to_string(), format!("{}:{signature}", app_key_config.ak)));
    header.push((app_key_config.head_key_date_flag.to_string(), date));
    Ok(header)
}
