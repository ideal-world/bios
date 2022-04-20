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
    pub ids: Option<Vec<String>>,
    pub scope_level: Option<RbumScopeLevelKind>,
    pub enabled: Option<bool>,
    pub name: Option<String>,
    pub code: Option<String>,
    pub rbum_kind_id: Option<String>,
    pub rbum_domain_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object))]
#[serde(default)]
pub struct RbumCertConfFilterReq {
    pub basic: RbumBasicFilterReq,
    pub rel_rbum_domain_id: Option<String>,
    pub rel_rbum_item_id: Option<String>,
}

impl Default for RbumCertConfFilterReq {
    fn default() -> Self {
        Self {
            basic: Default::default(),
            rel_rbum_domain_id: None,
            rel_rbum_item_id: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object))]
#[serde(default)]
pub struct RbumCertFilterReq {
    pub basic: RbumBasicFilterReq,
    pub rel_rbum_kind: Option<RbumCertRelKind>,
    pub rel_rbum_id: Option<String>,
    pub rel_rbum_cert_conf_id: Option<String>,
}

impl Default for RbumCertFilterReq {
    fn default() -> Self {
        Self {
            basic: Default::default(),
            rel_rbum_kind: None,
            rel_rbum_id: None,
            rel_rbum_cert_conf_id: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

impl Default for RbumRelFilterReq {
    fn default() -> Self {
        Self {
            basic: Default::default(),
            tag: None,
            from_rbum_kind: None,
            from_rbum_id: None,
            to_rbum_item_id: None,
            to_own_paths: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object))]
#[serde(default)]
pub struct RbumRelExtFilterReq {
    pub basic: RbumBasicFilterReq,
    pub rel_rbum_rel_id: Option<String>,
}

impl Default for RbumRelExtFilterReq {
    fn default() -> Self {
        Self {
            basic: Default::default(),
            rel_rbum_rel_id: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object))]
pub struct RbumSetCateFilterReq {
    pub basic: RbumBasicFilterReq,
    pub rel_rbum_set_id: Option<String>,
    pub sys_code: Option<String>,
    pub find_parent: Option<bool>,
}

impl Default for RbumSetCateFilterReq {
    fn default() -> Self {
        Self {
            basic: Default::default(),
            rel_rbum_set_id: None,
            sys_code: None,
            find_parent: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object))]
pub struct RbumSetItemFilterReq {
    pub basic: RbumBasicFilterReq,
    pub rel_rbum_set_id: Option<String>,
    pub rel_rbum_set_cate_id: Option<String>,
    pub rel_rbum_item_id: Option<String>,
}

impl Default for RbumSetItemFilterReq {
    fn default() -> Self {
        Self {
            basic: Default::default(),
            rel_rbum_set_id: None,
            rel_rbum_set_cate_id: None,
            rel_rbum_item_id: None,
        }
    }
}

pub trait RbumBasicFilterFetcher {
    fn basic(&self) -> &RbumBasicFilterReq;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object))]
#[serde(default)]
pub struct RbumItemFilterReq {
    pub basic: RbumBasicFilterReq,
}

impl Default for RbumItemFilterReq {
    fn default() -> Self {
        Self { basic: Default::default() }
    }
}

impl RbumBasicFilterFetcher for RbumItemFilterReq {
    fn basic(&self) -> &RbumBasicFilterReq {
        &self.basic
    }
}
