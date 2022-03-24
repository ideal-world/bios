use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::web::poem_openapi::Object;

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamCtTenantModifyReq {
    pub name: Option<TrimString>,
    pub icon: Option<String>,
    pub sort: Option<i32>,

    pub contact_phone: Option<String>,

    pub scope_level: Option<i32>,
    pub disabled: Option<bool>,
}
