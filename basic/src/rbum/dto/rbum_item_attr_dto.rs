use serde::{Deserialize, Serialize};
use tardis::chrono::{DateTime, Utc};
use tardis::web::poem_openapi::Object;

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumItemAttrAddReq {
    #[oai(validator(min_length = "1", max_length = "2000"))]
    pub value: String,

    #[oai(validator(min_length = "2", max_length = "255"))]
    pub rel_rbum_item_id: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub rel_rbum_kind_attr_id: String,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumItemAttrModifyReq {
    #[oai(validator(min_length = "1", max_length = "2000"))]
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object, tardis::db::sea_orm::FromQueryResult))]
pub struct RbumItemAttrSummaryResp {
    pub id: String,
    pub value: String,
    pub rel_rbum_item_id: String,
    pub rel_rbum_item_name: String,
    pub rel_rbum_kind_attr_id: String,
    pub rel_rbum_kind_attr_name: String,

    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object, tardis::db::sea_orm::FromQueryResult))]
pub struct RbumItemAttrDetailResp {
    pub id: String,
    pub value: String,
    pub rel_rbum_item_id: String,
    pub rel_rbum_item_name: String,
    pub rel_rbum_kind_attr_id: String,
    pub rel_rbum_kind_attr_name: String,

    pub rel_app_code: String,
    pub rel_app_name: String,
    pub updater_code: String,
    pub updater_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}
