use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use bios_basic::rbum::enumeration::RbumScopeKind;

#[derive(Serialize, Deserialize, Debug)]
pub struct IamCtTenantModifyReq {
    pub name: Option<TrimString>,
    pub icon: Option<String>,
    pub sort: Option<i32>,

    pub contact_phone: Option<String>,

    pub scope_kind: Option<RbumScopeKind>,
    pub disabled: Option<bool>,
}
