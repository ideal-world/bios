use serde::{Deserialize, Serialize};

use tardis::web::poem_openapi;

#[derive(Serialize, Deserialize, Debug)]
pub enum ObjectObjPresignKind {
    Upload,
    Delete,
    View,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct ObjectInitiateMultipartUploadReq {
    pub object_path: String,
    pub content_type: Option<String>,
    pub private: Option<bool>,
    pub special: Option<bool>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct ObjectBatchBuildCreatePresignUrlReq {
    pub object_path: String,
    pub upload_id: String,
    pub part_number: u32,
    pub expire_sec: u32,
    pub private: Option<bool>,
    pub special: Option<bool>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct ObjectCompleteMultipartUploadReq {
    pub object_path: String,
    pub upload_id: String,
    pub parts: Vec<String>,
    pub private: Option<bool>,
    pub special: Option<bool>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct ObjectCopyReq {
    pub from: String,
    pub to: String,
    pub private: Option<bool>,
    pub special: Option<bool>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct ObjectBatchDeleteReq {
    pub object_path: Vec<String>,
    pub private: Option<bool>,
    pub special: Option<bool>,
    pub obj_exp: Option<u32>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct ObjectPresignBatchViewReq {
    pub object_path: Vec<String>,
    pub expire_sec: u32,
    pub private: Option<bool>,
    pub special: Option<bool>,
    pub obj_exp: Option<u32>,
}
