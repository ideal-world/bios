use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use serde::{Deserialize, Serialize};
use strum::Display;
use tardis::chrono::{self, Utc};
use tardis::{basic::field::TrimString, web::poem_openapi};

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamOpenAddOrModifyProductReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub code: TrimString,
    pub name: TrimString,
    pub icon: Option<String>,

    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,

    pub specifications: Vec<IamOpenAddSpecReq>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamOpenAddSpecReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub code: TrimString,
    pub name: TrimString,
    pub icon: Option<String>,
    pub url: Option<String>,

    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,

    pub bind_api_res: Option<Vec<String>>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamOpenBindAkProductReq {
    pub product_code: String,
    pub spec_code: String,
    pub start_time: Option<chrono::DateTime<Utc>>,
    pub end_time: Option<chrono::DateTime<Utc>>,
    pub api_call_frequency: Option<u32>,
    pub api_call_count: Option<u32>,
    pub create_proj_code: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct IamOpenRuleResp {
    pub cert_id: String,
    pub spec_code: String,
    pub start_time: Option<chrono::DateTime<Utc>>,
    pub end_time: Option<chrono::DateTime<Utc>>,
    pub api_call_frequency: Option<u32>,
    pub api_call_count: Option<u32>,
    pub api_call_cumulative_count: Option<u32>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamOpenAkSkAddReq {
    pub tenant_id: String,
    pub app_id: Option<String>,
    pub state: Option<IamOpenCertStateKind>,
}

/// 开放平台用户状态类型
#[derive(Display, Serialize, Deserialize, Clone, Debug, Default, PartialEq, poem_openapi::Enum)]
pub enum IamOpenCertStateKind {
    #[default]
    Enabled,
    Disabled,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamOpenAkSkResp {
    pub id: String,
    pub ak: String,
    pub sk: String,
}

// modify_cert_state
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamOpenCertModifyReq {
    pub state: Option<IamOpenCertStateKind>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IamOpenExtendData {
    pub id: String,
}
