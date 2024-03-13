use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use serde::{Deserialize, Serialize};
use tardis::{basic::field::TrimString, web::poem_openapi};

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamOpenAddProductReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub code: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: TrimString,
    pub icon: Option<String>,

    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,

    pub specifications: Vec<IamOpenAddSpecReq>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamOpenAddSpecReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub code: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: TrimString,
    pub icon: Option<String>,
    pub url: Option<String>,

    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,
}