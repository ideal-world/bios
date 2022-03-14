use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi::Object;

use crate::rbum::enumeration::RbumScopeKind;

#[derive(Object, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct RbumBasicFilterReq {
    pub rel_cxt_app: bool,
    pub rel_cxt_tenant: bool,
    pub rel_cxt_updater: bool,
    pub scope_kind: Option<RbumScopeKind>,
    pub kind_id: Option<String>,
    pub domain_id: Option<String>,
    pub enabled: Option<bool>,
}

impl Default for RbumBasicFilterReq {
    fn default() -> Self {
        Self {
            rel_cxt_app: true,
            rel_cxt_tenant: true,
            rel_cxt_updater: false,
            scope_kind: Some(RbumScopeKind::App),
            kind_id: None,
            domain_id: None,
            enabled: None,
        }
    }
}
