use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm;
use tardis::web::poem_openapi;

use crate::iam_enumeration::{IamConfigDataTypeKind, IamConfigKind};

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct IamConfigAddReq {
    pub code: IamConfigKind,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<String>,
    pub data_type: IamConfigDataTypeKind,
    pub note: Option<String>,
    pub value1: Option<String>,
    pub value2: Option<String>,
    pub ext: Option<String>,
    pub disabled: Option<bool>,
    pub rel_item_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct IamConfigModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<String>,
    pub data_type: Option<IamConfigDataTypeKind>,
    pub note: Option<String>,
    pub value1: Option<String>,
    pub value2: Option<String>,
    pub ext: Option<String>,
    pub disabled: Option<bool>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Clone, Debug)]
pub struct IamConfigAggOrModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<String>,
    pub data_type: IamConfigDataTypeKind,
    pub note: Option<String>,
    pub value1: Option<String>,
    pub value2: Option<String>,
    pub ext: Option<String>,
    pub disabled: Option<bool>,
    pub code: IamConfigKind,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object, sea_orm::FromQueryResult))]
pub struct IamConfigSummaryResp {
    pub id: String,
    pub code: String,
    pub name: String,
    pub data_type: String,
    pub note: String,
    pub value1: String,
    pub value2: String,
    pub ext: String,
    pub rel_item_id: String,
    pub disabled: bool,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object, sea_orm::FromQueryResult))]
pub struct IamConfigDetailResp {
    pub id: String,
    pub code: String,
    pub name: String,
    pub data_type: String,
    pub note: String,
    pub value1: String,
    pub value2: String,
    pub ext: String,
    pub rel_item_id: String,
    pub disabled: bool,

    pub own_paths: String,
    pub owner: String,
    pub owner_name: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}
