use super::conf_namespace_dto::NamespaceId;
use serde::{Deserialize, Serialize};
use tardis::db::sea_orm;
use tardis::web::poem_openapi::{ApiExtractor, ApiExtractorType};
use tardis::{db::sea_orm::prelude::*, web::poem_openapi};

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct ConfigDescriptor {
    /// 命名空间，默认为public与 ''相同
    #[serde(alias = "tenant")]
    #[serde(default)]
    #[oai(default)]
    pub namespace_id: NamespaceId,
    /// 配置分组名
    pub group: String,
    /// 配置名
    pub data_id: String,
    /// 标签
    pub tag: Option<String>,
    #[serde(rename = "type")]
    /// 配置类型
    pub tp: Option<String>,
}
impl Default for ConfigDescriptor {
    fn default() -> Self {
        Self {
            namespace_id: "public".into(),
            group: Default::default(),
            data_id: Default::default(),
            tag: Default::default(),
            tp: Default::default(),
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
    /// 配置标签列表，可多个，逗号分隔
    pub config_tags: Option<String>,
    /// 配置描述
    pub desc: Option<String>,
    ///
    pub r#use: Option<String>,
    ///
    pub effect: Option<String>,
    /// -
    pub schema: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct ConfigItem {
    // 	配置id
    id: String,
    //
    last_id: u32,
    // 	配置名
    data_id: String,
    // 	配置分组
    group: String,
    // 	租户信息（命名空间）
    tenant: String,
    // 	应用名
    app_name: String,
    // 	配置内容的md5值
    md5: String,
    // 	配置内容
    content: String,
    // 	源ip
    src_ip: String,
    // 	源用户
    src_user: String,
    // 	操作类型
    op_type: String,
    // 	创建时间
    created_time: String,
    // 	上次修改时间
    last_modified_time: String,
    //
    encrypted_data_key: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct ConfigHistoryList {
    #[serde(flatten)]
    descriptor: ConfigDescriptor,
    page_no: u32,
    page_size: u32,
}

impl Default for ConfigHistoryList {
    fn default() -> Self {
        Self {
            descriptor: Default::default(),
            page_no: 1,
            page_size: 100,
        }
    }
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default)]
pub struct HistoryConfigsRequest {
    namespace_id: NamespaceId,
}
