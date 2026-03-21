use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use serde::{Deserialize, Serialize};

use crate::iam_enumeration::IamThirdPartyAppStatusKind;
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm;
use tardis::web::poem_openapi;

/// 第三方应用新增请求
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamThirdPartyAppAddReq {
    #[oai(skip = true)]
    pub id: Option<TrimString>,
    #[oai(validator(min_length = "1", max_length = "255"))]
    pub name: TrimString,
    #[oai(validator(max_length = "255"))]
    pub external_id: Option<String>,
    pub scope_level: Option<RbumScopeLevelKind>,
    #[oai(validator(max_length = "1000"))]
    pub description: Option<String>,
    #[oai(validator(max_length = "1000"))]
    pub icon: Option<String>,
    #[oai(validator(min_length = "1", max_length = "2000"))]
    pub link_url: TrimString,
    pub status: Option<IamThirdPartyAppStatusKind>,
    pub sort: Option<i64>,
}

/// 第三方应用修改请求
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamThirdPartyAppModifyReq {
    #[oai(validator(min_length = "1", max_length = "255"))]
    pub name: Option<TrimString>,
    #[oai(validator(max_length = "255"))]
    pub external_id: Option<String>,
    pub scope_level: Option<RbumScopeLevelKind>,
    #[oai(validator(max_length = "1000"))]
    pub description: Option<String>,
    #[oai(validator(max_length = "1000"))]
    pub icon: Option<String>,
    #[oai(validator(max_length = "2000"))]
    pub link_url: Option<String>,
    pub status: Option<IamThirdPartyAppStatusKind>,
    pub sort: Option<i64>,
}

/// 第三方应用概要响应
#[derive(poem_openapi::Object, sea_orm::FromQueryResult, Serialize, Deserialize, Debug)]
pub struct IamThirdPartyAppSummaryResp {
    pub id: String,
    pub name: String,
    pub external_id: Option<String>,
    pub scope_level: RbumScopeLevelKind,
    pub description: Option<String>,
    pub icon: String,
    pub link_url: String,
    pub status: IamThirdPartyAppStatusKind,
    pub sort: i64,
    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

/// 批量修改第三方应用展示状态请求项
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamThirdPartyAppDisplayModifyItem {
    /// 第三方应用ID
    #[oai(validator(min_length = "1", max_length = "255"))]
    pub app_id: String,
    /// 是否展示：true 为展示，false 为隐藏
    pub visible: bool,
}

/// 批量修改当前账号关联的第三方应用展示状态请求
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamThirdPartyAppBatchModifyDisplayReq {
    /// 待修改的展示状态列表
    pub items: Vec<IamThirdPartyAppDisplayModifyItem>,
}

/// 第三方应用详情响应
#[derive(poem_openapi::Object, sea_orm::FromQueryResult, Serialize, Deserialize, Debug, Clone)]
pub struct IamThirdPartyAppDetailResp {
    pub id: String,
    pub name: String,
    pub external_id: Option<String>,
    pub scope_level: RbumScopeLevelKind,
    pub description: Option<String>,
    pub icon: String,
    pub link_url: String,
    pub status: IamThirdPartyAppStatusKind,
    pub sort: i64,
    pub own_paths: String,
    pub owner: String,
    pub owner_name: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}
