use serde::{Deserialize, Serialize};

use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm;
use tardis::web::poem_openapi;

use crate::iam_enumeration::IamSubDeployHostKind;

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct IamSubDeployHostAddReq {
    pub name: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub sub_deploy_id: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub host: String,
    pub host_type: Option<IamSubDeployHostKind>,
    pub note: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct IamSubDeployHostModifyReq {
    pub name: Option<TrimString>,
    pub host: Option<String>,
    pub note: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object, sea_orm::FromQueryResult))]
pub struct IamSubDeployHostDetailResp {
    pub id: String,
    pub name: String,
    pub sub_deploy_id: String,
    pub host: String,
    pub host_type: String,
    pub note: String,

    pub own_paths: String,
    pub owner: String,
    pub owner_name: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub create_by: String,
    pub update_by: String,
}
