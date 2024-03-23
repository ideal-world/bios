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
    /// 字段数据
    /// Map格式，key=事实表的字段名，value=字段值
    /// Field data
    ///
    /// Map format，key = field name of the fact table，value = field value
    pub data: Value,
    /// 动态数据
    /// Dynamic data
    pub ext: Option<Value>,
}
/// 事实记录加载请求对象
/// Load Fact Record Request Object
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsFactRecordsLoadReq {
    /// Primary key
    pub key: String,
    /// Own paths
    pub own_paths: String,
    /// Create time
    pub ct: DateTime<Utc>,
    /// 字段数据
    /// Map格式，key=事实表的字段名，value=字段值
    /// Field data
    ///
    /// Map format，key = field name of the fact table，value = field value
    pub data: Value,
    
    /// 动态数据
    /// Dynamic data
    pub ext: Option<Value>,
}

/// 添加维度记录请求对象
/// Add Dimension Record Request Object
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsDimRecordAddReq {
    /// 主键
    /// Primary key
    pub key: Value,
    /// 显示名称
    /// The name of the dimension
    pub show_name: String,
    /// 父主键
    /// The parent primary key, if present, indicates that this is a multilevel record
    pub parent_key: Option<Value>,
}

/// 删除维度记录请求对象
/// Delete Dimension Record Request Object
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsDimRecordDeleteReq {
    /// 主键
    /// Primary key
    pub key: Value,
}
