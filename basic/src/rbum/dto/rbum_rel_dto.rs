use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tardis::chrono::{DateTime, Utc};
use tardis::web::poem_openapi::Object;

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumRelAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tag: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub from_rbum_item_id: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub to_rbum_item_id: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub to_scope_ids: String,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub ext: Option<String>,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumRelModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tag: Option<String>,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub ext: Option<String>,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumRelCheckReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tag: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub from_rbum_item_id: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub to_rbum_item_id: String,
    pub from_attrs: HashMap<String, String>,
    pub to_attrs: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object, tardis::db::sea_orm::FromQueryResult))]
pub struct RbumRelDetailResp {
    pub id: String,
    pub tag: String,
    pub from_rbum_item_id: String,
    pub from_rbum_item_name: String,
    pub to_rbum_item_id: String,
    pub to_rbum_item_name: String,
    pub to_scope_ids: String,
    pub ext: String,

    pub scope_ids: String,
    pub updater_id: String,
    pub updater_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}
