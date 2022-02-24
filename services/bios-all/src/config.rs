use tardis::serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct BiosConfig {}

impl Default for BiosConfig {
    fn default() -> Self {
        BiosConfig {}
    }
}
