use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::web::poem_openapi::Object;
use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;

#[derive(Object, Serialize, Deserialize, Debug)]
#[oai(rename_all = "camelCase")]
pub struct IamCtTenantModifyReq {
    pub name: Option<TrimString>,
    pub icon: Option<String>,
    pub sort: Option<u32>,

    pub contact_phone: Option<String>,

    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,
}
