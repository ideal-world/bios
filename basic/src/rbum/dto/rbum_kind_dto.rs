use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
use tardis::web::poem_openapi::Object;

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumKindAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub uri_scheme: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: TrimString,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub note: Option<String>,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub icon: Option<String>,
    pub sort: Option<i32>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub ext_table_name: Option<String>,

    pub scope_level: i32,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumKindModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub uri_scheme: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub note: Option<String>,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub icon: Option<String>,
    pub sort: Option<i32>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub ext_table_name: Option<String>,

    pub scope_level: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object, tardis::db::sea_orm::FromQueryResult))]
pub struct RbumKindSummaryResp {
    pub id: String,
    pub uri_scheme: String,
    pub name: String,
    pub icon: String,
    pub sort: i32,
    pub ext_table_name: String,

    pub scope_paths: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: i32,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object, tardis::db::sea_orm::FromQueryResult))]
pub struct RbumKindDetailResp {
    pub id: String,
    pub uri_scheme: String,
    pub name: String,
    pub note: String,
    pub icon: String,
    pub sort: i32,
    pub ext_table_name: String,

    pub scope_paths: String,
    pub updater_id: String,
    pub updater_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: i32,
}
