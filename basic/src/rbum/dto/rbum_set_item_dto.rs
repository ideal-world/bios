use serde::{Deserialize, Serialize};
use tardis::chrono::{DateTime, Utc};
#[cfg(feature = "default")]
use tardis::db::sea_orm;
#[cfg(feature = "default")]
use tardis::web::poem_openapi;

use crate::rbum::rbum_enumeration::RbumScopeLevelKind;

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct RbumSetItemAddReq {
    pub sort: u32,

    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub rel_rbum_set_id: String,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub rel_rbum_set_cate_id: String,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub rel_rbum_item_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct RbumSetItemModifyReq {
    pub sort: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object, sea_orm::FromQueryResult))]
pub struct RbumSetItemSummaryResp {
    pub id: String,
    pub sort: u32,
    pub rel_rbum_set_id: String,
    pub rel_rbum_set_cate_id: String,
    pub rel_rbum_set_cate_sys_code: String,
    pub rel_rbum_set_cate_name: String,
    pub rel_rbum_item_id: String,
    pub rel_rbum_item_name: String,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object, sea_orm::FromQueryResult))]
pub struct RbumSetItemDetailResp {
    pub id: String,
    pub sort: u32,
    pub rel_rbum_set_id: String,
    pub rel_rbum_set_cate_id: String,
    pub rel_rbum_set_cate_sys_code: String,
    pub rel_rbum_set_cate_name: String,
    pub rel_rbum_item_id: String,
    pub rel_rbum_item_code: String,
    pub rel_rbum_item_name: String,
    pub rel_rbum_item_kind_id: String,
    pub rel_rbum_item_domain_id: String,
    pub rel_rbum_item_owner: String,
    pub rel_rbum_item_create_time: DateTime<Utc>,
    pub rel_rbum_item_update_time: DateTime<Utc>,
    pub rel_rbum_item_disabled: bool,
    pub rel_rbum_item_scope_level: RbumScopeLevelKind,

    pub own_paths: String,
    pub owner: String,
    pub owner_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object, sea_orm::FromQueryResult))]
pub struct RbumSetItemInfoResp {
    pub id: String,
    pub sort: u32,
    pub rel_rbum_item_id: String,
    pub rel_rbum_item_code: String,
    pub rel_rbum_item_name: String,
    pub rel_rbum_item_kind_id: String,
    pub rel_rbum_item_domain_id: String,
    pub rel_rbum_item_owner: String,
    pub rel_rbum_item_create_time: DateTime<Utc>,
    pub rel_rbum_item_update_time: DateTime<Utc>,
    pub rel_rbum_item_disabled: bool,
    pub rel_rbum_item_scope_level: RbumScopeLevelKind,

    pub own_paths: String,
    pub owner: String,
}
