use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemFilterFetcher};
use serde::{Deserialize, Serialize};
use tardis::{
    basic::field::TrimString,
    chrono::{self, Utc},
    db::sea_orm::{self},
    web::poem_openapi,
};

use crate::plugin_enumeration::PluginApiMethodKind;

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct PluginApiAddOrModifyReq {
    // #[oai(validator(pattern = r"^[a-z0-9]+$"))]
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub code: TrimString,
    pub name: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub kind_id: TrimString,
    pub callback: String,
    pub content_type: String,
    pub timeout: i32,
    pub ext: String,
    pub http_method: PluginApiMethodKind,
    pub kind: String,
    pub path_and_query: String,
    pub save_message: bool,
}

#[derive(poem_openapi::Object, sea_orm::FromQueryResult, Serialize, Deserialize, Debug)]
pub struct PluginApiSummaryResp {
    pub code: String,
    pub name: String,
    pub callback: String,
    pub content_type: String,
    pub timeout: i32,
    pub ext: String,
    pub http_method: PluginApiMethodKind,
    pub kind: String,
    pub path_and_query: String,
    pub save_message: bool,
}

#[derive(poem_openapi::Object, sea_orm::FromQueryResult, Serialize, Deserialize, Debug)]
pub struct PluginApiDetailResp {
    pub id: String,
    pub code: String,
    pub name: String,
    pub callback: String,
    pub content_type: String,
    pub timeout: i32,
    pub ext: String,
    pub http_method: PluginApiMethodKind,
    pub kind: String,
    pub path_and_query: String,
    pub save_message: bool,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct PluginApiFilterReq {
    pub basic: RbumBasicFilterReq,
    pub code: Option<String>,
    pub path_and_query: Option<String>,
    pub create_start: Option<chrono::DateTime<Utc>>,
    pub create_end: Option<chrono::DateTime<Utc>>,
}

impl RbumItemFilterFetcher for PluginApiFilterReq {
    fn basic(&self) -> &RbumBasicFilterReq {
        &self.basic
    }

    fn rel(&self) -> &Option<bios_basic::rbum::dto::rbum_filer_dto::RbumItemRelFilterReq> {
        &None
    }

    fn rel2(&self) -> &Option<bios_basic::rbum::dto::rbum_filer_dto::RbumItemRelFilterReq> {
        &None
    }
}
