use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi;

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct KReq {
    #[oai(validator(min_length = "1"))]
    pub key: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct ExpReq {
    #[oai(validator(min_length = "1"))]
    pub key: String,
    pub exp_sec: u64,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct KvReq {
    #[oai(validator(min_length = "1"))]
    pub key: String,
    pub value: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct KvWithExReq {
    #[oai(validator(min_length = "1"))]
    pub key: String,
    pub value: String,
    pub exp_sec: u64,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct KIncrReq {
    #[oai(validator(min_length = "1"))]
    pub key: String,
    pub delta: i64,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct KfReq {
    #[oai(validator(min_length = "1"))]
    pub key: String,
    #[oai(validator(min_length = "1"))]
    pub field: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct KfvReq {
    #[oai(validator(min_length = "1"))]
    pub key: String,
    #[oai(validator(min_length = "1"))]
    pub field: String,
    pub value: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct KfIncrReq {
    #[oai(validator(min_length = "1"))]
    pub key: String,
    #[oai(validator(min_length = "1"))]
    pub field: String,
    pub delta: i64,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct KbvReq {
    #[oai(validator(min_length = "1"))]
    pub key: String,
    pub offset: u32,
    pub value: bool,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct KbReq {
    #[oai(validator(min_length = "1"))]
    pub key: String,
    pub offset: u32,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct KbRagngeReq {
    #[oai(validator(min_length = "1"))]
    pub key: String,
    pub start: u32,
    pub end: u32,
}
