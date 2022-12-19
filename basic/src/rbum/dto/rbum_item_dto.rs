use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
#[cfg(feature = "default")]
use tardis::db::sea_orm;
#[cfg(feature = "default")]
use tardis::web::poem_openapi;

use crate::rbum::rbum_enumeration::RbumScopeLevelKind;

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct RbumItemAddReq {
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub id: Option<TrimString>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub code: Option<TrimString>,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub name: TrimString,

    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub rel_rbum_kind_id: String,
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub rel_rbum_domain_id: String,

    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,
}

/// For security reasons, this object cannot be used as an input to the API
#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct RbumItemKernelAddReq {
    pub id: Option<TrimString>,
    pub code: Option<TrimString>,
    pub name: TrimString,
    // Special kind can be set, otherwise the default kind will be used.
    // Note that setting special kind must ensure that the permissions are correct.
    pub rel_rbum_kind_id: Option<String>,
    // Special domain can be set, otherwise the default domain will be used.
    // Note that setting special domain must ensure that the permissions are correct.
    pub rel_rbum_domain_id: Option<String>,
    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,
}

impl Default for RbumItemKernelAddReq {
    fn default() -> Self {
        Self {
            id: None,
            code: None,
            name: TrimString("".to_string()),
            rel_rbum_kind_id: None,
            rel_rbum_domain_id: None,
            scope_level: None,
            disabled: None,
        }
    }
}

/// For security reasons, this object cannot be used as an input to the API
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct RbumItemKernelModifyReq {
    pub code: Option<TrimString>,
    pub name: Option<TrimString>,
    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object, sea_orm::FromQueryResult))]
pub struct RbumItemSummaryResp {
    pub id: String,
    pub code: String,
    pub name: String,
    pub rel_rbum_kind_id: String,
    pub rel_rbum_domain_id: String,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
    pub disabled: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object, sea_orm::FromQueryResult))]
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
    pub owner_name: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
    pub disabled: bool,
}
