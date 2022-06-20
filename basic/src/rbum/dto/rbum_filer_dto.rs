use std::default::Default;

use serde::{Deserialize, Serialize};

use crate::rbum::rbum_enumeration::{RbumCertRelKind, RbumCertStatusKind, RbumRelFromKind, RbumScopeLevelKind, RbumSetCateLevelQueryKind};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object))]
#[serde(default)]
pub struct RbumBasicFilterReq {
    pub ignore_scope: bool,
    pub rel_ctx_owner: bool,

    pub own_paths: Option<String>,
    pub with_sub_own_paths: bool,
    pub ids: Option<Vec<String>>,
    pub scope_level: Option<RbumScopeLevelKind>,
    pub enabled: Option<bool>,
    pub name: Option<String>,
    pub code: Option<String>,
    pub rbum_kind_id: Option<String>,
    pub rbum_domain_id: Option<String>,

    pub desc_by_sort: Option<bool>,
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
    pub ak: Option<String>,
    pub status: Option<RbumCertStatusKind>,
    pub rel_rbum_kind: Option<RbumCertRelKind>,
    pub rel_rbum_id: Option<String>,
    pub rel_rbum_cert_conf_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object))]
#[serde(default)]
pub struct RbumKindAttrFilterReq {
    pub basic: RbumBasicFilterReq,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object))]
#[serde(default)]
pub struct RbumItemAttrFilterReq {
    pub basic: RbumBasicFilterReq,
    pub rel_rbum_item_id: Option<String>,
    pub rel_rbum_kind_attr_id: Option<String>,
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
    pub ext_eq: Option<String>,
    pub ext_like: Option<String>,
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
pub struct RbumSetFilterReq {
    pub basic: RbumBasicFilterReq,
    pub rel: Option<RbumItemRelFilterReq>,
    pub kind: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object))]
pub struct RbumSetCateFilterReq {
    pub basic: RbumBasicFilterReq,
    pub rel: Option<RbumItemRelFilterReq>,
    pub rel_rbum_set_id: Option<String>,
    pub sys_code: Option<String>,
    pub find_filter: Option<RbumSetCateLevelQueryKind>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object))]
pub struct RbumSetItemFilterReq {
    pub basic: RbumBasicFilterReq,
    pub rel_rbum_set_id: Option<String>,
    pub rel_rbum_set_cate_id: Option<String>,
    pub rel_rbum_item_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object))]
#[serde(default)]
pub struct RbumItemRelFilterReq {
    pub rel_by_from: bool,
    pub tag: Option<String>,
    pub from_rbum_kind: Option<RbumRelFromKind>,
    pub rel_item_id: Option<String>,
    pub ext_eq: Option<String>,
    pub ext_like: Option<String>,
}

pub trait RbumItemFilterFetcher {
    fn basic(&self) -> &RbumBasicFilterReq;
    fn rel(&self) -> &Option<RbumItemRelFilterReq>;
    fn rel2(&self) -> &Option<RbumItemRelFilterReq>;
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(tardis::web::poem_openapi::Object))]
#[serde(default)]
pub struct RbumItemBasicFilterReq {
    pub basic: RbumBasicFilterReq,
    pub rel: Option<RbumItemRelFilterReq>,
}

impl RbumItemFilterFetcher for RbumItemBasicFilterReq {
    fn basic(&self) -> &RbumBasicFilterReq {
        &self.basic
    }
    fn rel(&self) -> &Option<RbumItemRelFilterReq> {
        &self.rel
    }
    fn rel2(&self) -> &Option<RbumItemRelFilterReq> {
        &self.rel
    }
}
