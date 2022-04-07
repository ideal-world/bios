use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
use tardis::web::poem_openapi::Object;
use crate::rbum::rbum_enumeration::RbumScopeLevelKind;

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumItemAddReq {
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub id: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub code: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: TrimString,
    
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub rel_rbum_kind_id: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub rel_rbum_domain_id: String,

    pub scope_level: RbumScopeLevelKind,
    pub disabled: Option<bool>,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumItemModifyReq {
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub code: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<TrimString>,

    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object, tardis::db::sea_orm::FromQueryResult))]
pub struct RbumItemSummaryResp {
    pub id: String,
    pub code: String,
    pub name: String,

    pub own_paths: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
    pub disabled: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object, tardis::db::sea_orm::FromQueryResult))]
pub struct RbumItemDetailResp {
    pub id: String,
    pub code: String,
    pub name: String,
    pub rel_rbum_kind_id: String,
    pub rel_rbum_kind_name: String,
    pub rel_rbum_domain_id: String,
    pub rel_rbum_domain_name: String,

    pub own_paths: String,
    pub owner: String,
    pub owner_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
    pub disabled: bool,
}
