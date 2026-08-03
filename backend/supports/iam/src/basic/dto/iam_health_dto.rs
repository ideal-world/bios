use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi;

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct IamHealthComponentResp {
    pub healthy: bool,
    pub detail: Option<String>,
}


#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct IamHealthResp {
    pub healthy: bool,
    pub service: IamHealthComponentResp,
    pub database: IamHealthComponentResp,
    pub redis: IamHealthComponentResp,
    pub mq: IamHealthComponentResp,
}

