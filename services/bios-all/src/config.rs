use tardis::serde::{Deserialize, Serialize};

use bios_iam::iam_config::IamConfig;

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct BiosConfig {
    pub iam: IamConfig,
}

impl Default for BiosConfig {
    fn default() -> Self {
        BiosConfig { iam: Default::default() }
    }
}
