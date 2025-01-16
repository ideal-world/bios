use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Mutex;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::TardisFunsInst;

/// Rbum configuration
///
/// Rbum 配置
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct RbumConfig {
    /// The length of the system code of the set category(node)
    ///
    /// 集合分类（节点）系统代码的长度
    pub set_cate_sys_code_node_len: usize,
    /// The topic of the message queue when the entity is deleted
    ///
    /// 实体删除时的消息队列主题
    ///
    /// TODO
    #[deprecated]
    pub mq_topic_entity_deleted: String,
    /// The topic of the message queue when the event occurs
    ///
    /// 事件发生时的消息队列主题
    pub mq_topic_event: String,
    #[deprecated]
    pub mq_header_name_operator: String,
    #[deprecated]
    pub task_mq_topic_event: String,
    /// Cache key prefix for resource set code
    ///
    /// 资源集合代码的缓存键前缀
    ///
    /// Format: ``set_code -> set_id``
    pub cache_key_set_code_: String,
    /// Cache key expiration time for resource set code
    ///
    /// 资源集合代码的缓存键过期时间
    pub cache_key_set_code_expire_sec: usize,
    /// Cache key prefix for certificate verification code information
    ///
    /// 凭证验证码信息的缓存键前缀
    ///
    /// Format: ``own_paths:ak -> vcode``
    pub cache_key_cert_vcode_info_: String,
    /// Cache key prefix for locked certificate
    ///
    /// 锁定凭证的缓存键前缀
    ///
    /// Format: ``rbum_item_id -> nil``
    pub cache_key_cert_locked_: String,
    /// Cache key prefix for certificate error times
    ///
    /// 凭证错误次数的缓存键前缀
    ///
    /// Format: ``rbum_item_id -> error times by cycle``
    pub cache_key_cert_err_times_: String,
    /// Event domain configuration
    ///
    /// 事件域配置
    ///
    /// Format: ``table name (supports prefix matching) -> <c><u><d>``
    /// TODO
    pub event_domains: HashMap<String, String>,
    /// Header name of BIOS context request
    ///
    /// BIOS 上下文的请求头名称
    pub head_key_bios_ctx: String,
    pub sm4_key: String,
    pub sm4_iv: String,
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
            cache_key_set_code_: "rbum:cache:set:code:".to_string(),
            cache_key_set_code_expire_sec: 60 * 60 * 24,
            cache_key_cert_locked_: "rbum:cert:locked:".to_string(),
            cache_key_cert_err_times_: "rbum:cert:err_times:".to_string(),
            event_domains: HashMap::from([("rbum_".to_string(), "cud".to_string())]),
            head_key_bios_ctx: "Bios-Ctx".to_string(),
            sm4_key: "1234567890abcdef".to_string(),
            sm4_iv: "1234567890abcdef".to_string(),
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

// TODO simplify
pub trait RbumConfigApi {
    fn rbum_conf_set_cate_sys_code_node_len(&self) -> usize;
    fn rbum_conf_mq_topic_entity_deleted(&self) -> String;
    fn rbum_conf_mq_topic_event(&self) -> String;
    fn rbum_conf_task_mq_topic_event(&self) -> String;
    fn rbum_conf_mq_header_name_operator(&self) -> String;
    fn rbum_conf_cache_key_cert_vcode_info_(&self) -> String;
    fn rbum_conf_cache_key_set_code_(&self) -> String;
    fn rbum_conf_cache_key_set_code_expire_sec(&self) -> usize;
    fn rbum_conf_cache_key_cert_locked_(&self) -> String;
    fn rbum_conf_cache_key_cert_err_times_(&self) -> String;
    fn rbum_conf_match_event(&self, table_name: &str, operate: &str) -> bool;
    fn rbum_head_key_bios_ctx(&self) -> String;
    fn rbum_conf_sm4_key(&self) -> String;
    fn rbum_conf_sm4_iv(&self) -> String;
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

    fn rbum_conf_sm4_key(&self) -> String {
        RbumConfigManager::get_config(self.module_code(), |conf| conf.sm4_key.to_string())
    }

    fn rbum_conf_sm4_iv(&self) -> String {
        RbumConfigManager::get_config(self.module_code(), |conf| conf.sm4_iv.to_string())
    }
}
