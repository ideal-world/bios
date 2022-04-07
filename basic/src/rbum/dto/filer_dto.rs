use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi::Object;

use crate::rbum::rbum_enumeration::{RbumRelFromKind, RbumScopeLevelKind};

#[derive(Object, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct RbumBasicFilterReq {
    pub rel_cxt_scope: bool,
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

impl Default for RbumBasicFilterReq {
    fn default() -> Self {
        Self {
            rel_cxt_scope: false,
            rel_cxt_owner: false,
            own_paths: None,
            ids: None,
            scope_level: None,
            enabled: None,
            name: None,
            code: None,
            rbum_kind_id: None,
            rbum_domain_id: None,
        }
    }
}

#[derive(Object, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct RbumCertFilterReq {
    pub basic: RbumBasicFilterReq,
    pub rbum_item_id: Option<String>,
    pub rbum_cert_conf_id: Option<String>,
}

impl Default for RbumCertFilterReq {
    fn default() -> Self {
        Self {
            basic: Default::default(),
            rbum_item_id: None,
            rbum_cert_conf_id: None,
        }
    }
}

#[derive(Object, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct RbumRelFilterReq {
    pub basic: RbumBasicFilterReq,
    pub rbum_rel_id: Option<String>,
    pub rbum_rel_tag: Option<String>,
    pub rbum_rel_from_kind: Option<RbumRelFromKind>,
    pub rbum_rel_is_from: Option<bool>,
}

impl Default for RbumRelFilterReq {
    fn default() -> Self {
        Self {
            basic: Default::default(),
            rbum_rel_id: None,
            rbum_rel_tag: None,
            rbum_rel_from_kind: None,
            rbum_rel_is_from: None,
        }
    }
}

#[derive(Object, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct RbumItemFilterReq {
    pub basic: RbumBasicFilterReq,
}

impl Default for RbumItemFilterReq {
    fn default() -> Self {
        Self { basic: Default::default() }
    }
}
