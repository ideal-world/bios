use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi;

use super::conf_namespace_dto::NamespaceId;

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct ConfigDescriptor {
    /// 命名空间，默认为public与 ''相同
    pub namespace_id: Option<NamespaceId>,
    /// 配置分组名
    pub group: String,
    /// 配置名
    pub data_id: String,
    /// 标签
    pub tag: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct ConfigPublishRequest {
    /// 配置内容
    content: String,
    #[serde(flatten)]
    descriptor: ConfigDescriptor,
    /// 应用名
    app_name: Option<String>,
    /// 源用户
    src_user: Option<String>,
    /// 配置标签列表，可多个，逗号分隔
    config_tags: Option<String>,
    /// 配置描述
    desc: Option<String>,
    ///
    r#use: Option<String>,
    ///
    effect: Option<String>,
    #[serde(rename = "type")]
    /// 配置类型
    tp: Option<String>,
    /// -
    schema: Option<String>,
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
