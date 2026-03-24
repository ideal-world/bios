use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use serde::{Deserialize, Serialize};
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
    pub scope_level: Option<RbumScopeLevelKind>,
    #[oai(validator(max_length = "1000"))]
    pub description: Option<String>,
    #[oai(validator(max_length = "1000"))]
    pub icon: Option<String>,
    #[oai(validator(min_length = "1", max_length = "2000"))]
    pub link_url: TrimString,
    pub status: Option<i16>,
    pub sort: Option<i64>,
}

/// 第三方应用修改请求
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamThirdPartyAppModifyReq {
    #[oai(validator(min_length = "1", max_length = "255"))]
    pub name: Option<TrimString>,
    pub scope_level: Option<RbumScopeLevelKind>,
    #[oai(validator(max_length = "1000"))]
    pub description: Option<String>,
    #[oai(validator(max_length = "1000"))]
    pub icon: Option<String>,
    #[oai(validator(max_length = "2000"))]
    pub link_url: Option<String>,
    pub status: Option<i16>,
    pub sort: Option<i64>,
}

/// 第三方应用概要响应
#[derive(poem_openapi::Object, sea_orm::FromQueryResult, Serialize, Deserialize, Debug)]
pub struct IamThirdPartyAppSummaryResp {
    pub id: String,
    pub name: String,
    pub scope_level: RbumScopeLevelKind,
    pub description: Option<String>,
    pub icon: String,
    pub link_url: String,
    pub status: i16,
    pub sort: i64,
    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

/// 第三方应用详情响应
#[derive(poem_openapi::Object, sea_orm::FromQueryResult, Serialize, Deserialize, Debug, Clone)]
pub struct IamThirdPartyAppDetailResp {
    pub id: String,
    pub name: String,
    pub scope_level: RbumScopeLevelKind,
    pub description: Option<String>,
    pub icon: String,
    pub link_url: String,
    pub status: i16,
    pub sort: i64,
    pub own_paths: String,
    pub owner: String,
    pub owner_name: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}
