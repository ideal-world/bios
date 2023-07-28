use serde::{Serialize, Deserialize};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Config {
    pub base_url: String,
}

