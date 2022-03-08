use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi::Object;

use crate::rbum::enumeration::RbumScopeKind;

#[derive(Default, Object, Serialize, Deserialize, Debug)]
pub struct RbumBasicFilterReq {
    pub rel_cxt_app: bool,
    pub rel_cxt_tenant: bool,
    pub rel_cxt_updater: bool,
    pub scope_kind: Option<RbumScopeKind>,
    pub kind_id: Option<String>,
    pub domain_id: Option<String>,
    pub disabled: bool,
}
