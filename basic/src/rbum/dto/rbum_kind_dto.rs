use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
#[cfg(feature = "default")]
use tardis::db::sea_orm;
#[cfg(feature = "default")]
use tardis::web::poem_openapi;

use crate::rbum::rbum_enumeration::RbumScopeLevelKind;

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct RbumKindAddReq {
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub code: TrimString,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub name: TrimString,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "2000")))]
    pub note: Option<String>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "1000")))]
    pub icon: Option<String>,
    pub sort: Option<i64>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub module: Option<String>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub ext_table_name: Option<String>,

    pub scope_level: Option<RbumScopeLevelKind>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct RbumKindModifyReq {
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub name: Option<TrimString>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "2000")))]
    pub note: Option<String>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "1000")))]
    pub icon: Option<String>,
    pub sort: Option<i64>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub module: Option<String>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub ext_table_name: Option<String>,

    pub scope_level: Option<RbumScopeLevelKind>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object, sea_orm::FromQueryResult))]
pub struct RbumKindSummaryResp {
    pub id: String,
    pub code: String,
    pub name: String,
    pub icon: String,
    pub sort: i64,
    pub module: String,
    pub ext_table_name: String,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object, sea_orm::FromQueryResult))]
pub struct RbumKindDetailResp {
    pub id: String,
    pub code: String,
    pub name: String,
    pub note: String,
    pub icon: String,
    pub sort: i64,
    pub module: String,
    pub ext_table_name: String,

    pub own_paths: String,
    pub owner: String,
    pub owner_name: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
}
