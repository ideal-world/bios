use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tardis::chrono::{DateTime, Utc};
use tardis::web::poem_openapi::Object;

use crate::rbum::rbum_enumeration::RbumRelFromKind;

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumRelAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tag: String,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub note: Option<String>,
    pub from_rbum_kind: RbumRelFromKind,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub from_rbum_id: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub to_rbum_item_id: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub to_own_paths: String,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub ext: Option<String>,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumRelModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tag: Option<String>,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub note: Option<String>,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub ext: Option<String>,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumRelCheckReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tag: String,
    pub from_rbum_kind: RbumRelFromKind,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub from_rbum_id: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub to_rbum_item_id: String,
    pub from_attrs: HashMap<String, String>,
    pub to_attrs: HashMap<String, String>,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumRelFindReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tag: String,
    pub from_rbum_kind: RbumRelFromKind,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub from_rbum_id: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub to_rbum_item_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object, tardis::db::sea_orm::FromQueryResult))]
pub struct RbumRelDetailResp {
    pub id: String,
    pub tag: String,
    pub note: String,
    pub from_rbum_kind: RbumRelFromKind,
    pub from_rbum_id: String,
    pub from_rbum_item_name: String,
    pub from_rbum_set_name: String,
    pub from_rbum_set_cate_name: String,
    pub to_rbum_item_id: String,
    pub to_rbum_item_name: String,
    pub to_own_paths: String,
    pub ext: String,

    pub own_paths: String,
    pub owner: String,
    pub owner_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}
