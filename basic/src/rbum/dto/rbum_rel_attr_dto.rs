use serde::{Deserialize, Serialize};
use tardis::chrono::{DateTime, Utc};
use tardis::web::poem_openapi::Object;

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumRelAttrAddReq {
    pub is_from: bool,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub value: String,

    #[oai(validator(min_length = "2", max_length = "255"))]
    pub rel_rbum_rel_id: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub rel_rbum_kind_attr_id: String,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumRelAttrModifyReq {
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object, tardis::db::sea_orm::FromQueryResult))]
pub struct RbumRelAttrDetailResp {
    pub id: String,
    pub is_from: bool,
    pub value: String,
    pub name: String,
    pub rel_rbum_kind_attr_id: String,
    pub rel_rbum_kind_attr_name: String,
    pub rel_rbum_rel_id: String,

    pub scope_ids: String,
    pub updater_id: String,
    pub updater_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}
