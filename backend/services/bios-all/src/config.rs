use tardis::serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct BiosConfig {
    pub intranet: bool,
}

impl Default for BiosConfig {
    fn default() -> Self {
        BiosConfig { intranet: false }
    }
}
