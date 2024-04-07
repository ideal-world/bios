use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Mutex;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::TardisFunsInst;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct RbumConfig {
    pub set_cate_sys_code_node_len: usize,
    pub mq_topic_entity_deleted: String,
    pub mq_topic_event: String,
    pub mq_header_name_operator: String,
    pub task_mq_topic_event: String,
    // own_paths:ak -> vcode
    pub cache_key_cert_vcode_info_: String,
    pub cache_key_cert_vcode_expire_sec: usize,
    pub cache_key_cert_code_: String,
    pub cache_key_cert_code_expire_sec: usize,
    // set_code -> set_id
    pub cache_key_set_code_: String,
    pub cache_key_set_code_expire_sec: usize,
    // rbum_item_id -> nil expired
    pub cache_key_cert_locked_: String,
    // rbum_item_id -> error times by cycle
    pub cache_key_cert_err_times_: String,
    // table name (supports prefix matching) -> <c><u><d>
    pub event_domains: HashMap<String, String>,
    pub head_key_bios_ctx: String,
}

impl Default for RbumConfig {
    fn default() -> Self {
        RbumConfig {
            set_cate_sys_code_node_len: 4,
            mq_topic_entity_deleted: "rbum::entity_deleted".to_string(),
            mq_topic_event: "rbum::event".to_string(),
            mq_header_name_operator: "OP".to_string(),
            task_mq_topic_event: "rbum::task::event".to_string(),
            cache_key_cert_vcode_info_: "rbum:cache:cert:vcode:".to_string(),
            cache_key_cert_vcode_expire_sec: 300,
            cache_key_cert_code_: "rbum:cache:cert:code:".to_string(),
            cache_key_cert_code_expire_sec: 60 * 60 * 24,
            cache_key_set_code_: "rbum:cache:set:code:".to_string(),
            cache_key_set_code_expire_sec: 60 * 60 * 24,
            cache_key_cert_locked_: "rbum:cert:locked:".to_string(),
            cache_key_cert_err_times_: "rbum:cert:err_times:".to_string(),
            event_domains: HashMap::from([("rbum_".to_string(), "cud".to_string())]),
            head_key_bios_ctx: "Bios-Ctx".to_string(),
        }
    }
}

lazy_static! {
    static ref RBUM_CONFIG: Mutex<HashMap<String, RbumConfig>> = Mutex::new(HashMap::new());
}

pub struct RbumConfigManager;

impl RbumConfigManager {
    pub fn add(code: &str, config: RbumConfig) -> TardisResult<()> {
        let mut conf = RBUM_CONFIG.lock().map_err(|e| TardisError::internal_error(&format!("{e:?}"), ""))?;
        conf.insert(code.to_string(), config);
        Ok(())
    }

    pub fn match_event(code: &str, table_name: &str, operate: &str) -> bool {
        Self::get_config(code, |conf| conf.event_domains.iter().any(|(k, v)| table_name.contains(k) && v.contains(operate)))
    }

    pub fn get_config<F, T>(code: &str, fun: F) -> T
    where
        F: Fn(&RbumConfig) -> T,
    {
        let conf = RBUM_CONFIG.lock().unwrap_or_else(|e| panic!("rbum config lock error: {e:?}"));
        let conf = conf.get(code).unwrap_or_else(|| panic!("not found rbum config code {code}"));
        fun(conf)
    }
}

pub trait RbumConfigApi {
    fn rbum_conf_set_cate_sys_code_node_len(&self) -> usize;
    fn rbum_conf_mq_topic_entity_deleted(&self) -> String;
    fn rbum_conf_mq_topic_event(&self) -> String;
    fn rbum_conf_task_mq_topic_event(&self) -> String;
    fn rbum_conf_mq_header_name_operator(&self) -> String;
    fn rbum_conf_cache_key_cert_vcode_info_(&self) -> String;
    fn rbum_conf_cache_key_cert_vcode_expire_sec(&self) -> usize;
    fn rbum_conf_cache_key_cert_code_(&self) -> String;
    fn rbum_conf_cache_key_cert_code_expire_sec(&self) -> usize;
    fn rbum_conf_cache_key_set_code_(&self) -> String;
    fn rbum_conf_cache_key_set_code_expire_sec(&self) -> usize;
    fn rbum_conf_cache_key_cert_locked_(&self) -> String;
    fn rbum_conf_cache_key_cert_err_times_(&self) -> String;
    fn rbum_conf_match_event(&self, table_name: &str, operate: &str) -> bool;
    fn rbum_head_key_bios_ctx(&self) -> String;
}

impl RbumConfigApi for TardisFunsInst {
    fn rbum_conf_set_cate_sys_code_node_len(&self) -> usize {
        RbumConfigManager::get_config(self.module_code(), |conf| conf.set_cate_sys_code_node_len)
    }

    fn rbum_conf_mq_topic_entity_deleted(&self) -> String {
        RbumConfigManager::get_config(self.module_code(), |conf| conf.mq_topic_entity_deleted.to_string())
    }

    fn rbum_conf_mq_topic_event(&self) -> String {
        RbumConfigManager::get_config(self.module_code(), |conf| conf.mq_topic_event.to_string())
    }

    fn rbum_conf_task_mq_topic_event(&self) -> String {
        RbumConfigManager::get_config(self.module_code(), |conf| conf.task_mq_topic_event.to_string())
    }

    fn rbum_conf_mq_header_name_operator(&self) -> String {
        RbumConfigManager::get_config(self.module_code(), |conf| conf.mq_header_name_operator.to_string())
    }

    fn rbum_conf_cache_key_cert_vcode_info_(&self) -> String {
        RbumConfigManager::get_config(self.module_code(), |conf| conf.cache_key_cert_vcode_info_.to_string())
    }

    fn rbum_conf_cache_key_cert_vcode_expire_sec(&self) -> usize {
        RbumConfigManager::get_config(self.module_code(), |conf| conf.cache_key_cert_vcode_expire_sec)
    }

    fn rbum_conf_cache_key_cert_code_(&self) -> String {
        RbumConfigManager::get_config(self.module_code(), |conf| conf.cache_key_cert_code_.to_string())
    }

    fn rbum_conf_cache_key_cert_code_expire_sec(&self) -> usize {
        RbumConfigManager::get_config(self.module_code(), |conf| conf.cache_key_cert_code_expire_sec)
    }

    fn rbum_conf_cache_key_set_code_(&self) -> String {
        RbumConfigManager::get_config(self.module_code(), |conf| conf.cache_key_set_code_.to_string())
    }

    fn rbum_conf_cache_key_set_code_expire_sec(&self) -> usize {
        RbumConfigManager::get_config(self.module_code(), |conf| conf.cache_key_set_code_expire_sec)
    }

    fn rbum_conf_cache_key_cert_locked_(&self) -> String {
        RbumConfigManager::get_config(self.module_code(), |conf| conf.cache_key_cert_locked_.to_string())
    }

    fn rbum_conf_cache_key_cert_err_times_(&self) -> String {
        RbumConfigManager::get_config(self.module_code(), |conf| conf.cache_key_cert_err_times_.to_string())
    }

    fn rbum_conf_match_event(&self, table_name: &str, operate: &str) -> bool {
        RbumConfigManager::match_event(self.module_code(), table_name, operate)
    }
    fn rbum_head_key_bios_ctx(&self) -> String {
        RbumConfigManager::get_config(self.module_code(), |conf| conf.head_key_bios_ctx.to_string())
    }
}
