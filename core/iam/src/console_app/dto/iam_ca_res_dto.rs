use crate::iam_enumeration::IamResKind;
use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::web::poem_openapi::Object;

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamCaResAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub code: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: TrimString,
    pub kind: IamResKind,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub icon: Option<String>,
    pub sort: Option<u32>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub method: TrimString,
    pub hide: Option<bool>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub action: Option<String>,

    pub disabled: Option<bool>,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamCaResModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub code: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub icon: Option<String>,
    pub sort: Option<u32>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub method: Option<TrimString>,
    pub hide: Option<bool>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub action: Option<String>,

    pub disabled: Option<bool>,
}
