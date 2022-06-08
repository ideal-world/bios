use tardis::serde::{Deserialize, Serialize};

use bios_iam::iam_config::IamConfig;

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct BiosConfig {
    pub iam: IamConfig,
}
