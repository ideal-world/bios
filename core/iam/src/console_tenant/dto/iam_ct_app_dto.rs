use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::web::poem_openapi::Object;

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamCtAppAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub app_name: TrimString,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub app_icon: Option<String>,
    pub app_sort: Option<u32>,

    #[oai(validator(min_length = "2", max_length = "255"))]
    pub app_contact_phone: Option<String>,

    #[oai(validator(min_length = "2", max_length = "255"))]
    pub admin_id: String,

    pub disabled: Option<bool>,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamCtAppModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub icon: Option<String>,
    pub sort: Option<u32>,

    #[oai(validator(min_length = "2", max_length = "255"))]
    pub contact_phone: Option<String>,

    pub disabled: Option<bool>,
}
