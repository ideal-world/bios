use std::default::Default;

use serde::{Deserialize, Serialize};

use crate::rbum::rbum_enumeration::{RbumCertRelKind, RbumRelFromKind, RbumScopeLevelKind};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object))]
#[serde(default)]
pub struct RbumBasicFilterReq {
    pub ignore_scope: bool,
    pub rel_cxt_owner: bool,

    pub own_paths: Option<String>,
    pub own_paths_with_sub: Option<String>,
    pub ids: Option<Vec<String>>,
    pub scope_level: Option<RbumScopeLevelKind>,
    pub enabled: Option<bool>,
    pub name: Option<String>,
    pub code: Option<String>,
    pub rbum_kind_id: Option<String>,
    pub rbum_domain_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object))]
#[serde(default)]
pub struct RbumCertConfFilterReq {
    pub basic: RbumBasicFilterReq,
    pub rel_rbum_domain_id: Option<String>,
    pub rel_rbum_item_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object))]
#[serde(default)]
pub struct RbumCertFilterReq {
    pub basic: RbumBasicFilterReq,
    pub rel_rbum_kind: Option<RbumCertRelKind>,
    pub rel_rbum_id: Option<String>,
    pub rel_rbum_cert_conf_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object))]
#[serde(default)]
pub struct RbumRelFilterReq {
    pub basic: RbumBasicFilterReq,
    pub tag: Option<String>,
    pub from_rbum_kind: Option<RbumRelFromKind>,
    pub from_rbum_id: Option<String>,
    pub to_rbum_item_id: Option<String>,
    pub to_own_paths: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object))]
#[serde(default)]
pub struct RbumRelExtFilterReq {
    pub basic: RbumBasicFilterReq,
    pub rel_rbum_rel_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object))]
pub struct RbumSetCateFilterReq {
    pub basic: RbumBasicFilterReq,
    pub rel_rbum_set_id: Option<String>,
    pub sys_code: Option<String>,
    pub find_parent: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object))]
pub struct RbumSetItemFilterReq {
    pub basic: RbumBasicFilterReq,
    pub rel_rbum_set_id: Option<String>,
    pub rel_rbum_set_cate_id: Option<String>,
    pub rel_rbum_item_id: Option<String>,
}

pub trait RbumBasicFilterFetcher {
    fn basic(&self) -> &RbumBasicFilterReq;
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object))]
#[serde(default)]
pub struct RbumItemFilterReq {
    pub basic: RbumBasicFilterReq,
}

impl RbumBasicFilterFetcher for RbumItemFilterReq {
    fn basic(&self) -> &RbumBasicFilterReq {
        &self.basic
    }
}
