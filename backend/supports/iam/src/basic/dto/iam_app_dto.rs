use serde::{Deserialize, Serialize};
use strum::Display;
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm::{self, prelude::*};
use tardis::web::poem_openapi;

use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamAppAggAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub app_name: TrimString,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub app_icon: Option<String>,
    pub app_sort: Option<i64>,

    #[oai(validator(min_length = "2", max_length = "255"))]
    pub app_contact_phone: Option<String>,

    pub admin_ids: Option<Vec<String>>,

    pub disabled: Option<bool>,
    pub set_cate_id: Option<String>,

    pub kind: Option<IamAppKind>,
    pub sync_apps_group: Option<bool>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamAppAddReq {
    #[oai(skip = true)]
    pub id: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: TrimString,

    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,

    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub icon: Option<String>,
    pub sort: Option<i64>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub contact_phone: Option<String>,

    pub kind: Option<IamAppKind>,
    pub sync_apps_group: Option<bool>,
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, poem_openapi::Enum, EnumIter, sea_orm::DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(255))")]
pub enum IamAppKind {
    // 项目
    #[sea_orm(string_value = "Project")]
    Project,
    #[sea_orm(string_value = "Product")]
    // 产品
    Product,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamAppAggModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<TrimString>,

    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,

    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub icon: Option<String>,
    pub sort: Option<i64>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub contact_phone: Option<String>,

    pub admin_ids: Option<Vec<String>>,
    pub set_cate_id: Option<String>,

    pub sync_apps_group: Option<bool>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamAppModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<TrimString>,

    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,

    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub icon: Option<String>,
    pub sort: Option<i64>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub contact_phone: Option<String>,
}

#[derive(poem_openapi::Object, sea_orm::FromQueryResult, Serialize, Deserialize, Debug)]
pub struct IamAppSummaryResp {
    pub id: String,
    pub name: String,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
    pub disabled: bool,

    pub icon: String,
    pub sort: i64,
    pub contact_phone: String,

    pub kind: IamAppKind,
}

#[derive(poem_openapi::Object, sea_orm::FromQueryResult, Serialize, Deserialize, Debug, Clone)]
pub struct IamAppDetailResp {
    pub id: String,
    pub name: String,

    pub own_paths: String,
    pub owner: String,
    pub owner_name: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
    pub disabled: bool,

    pub icon: String,
    pub sort: i64,
    pub contact_phone: String,

    pub kind: IamAppKind,
}
