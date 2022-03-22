use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi::Object;

use crate::rbum::enumeration::RbumScopeKind;

#[derive(Object, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct RbumBasicFilterReq {
    pub rel_cxt_app: bool,
    pub rel_cxt_updater: bool,
    pub scope_kind: Option<RbumScopeKind>,
    pub enabled: Option<bool>,
    pub name: Option<String>,
    pub code: Option<String>,
    pub ids: Option<Vec<String>>,

    pub rbum_kind_id: Option<String>,
    pub rbum_domain_id: Option<String>,
    pub rbum_cert_conf_id: Option<String>,

    pub rbum_rel_tag: Option<String>,
    pub rbum_rel_is_from: Option<bool>,
    pub rbum_rel_rbum_kind_id: Option<String>,
    pub rbum_rel_rbum_item_id: Option<String>,
    pub rbum_rel_app_code: Option<String>,
    pub rbum_rel_id: Option<String>,
}

impl Default for RbumBasicFilterReq {
    fn default() -> Self {
        Self {
            rel_cxt_app: true,
            rel_cxt_updater: false,
            scope_kind: None,
            rbum_kind_id: None,
            rbum_domain_id: None,
            enabled: None,
            name: None,
            code: None,
            ids: None,
            rbum_cert_conf_id: None,
            rbum_rel_tag: None,
            rbum_rel_is_from: None,
            rbum_rel_rbum_kind_id: None,
            rbum_rel_rbum_item_id: None,
            rbum_rel_app_code: None,
            rbum_rel_id: None,
        }
    }
}

#[derive(Object, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct RbumItemFilterReq {
    pub rel_cxt_app: bool,
    pub rel_cxt_updater: bool,
    pub scope_kind: Option<RbumScopeKind>,
    pub enabled: Option<bool>,
    pub name: Option<String>,
    pub code: Option<String>,
}

impl Default for RbumItemFilterReq {
    fn default() -> Self {
        Self {
            rel_cxt_app: true,
            rel_cxt_updater: false,
            scope_kind: None,
            enabled: None,
            name: None,
            code: None,
        }
    }
}