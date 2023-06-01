use std::hash::Hash;

use super::conf_namespace_dto::NamespaceId;
use serde::{Deserialize, Serialize};
use tardis::{db::sea_orm::prelude::*, web::poem_openapi};
pub enum SearchMode {
    /// 模糊查询
    Fuzzy,
    /// 精确查询
    Exact,
}

impl Default for SearchMode {
    fn default() -> Self {
        Self::Fuzzy
    }
}

impl From<&str> for SearchMode {
    fn from(s: &str) -> Self {
        match s {
            // nacos use blur
            "fuzzy" | "blur" => Self::Fuzzy,
            // nacos use accurate
            "exact" | "accurate" => Self::Exact,
            _ => Self::default(),
        }
    }
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct ConfigDescriptor {
    /// 命名空间，默认为public与 ''相同
    #[serde(alias = "tenant")]
    #[serde(default)]
    #[oai(default, validator(min_length = 1, max_length = 256))]
    pub namespace_id: NamespaceId,
    /// 配置分组名
    #[oai(validator(min_length = 1, max_length = 256))]
    pub group: String,
    /// 配置名
    #[oai(validator(min_length = 1, max_length = 256))]
    pub data_id: String,
    /// 标签
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(rename = "type")]
    /// 配置类型
    pub tp: Option<String>,
}

impl PartialEq for ConfigDescriptor {
    fn eq(&self, other: &Self) -> bool {
        self.namespace_id == other.namespace_id && self.group == other.group && self.data_id == other.data_id
    }
}

impl Eq for ConfigDescriptor {}
impl Hash for ConfigDescriptor {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.namespace_id.hash(state);
        self.group.hash(state);
        self.data_id.hash(state);
    }
}

impl Default for ConfigDescriptor {
    fn default() -> Self {
        Self {
            namespace_id: "public".into(),
            group: Default::default(),
            data_id: Default::default(),
            tags: Default::default(),
            tp: Default::default(),
        }
    }
}

impl ConfigDescriptor {
    pub fn fix_namespace_id(&mut self) {
        if self.namespace_id.is_empty() {
            self.namespace_id = "public".into();
        }
    }
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct ConfigPublishRequest {
    /// 配置内容
    pub content: String,
    #[serde(flatten)]
    #[oai(flatten)]
    pub descriptor: ConfigDescriptor,
    /// 应用名
    pub app_name: Option<String>,
    /// 源用户
    pub src_user: Option<String>,
    /// 配置标签列表，可多个
    pub config_tags: Vec<String>,
    /// 配置描述
    pub desc: Option<String>,
    ///
    pub r#use: Option<String>,
    ///
    pub effect: Option<String>,
    /// -
    pub schema: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct ConfigItem {
    /// 配置id
    pub id: String,
    ///
    pub last_id: i32,
    /// 配置名
    pub data_id: String,
    /// 配置分组
    pub group: String,
    /// 租户信息（命名空间）
    pub namespace: String,
    /// 应用名
    pub app_name: Option<String>,
    /// 配置内容的md5值
    pub md5: String,
    /// 配置内容
    pub content: String,
    /// 源ip
    pub src_ip: Option<String>,
    /// 源用户
    pub src_user: String,
    /// 操作类型
    pub op_type: String,
    /// 创建时间
    pub created_time: DateTimeUtc,
    /// 上次修改时间
    pub last_modified_time: DateTimeUtc,
    ///
    pub encrypted_data_key: Option<String>,
}

impl Default for ConfigItem {
    fn default() -> Self {
        Self {
            id: Default::default(),
            last_id: -1,
            data_id: Default::default(),
            group: Default::default(),
            namespace: Default::default(),
            app_name: Default::default(),
            md5: Default::default(),
            content: Default::default(),
            src_ip: None,
            src_user: Default::default(),
            op_type: Default::default(),
            created_time: Default::default(),
            last_modified_time: Default::default(),
            encrypted_data_key: None,
        }
    }
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct ConfigItemDigest {
    /// 配置名
    pub data_id: String,
    /// 配置分组
    pub group: String,
    /// 租户信息（命名空间）
    pub namespace: String,
    /// 应用名
    pub app_name: Option<String>,
    /// 类型
    pub r#type: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct ConfigHistoryListRequest {
    #[serde(flatten)]
    pub descriptor: ConfigDescriptor,
    pub page_no: u32,
    pub page_size: u32,
}

impl Default for ConfigHistoryListRequest {
    fn default() -> Self {
        Self {
            descriptor: Default::default(),
            page_no: 1,
            page_size: 100,
        }
    }
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct ConfigListRequest {
    /// 命名空间，默认为public与 ''相同
    #[serde(alias = "tenant")]
    #[serde(default)]
    #[oai(default, validator(min_length = 1, max_length = 256))]
    pub namespace_id: Option<NamespaceId>,
    /// 配置分组名
    #[oai(validator(min_length = 1, max_length = 256))]
    pub group:  Option<String>,
    /// 配置名
    #[oai(validator(min_length = 1, max_length = 256))]
    pub data_id:  Option<String>,
    /// 标签
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(rename = "type")]
    /// 配置类型
    pub tp: Option<String>,
    pub page_no: u32,
    pub page_size: u32,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct ConfigListResponse {
    pub total_count: u32,
    pub page_number: u32,
    pub pages_available: u32,
    pub page_items: Vec<ConfigItem>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default)]
pub struct HistoryConfigsRequest {
    namespace_id: NamespaceId,
}
