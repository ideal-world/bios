use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi::Object;

#[derive(Object, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct RbumBasicFilterReq {
    pub rel_cxt_scope: bool,
    pub rel_cxt_updater: bool,
    pub scope_level: Option<i32>,
    pub enabled: Option<bool>,
    pub name: Option<String>,
    pub code: Option<String>,
    pub ids: Option<Vec<String>>,

    pub rel_scope_ids: Option<String>,

    pub rbum_kind_id: Option<String>,
    pub rbum_domain_id: Option<String>,
    pub rbum_cert_conf_id: Option<String>,

    pub rbum_rel_tag: Option<String>,
    pub rbum_rel_is_from: Option<bool>,
    pub rbum_rel_rbum_kind_id: Option<String>,
    pub rbum_rel_rbum_item_id: Option<String>,
    pub rbum_rel_id: Option<String>,

    pub ignore_scope_check: bool,
}

impl Default for RbumBasicFilterReq {
    fn default() -> Self {
        Self {
            rel_cxt_scope: false,
            rel_cxt_updater: false,
            scope_level: None,
            rbum_kind_id: None,
            rbum_domain_id: None,
            enabled: None,
            name: None,
            code: None,
            ids: None,
            rel_scope_ids: None,
            rbum_cert_conf_id: None,
            rbum_rel_tag: None,
            rbum_rel_is_from: None,
            rbum_rel_rbum_kind_id: None,
            rbum_rel_rbum_item_id: None,
            rbum_rel_id: None,
            ignore_scope_check: false,
        }
    }
}

#[derive(Object, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct RbumItemFilterReq {
    pub rel_cxt_scope: bool,
    pub rel_cxt_updater: bool,
    pub scope_level: Option<i32>,
    pub enabled: Option<bool>,
    pub name: Option<String>,
    pub code: Option<String>,

    pub rel_scope_ids: Option<String>,

    pub ignore_scope_check: bool,
}

impl Default for RbumItemFilterReq {
    fn default() -> Self {
        Self {
            rel_cxt_scope: true,
            rel_cxt_updater: false,
            scope_level: None,
            enabled: None,
            name: None,
            code: None,
            rel_scope_ids: None,
            ignore_scope_check: false,
        }
    }
}
