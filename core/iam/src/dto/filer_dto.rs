use poem_openapi::Object;
use serde::{Deserialize, Serialize};

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RbumBasicFilterReq {
    pub rel_cxt_app: bool,
    pub rel_cxt_tenant: bool,
    pub rel_cxt_creator: bool,
    pub rel_cxt_updater: bool,
    pub scope_kind: Option<String>,
    pub disabled: bool,
}
