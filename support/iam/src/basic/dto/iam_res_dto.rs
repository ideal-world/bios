use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm;
use tardis::web::poem_openapi;

use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;

use crate::basic::dto::iam_set_dto::IamSetItemAggAddReq;
use crate::iam_enumeration::IamResKind;

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamResAggAddReq {
    pub res: IamResAddReq,
    pub set: IamSetItemAggAddReq,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamResAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub code: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: TrimString,
    pub kind: IamResKind,
    pub icon: Option<String>,
    pub sort: Option<i64>,
    #[oai(validator(min_length = "1", max_length = "255"))]
    pub method: Option<TrimString>,
    pub hide: Option<bool>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub action: Option<String>,

    pub scope_level: Option<RbumScopeLevelKind>,

    pub crypto_req: Option<bool>,
    pub crypto_resp: Option<bool>,
    pub double_auth: Option<bool>,
    pub double_auth_msg: Option<String>,
    pub need_login: Option<bool>,
    pub disabled: Option<bool>,
    pub bind_api_res: Option<Vec<String>>,
}

impl IamResAddReq {
    pub fn encoding(&mut self) -> &mut Self {
        if self.code.starts_with('/') {
            self.code = TrimString::new(self.code[1..].to_string());
        }
        self.code = TrimString(format!(
            "{}/{}/{}",
            self.kind.to_int(),
            self.method.as_ref().unwrap_or(&TrimString("*".to_string())),
            self.code
        ));
        self
    }
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamResModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub code: Option<TrimString>,
    pub method: Option<TrimString>,
    pub icon: Option<String>,
    pub sort: Option<i64>,
    pub hide: Option<bool>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub action: Option<String>,

    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,
    pub crypto_req: Option<bool>,
    pub crypto_resp: Option<bool>,
    pub double_auth: Option<bool>,
    pub double_auth_msg: Option<String>,
    pub need_login: Option<bool>,
    pub bind_api_res: Option<Vec<String>>,
}

impl IamResModifyReq {
    pub fn encoding(&mut self, kind: IamResKind, method: String) -> &mut Self {
        if self.code.is_none() {
            return self;
        }
        let code = self.code.clone().unwrap();
        if code.starts_with('/') {
            self.code = Some(TrimString::new(code[1..].to_string()));
        }
        self.code = Some(TrimString(format!("{}/{}/{}", kind.to_int(), method, code)));
        self
    }
}

#[derive(poem_openapi::Object, sea_orm::FromQueryResult, Serialize, Deserialize, Debug)]
pub struct IamResSummaryResp {
    pub id: String,
    pub code: String,
    pub name: String,
    pub kind: IamResKind,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
    pub disabled: bool,

    pub icon: String,
    pub sort: i64,
    pub method: String,
    pub hide: bool,
    pub action: String,
    pub crypto_req: bool,
    pub crypto_resp: bool,
    pub double_auth: bool,
    pub double_auth_msg: String,
    pub need_login: bool,
}

impl IamResSummaryResp {
    pub fn decoding(mut self) -> Self {
        let offset = format!("{}/{}/", self.kind.to_int(), self.method,).len();
        self.code = self.code.chars().skip(offset).collect();
        self
    }
}

#[derive(poem_openapi::Object, sea_orm::FromQueryResult, Serialize, Deserialize, Debug)]
pub struct IamResDetailResp {
    pub id: String,
    pub code: String,
    pub name: String,
    pub kind: IamResKind,

    pub own_paths: String,
    pub owner: String,
    pub owner_name: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
    pub disabled: bool,

    pub icon: String,
    pub sort: i64,
    pub method: String,
    pub hide: bool,
    pub action: String,
    pub crypto_req: bool,
    pub crypto_resp: bool,
    pub double_auth: bool,
    pub double_auth_msg: String,
    pub need_login: bool,
}

impl IamResDetailResp {
    pub fn decoding(mut self) -> Self {
        let offset = format!("{}/{}/", self.kind.to_int(), self.method,).len();
        self.code = self.code.chars().skip(offset).collect();
        self
    }
}

#[derive(Serialize, Deserialize)]
pub struct JsonMenu {
    pub name: String,
    pub bus_code: String,
    pub ext: String,
    pub items: Option<Vec<MenuItem>>,
    pub children: Option<Vec<JsonMenu>>,
}
#[derive(Serialize, Deserialize)]
pub struct MenuItem {
    pub code: String,
    pub name: String,
    pub kind: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamResAppReq {
    pub app_ids: Vec<String>,
    pub res_codes: Vec<String>,
}
