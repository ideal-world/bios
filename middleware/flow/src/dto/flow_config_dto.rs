use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi;

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowConfigModifyReq {
    pub code: String,
    pub value: String,
}
