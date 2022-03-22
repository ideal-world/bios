use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm::FromQueryResult;
use tardis::web::poem_openapi::Object;

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamCaHttpResAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: TrimString,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub uri_path: TrimString,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub icon: Option<String>,

    #[oai(validator(min_length = "2", max_length = "255"))]
    pub method: TrimString,

    pub disabled: Option<bool>,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamCaHttpResModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub uri_path: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub icon: Option<String>,

    #[oai(validator(min_length = "2", max_length = "255"))]
    pub method: Option<TrimString>,

    pub disabled: Option<bool>,
}

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
pub struct IamCaHttpResSummaryResp {
    pub id: String,
    pub uri_path: String,
    pub name: String,
    pub icon: String,

    pub method: String,

    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub disabled: bool,
}

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
pub struct IamCaHttpResDetailResp {
    pub id: String,
    pub uri_path: String,
    pub name: String,
    pub icon: String,

    pub method: String,

    pub updater_code: String,
    pub updater_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub disabled: bool,
}
