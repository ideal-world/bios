use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use serde::{Deserialize, Serialize};
use tardis::chrono::{self, Utc};
use tardis::{basic::field::TrimString, web::poem_openapi};

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamOpenAddOrModifyProductReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub code: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
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
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: TrimString,
    pub icon: Option<String>,
    pub url: Option<String>,

    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamOpenBindAkProductReq {
    pub product_code: String,
    pub spec_code: String,
    pub start_time: Option<chrono::DateTime<Utc>>,
    pub end_time: Option<chrono::DateTime<Utc>>,
    pub api_call_frequency: Option<u32>,
    pub api_call_count: Option<u32>,
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
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamOpenAkSkResp {
    pub id: String,
    pub ak: String,
    pub sk: String,
}
