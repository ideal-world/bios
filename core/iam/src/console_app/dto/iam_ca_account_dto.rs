use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::web::poem_openapi::Object;
use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamCaAccountAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: TrimString,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub icon: Option<String>,

    pub scope_level: RbumScopeLevelKind,

    pub disabled: Option<bool>,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamCaAccountModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub icon: Option<String>,

    pub scope_level: Option<RbumScopeLevelKind>,

    pub disabled: Option<bool>,
}
