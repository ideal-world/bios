use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::web::poem_openapi::Object;

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamCpAccountModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub icon: Option<String>,

    pub disabled: Option<bool>,
}
