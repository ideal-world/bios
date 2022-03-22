use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::web::poem_openapi::Object;

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamCsTenantAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: TrimString,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub icon: Option<String>,

    #[oai(validator(min_length = "2", max_length = "255"))]
    pub contact_phone: Option<String>,

    pub disabled: Option<bool>,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamCsTenantModifyReq {
    pub disabled: Option<bool>,
}
