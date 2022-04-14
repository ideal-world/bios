use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::web::poem_openapi::Object;

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamCsTenantAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tenant_name: TrimString,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub tenant_icon: Option<String>,

    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tenant_contact_phone: Option<String>,

    #[oai(validator(min_length = "2", max_length = "255"))]
    pub admin_username: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub admin_name: TrimString,

    pub disabled: Option<bool>,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamCsTenantModifyReq {
    pub disabled: Option<bool>,
}
