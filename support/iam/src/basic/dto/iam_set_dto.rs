use bios_basic::rbum::dto::rbum_set_dto::{RbumSetTreeExtResp, RbumSetTreeMainResp};
use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::web::poem_openapi;

use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamSetCateAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: TrimString,
    pub scope_level: Option<RbumScopeLevelKind>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub bus_code: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub icon: Option<String>,
    pub sort: Option<i64>,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub ext: Option<String>,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub rbum_parent_cate_id: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamSetCateModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<TrimString>,
    pub scope_level: Option<RbumScopeLevelKind>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub bus_code: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub icon: Option<String>,
    pub sort: Option<i64>,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub ext: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamSetItemAggAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub set_cate_id: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamSetItemWithDefaultSetAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub set_cate_id: Option<String>,
    pub sort: i64,

    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub rel_rbum_item_id: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamSetItemAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub set_id: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub set_cate_id: String,
    pub sort: i64,

    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub rel_rbum_item_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct IamSetTreeResp {
    pub main: Vec<RbumSetTreeMainResp>,
    pub ext: Option<RbumSetTreeExtResp>,
    pub rel: Option<String>,
}
