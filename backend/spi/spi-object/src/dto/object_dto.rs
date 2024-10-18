use serde::{Deserialize, Serialize};

use tardis::{basic::field::TrimString, web::poem_openapi};

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
    // 服务ID，使用外部自定义服务时，传入该值。
    // Service ID, pass this value when using an external custom service.
    pub bs_id: Option<String>,
    // 指定桶，当且仅当使用自定义服务ID时该参数有效。
    // Specifies the bucket. This parameter is valid when and only when a custom service ID is used.
    pub bucket: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct ObjectBatchBuildCreatePresignUrlReq {
    pub object_path: String,
    pub upload_id: String,
    pub part_number: u32,
    pub expire_sec: u32,
    pub private: Option<bool>,
    pub special: Option<bool>,
    // 服务ID，使用外部自定义服务时，传入该值。
    // Service ID, pass this value when using an external custom service.
    pub bs_id: Option<String>,
    // 指定桶，当且仅当使用自定义服务ID时该参数有效。
    // Specifies the bucket. This parameter is valid when and only when a custom service ID is used.
    pub bucket: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct ObjectCompleteMultipartUploadReq {
    pub object_path: String,
    pub upload_id: String,
    pub parts: Vec<String>,
    pub private: Option<bool>,
    pub special: Option<bool>,
    // 服务ID，使用外部自定义服务时，传入该值。
    // Service ID, pass this value when using an external custom service.
    pub bs_id: Option<String>,
    // 指定桶，当且仅当使用自定义服务ID时该参数有效。
    // Specifies the bucket. This parameter is valid when and only when a custom service ID is used.
    pub bucket: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct ObjectCopyReq {
    pub from: String,
    pub to: String,
    pub private: Option<bool>,
    pub special: Option<bool>,
    // 服务ID，使用外部自定义服务时，传入该值。
    // Service ID, pass this value when using an external custom service.
    pub bs_id: Option<String>,
    // 指定桶，当且仅当使用自定义服务ID时该参数有效。
    // Specifies the bucket. This parameter is valid when and only when a custom service ID is used.
    pub bucket: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct ObjectBatchDeleteReq {
    pub object_path: Vec<String>,
    pub private: Option<bool>,
    pub special: Option<bool>,
    pub obj_exp: Option<u32>,
    // 服务ID，使用外部自定义服务时，传入该值。
    // Service ID, pass this value when using an external custom service.
    pub bs_id: Option<String>,
    // 指定桶，当且仅当使用自定义服务ID时该参数有效。
    // Specifies the bucket. This parameter is valid when and only when a custom service ID is used.
    pub bucket: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct ObjectPresignBatchViewReq {
    pub object_path: Vec<String>,
    pub expire_sec: u32,
    pub private: Option<bool>,
    pub special: Option<bool>,
    pub obj_exp: Option<u32>,
    // 服务ID，使用外部自定义服务时，传入该值。
    // Service ID, pass this value when using an external custom service.
    pub bs_id: Option<String>,
    // 指定桶，当且仅当使用自定义服务ID时该参数有效。
    // Specifies the bucket. This parameter is valid when and only when a custom service ID is used.
    pub bucket: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct ClientCreateReq {
    pub name: TrimString,
    pub conn_uri: String,
    pub ak: TrimString,
    pub sk: TrimString,
    pub ext: String,
}