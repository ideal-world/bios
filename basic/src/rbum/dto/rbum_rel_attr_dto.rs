use serde::{Deserialize, Serialize};
use tardis::chrono::{DateTime, Utc};
use tardis::web::poem_openapi::Object;

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumRelAttrAddReq {
    pub is_from: bool,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub value: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: String,
    pub record_only: bool,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub rel_rbum_kind_attr_id: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub rel_rbum_rel_id: String,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumRelAttrModifyReq {
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub value: Option<String>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object, tardis::db::sea_orm::FromQueryResult))]
pub struct RbumRelAttrDetailResp {
    pub id: String,
    pub is_from: bool,
    pub value: String,
    pub name: String,
    pub record_only: bool,
    pub rel_rbum_kind_attr_id: String,
    pub rel_rbum_kind_attr_name: String,
    pub rel_rbum_rel_id: String,

    pub own_paths: String,
    pub owner: String,
    pub owner_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}
