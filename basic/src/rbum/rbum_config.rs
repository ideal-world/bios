use std::fmt::Debug;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct RbumConfig {
    pub set_cate_sys_code_node_len: usize,
}

impl Default for RbumConfig {
    fn default() -> Self {
        RbumConfig { set_cate_sys_code_node_len: 4 }
    }
}

