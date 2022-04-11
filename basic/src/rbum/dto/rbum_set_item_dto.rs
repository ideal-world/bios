use serde::{Deserialize, Serialize};
use tardis::chrono::{DateTime, Utc};
use tardis::web::poem_openapi::Object;

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumSetItemAddReq {
    pub sort: u32,

    #[oai(validator(min_length = "2", max_length = "255"))]
    pub rel_rbum_set_id: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub rel_rbum_set_cate_id: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub rel_rbum_item_id: String,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumSetItemModifyReq {
    pub sort: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object, tardis::db::sea_orm::FromQueryResult))]
pub struct RbumSetItemDetailResp {
    pub id: String,
    pub sort: u32,
    pub rel_rbum_set_cate_id: String,
    pub rel_rbum_set_cate_name: String,
    pub rel_rbum_item_id: String,
    pub rel_rbum_item_name: String,

    pub own_paths: String,
    pub owner: String,
    pub owner_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}
