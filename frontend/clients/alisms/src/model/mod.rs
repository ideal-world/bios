use std::collections::HashMap;

use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use tardis::serde_json::{json, Value};

use crate::percent_code;

#[derive(Debug, Serialize, Deserialize)]
pub struct SmsResponse {
    #[serde(rename = "Message")]
    pub message: String,
    #[serde(rename = "RequestId")]
    pub request_id: String,
    #[serde(rename = "Code")]
    pub code: String,
    #[serde(rename = "BizId")]
    pub biz_id: Option<String>,
}

impl SmsResponse {
    pub fn is_error(&self) -> bool {
        self.biz_id.is_none()
    }
    pub fn is_ok(&self) -> bool {
        self.biz_id.is_some()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmsId {
    pub sms_msg_id: String,
    pub from: String,
    pub origin_to: String,
    pub status: String,
    pub create_time: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
/// see: [SmsClientBatchDiffSendRequest]
///
/// reference: https://support.huaweicloud.com/api-msgsms/sms_05_0002.html#ZH-CN_TOPIC_0000001430352905__table4039578
pub struct SmsContent<'r> {
    pub template_code: &'r str,
    pub template_param: HashMap<String, String>,
    pub sign_name: &'r str,
}

// 定义 FormData 类型数据的value类型
#[derive(Debug, Clone)]
pub enum FormValue {
    String(String),
    Vec(Vec<String>),
    HashMap(HashMap<String, String>),
}

// 定义一个body请求体枚举，用于统一处理请求体类型,包含Json/Binary/FormData类型 
#[derive(Debug)]
pub enum RequestBody {
    Json(HashMap<String, Value>), // Json
    Binary(Vec<u8>), // Binary
    FormData(HashMap<String, FormValue>), //  FormData 
    None,
}

impl ToString for RequestBody {
    fn to_string(&self) -> String {
        match self { 
            RequestBody::Json(body_map) => json!(body_map).to_string(),  
            RequestBody::Binary(binary_data) => {
                STANDARD.encode(binary_data)
            },
            RequestBody::FormData(form_data) => {
                let params: Vec<String> = form_data
                .iter()
                .flat_map(|(k, v)| {
                    match v {
                        FormValue::String(s) => {
                            vec![format!("{}={}", percent_code(k), percent_code(&s))]
                        },
                        FormValue::Vec(vec) => {
                            vec.iter()
                                .map(|s| format!("{}={}", percent_code(k), percent_code(s)))
                                .collect::<Vec<_>>()
                        },
                        FormValue::HashMap(map) => {
                            map.iter()
                                .map(|(sk, sv)| format!("{}={}", percent_code(sk), percent_code(sv)))
                                .collect::<Vec<_>>()
                        },
                    }
                })
                .collect();
                params.join("&") 
            },
            RequestBody::None => String::new(),
        }
    }
}