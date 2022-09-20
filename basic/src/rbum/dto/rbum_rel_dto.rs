use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::default::Default;
use tardis::chrono::{DateTime, Utc};
#[cfg(feature = "default")]
use tardis::db::sea_orm;
#[cfg(feature = "default")]
use tardis::web::poem_openapi;

use crate::rbum::rbum_enumeration::RbumRelFromKind;

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
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
    pub to_is_outside: bool,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "1000")))]
    pub ext: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct RbumRelModifyReq {
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub tag: Option<String>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "1000")))]
    pub note: Option<String>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "1000")))]
    pub ext: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
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

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
#[serde(default)]
pub struct RbumRelFindReq {
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub tag: Option<String>,
    pub from_rbum_kind: Option<RbumRelFromKind>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub from_rbum_id: Option<String>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub to_rbum_item_id: Option<String>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub from_own_paths: Option<String>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub to_rbum_own_paths: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object, sea_orm::FromQueryResult))]
pub struct RbumRelBoneResp {
    pub tag: String,
    pub note: String,
    pub from_rbum_kind: RbumRelFromKind,
    pub rel_id: String,
    pub rel_name: String,
    pub rel_own_paths: String,
    pub ext: String,
}

impl RbumRelBoneResp {
    pub fn new(detail: RbumRelDetailResp, is_from: bool) -> RbumRelBoneResp {
        if is_from {
            RbumRelBoneResp {
                tag: detail.tag,
                note: detail.note,
                from_rbum_kind: detail.from_rbum_kind,
                rel_id: detail.to_rbum_item_id,
                rel_name: detail.to_rbum_item_name,
                rel_own_paths: detail.to_own_paths,
                ext: detail.ext,
            }
        } else {
            RbumRelBoneResp {
                rel_name: match &detail.from_rbum_kind {
                    RbumRelFromKind::Item => detail.from_rbum_item_name,
                    RbumRelFromKind::Set => detail.from_rbum_set_name,
                    RbumRelFromKind::SetCate => detail.from_rbum_set_cate_name,
                    RbumRelFromKind::Cert => "".to_string(),
                },
                tag: detail.tag,
                note: detail.note,
                from_rbum_kind: detail.from_rbum_kind,
                rel_id: detail.from_rbum_id,
                rel_own_paths: detail.own_paths,
                ext: detail.ext,
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object, sea_orm::FromQueryResult))]
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
