use serde::{Deserialize, Serialize};
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm::*;
use tardis::web::poem_openapi::Object;

use crate::rbum::enumeration::RbumScopeKind;

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumRelAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub from_rbum_kind_id: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub from_rbum_item_id: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub to_rbum_kind_id: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub to_rbum_item_id: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub to_other_app_id: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub to_other_tenant_id: String,
    #[oai(validator(max_length = "255"))]
    pub tags: String,
    #[oai(validator(max_length = "1000"))]
    pub ext: String,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumRelModifyReq {
    #[oai(validator(max_length = "255"))]
    pub tags: Option<String>,
    #[oai(validator(max_length = "1000"))]
    pub ext: Option<String>,
}

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
pub struct RbumRelDetailResp {
    pub id: String,
    pub from_rbum_kind_id: String,
    pub from_rbum_kind_name: String,
    pub from_rbum_item_id: String,
    pub from_rbum_item_name: String,
    pub to_rbum_kind_id: String,
    pub to_rbum_kind_name: String,
    pub to_rbum_item_id: String,
    pub to_rbum_item_name: String,
    pub to_other_app_id: String,
    pub to_other_app_name: String,
    pub to_other_tenant_id: String,
    pub to_other_tenant_name: String,
    pub tags: String,
    pub ext: String,

    pub rel_app_id: String,
    pub rel_app_name: String,
    pub rel_tenant_id: String,
    pub rel_tenant_name: String,
    pub updater_id: String,
    pub updater_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}
