use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tardis::chrono::{DateTime, Utc};

use crate::rbum::rbum_enumeration::RbumRelFromKind;

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object))]
pub struct RbumRelAddReq {
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub tag: String,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "1000")))]
    pub note: Option<String>,
    pub from_rbum_kind: RbumRelFromKind,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub from_rbum_id: String,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub to_rbum_item_id: String,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub to_own_paths: String,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "1000")))]
    pub ext: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object))]
pub struct RbumRelModifyReq {
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub tag: Option<String>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "1000")))]
    pub note: Option<String>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "1000")))]
    pub ext: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object))]
pub struct RbumRelCheckReq {
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub tag: String,
    pub from_rbum_kind: RbumRelFromKind,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub from_rbum_id: String,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub to_rbum_item_id: String,
    pub from_attrs: HashMap<String, String>,
    pub to_attrs: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object))]
pub struct RbumRelFindReq {
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub tag: Option<String>,
    pub from_rbum_kind: Option<RbumRelFromKind>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub from_rbum_id: Option<String>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub to_rbum_item_id: Option<String>,
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
