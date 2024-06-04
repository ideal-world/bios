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
    pub bucket_name: Option<String>,
    pub content_type: Option<String>,
    pub private: Option<bool>,
    pub special: Option<bool>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct ObjectBatchBuildCreatePresignUrlReq {
    pub specified_bucket_name: Option<String>,
    pub object_path: String,
    pub upload_id: String,
    pub part_number: u32,
    pub expire_sec: u32,
    pub private: Option<bool>,
    pub special: Option<bool>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct ObjectCompleteMultipartUploadReq {
    pub specified_bucket_name: Option<String>,
    pub object_path: String,
    pub upload_id: String,
    pub parts: Vec<String>,
    pub private: Option<bool>,
    pub special: Option<bool>,
}