use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Mutex;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct RbumConfig {
    pub set_cate_sys_code_node_len: usize,
    pub mq_topic_entity_deleted: String,
    pub mq_topic_event: String,
    pub mq_header_name_operator: String,
    // own_paths:ak -> vcode
    pub cache_key_cert_vcode_info_: String,
    pub cache_key_cert_vcode_expire_sec: usize,
    pub cache_key_cert_code_: String,
    pub cache_key_cert_code_expire_sec: usize,
    pub cache_key_set_code_: String,
    pub cache_key_set_code_expire_sec: usize,
    // table name (support prefix matching) -> <c><u><d>
    pub event_domains: HashMap<String, String>,
}

impl Default for RbumConfig {
    fn default() -> Self {
        RbumConfig {
            set_cate_sys_code_node_len: 4,
            mq_topic_entity_deleted: "rbum::entity_deleted".to_string(),
            mq_topic_event: "rbum::event".to_string(),
            mq_header_name_operator: "OP".to_string(),
            cache_key_cert_vcode_info_: "rbum:cache:cert:vcode:".to_string(),
            cache_key_cert_vcode_expire_sec: 2,
            cache_key_cert_code_: "rbum:cache:cert:code:".to_string(),
            cache_key_cert_code_expire_sec: 60 * 60 * 24,
            cache_key_set_code_: "rbum:cache:set:code:".to_string(),
            cache_key_set_code_expire_sec: 60 * 60 * 24,
            event_domains: HashMap::from([("rbum_".to_string(), "cud".to_string())]),
        }
    }
}

lazy_static! {
    static ref RBUM_CONFIG: Mutex<HashMap<String, RbumConfig>> = Mutex::new(HashMap::new());
}

pub struct RbumConfigManager;

impl RbumConfigManager {
    pub fn add(code: &str, config: RbumConfig) -> TardisResult<()> {
        let mut conf = RBUM_CONFIG.lock().map_err(|e| TardisError::InternalError(format!("{:?}", e)))?;
        conf.insert(code.to_string(), config);
        Ok(())
    }

    pub fn get(code: &str) -> TardisResult<RbumConfig> {
        let conf = RBUM_CONFIG.lock().map_err(|e| TardisError::InternalError(format!("{:?}", e)))?;
        let conf = conf.get(code).ok_or_else(|| TardisError::NotFound(code.to_string()))?;
        // TODO
        Ok(conf.clone())
    }

    pub fn match_event(code: &str, table_name: &str, operate: &str) -> TardisResult<bool> {
        Self::get_config(code, |conf| conf.event_domains.iter().any(|(k, v)| table_name.contains(k) && v.contains(operate)))
    }

    fn get_config<F, T>(code: &str, fun: F) -> TardisResult<T>
    where
        F: Fn(&RbumConfig) -> T,
    {
        let conf = RBUM_CONFIG.lock().map_err(|e| TardisError::InternalError(format!("{:?}", e)))?;
        let conf = conf.get(code).ok_or_else(|| TardisError::NotFound(code.to_string()))?;
        Ok(fun(conf))
    }
}
