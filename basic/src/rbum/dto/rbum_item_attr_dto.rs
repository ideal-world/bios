use serde::{Deserialize, Serialize};
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm::*;
use tardis::web::poem_openapi::Object;

use crate::rbum::enumeration::RbumScopeKind;

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumItemAttrAddReq {
    #[oai(validator(min_length = "1", max_length = "2000"))]
    pub value: String,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumItemAttrModifyReq {
    #[oai(validator(min_length = "1", max_length = "2000"))]
    pub value: Option<String>,
}

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
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

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
pub struct RbumItemAttrDetailResp {
    pub id: String,
    pub value: String,
    pub rel_rbum_item_id: String,
    pub rel_rbum_item_name: String,
    pub rel_rbum_kind_attr_id: String,
    pub rel_rbum_kind_attr_name: String,

    pub rel_app_id: String,
    pub rel_app_name: String,
    pub rel_tenant_id: String,
    pub rel_tenant_name: String,
    pub updater_id: String,
    pub updater_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}
