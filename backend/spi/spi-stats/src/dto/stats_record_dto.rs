use serde::{Deserialize, Serialize};
use tardis::{
    chrono::{DateTime, Utc},
    serde_json::Value,
    web::poem_openapi,
};

/// 事实记录加载请求对象
/// Load Fact Record Request Object
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsFactRecordLoadReq {
    /// Own paths
    pub own_paths: String,
    /// Create time
    pub ct: DateTime<Utc>,

    /// Idempotent id
    /// ps: The idempotent id is used to ensure that the same request is not processed repeatedly
    /// 幂等id
    /// ps: 幂等id用于确保同一个请求不会重复处理
    pub idempotent_id: Option<String>,
    
    /// Field data
    /// 字段数据
    ///
    /// Map format，key = field name of the fact table，value = field value
    /// Map格式，key=事实表的字段名，value=字段值
    pub data: Value,
    /// Dynamic data
    ///
    /// 动态数据
    pub ext: Option<Value>,
}
/// Load Fact Record Request Object
///
/// 事实记录加载请求对象
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsFactRecordsLoadReq {
    /// Primary key
    pub key: String,
    /// Own paths
    pub own_paths: String,
    /// Create time
    pub ct: DateTime<Utc>,
    /// Idempotent id
    /// ps: The idempotent id is used to ensure that the same request is not processed repeatedly
    ///
    /// 幂等id
    /// ps: 幂等id用于确保同一个请求不会重复处理
    pub idempotent_id: Option<String>,
    /// Field data
    /// 字段数据
    ///
    /// Map format，key = field name of the fact table，value = field value
    /// Map格式，key=事实表的字段名，value=字段值
    pub data: Value,

    /// Dynamic data
    ///
    /// 动态数据
    pub ext: Option<Value>,
}

/// Add Dimension Record Request Object
///
/// 添加维度记录请求对象
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsDimRecordAddReq {
    /// Primary key
    ///
    /// 主键
    pub key: Value,
    /// The name of the dimension
    ///
    /// 显示名称
    pub show_name: String,
    /// The parent primary key, if present, indicates that this is a multilevel record
    ///
    /// 父主键，如果存在，表示这是一个多级记录
    pub parent_key: Option<Value>,
}

/// Delete Dimension Record Request Object
///
/// 删除维度记录请求对象
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsDimRecordDeleteReq {
    /// Primary key
    ///
    /// 主键
    pub key: Value,
}
